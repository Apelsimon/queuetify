use crate::configuration::Settings;
use crate::controller;
use crate::controller::messages::{
    SearchComplete, SearchResultPayload, StateUpdate, StateUpdatePayload,
};
use crate::controller::Controller;
use crate::db::Database;
use crate::spotify::get_spotify_from_db;
use actix::Addr;
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::device::Device;
use rspotify::model::enums::misc::Market;
use rspotify::model::enums::types::DeviceType;
use rspotify::model::{AdditionalType, PlayableId, PlayableItem, SimplifiedArtist};
use rspotify::model::{FullTrack, TrackId};
use rspotify::model::{SearchResult::Tracks, SearchType};
use rspotify::AuthCodeSpotify;
use rspotify::ClientError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub enum SessionAgentRequest {
    Search((controller::Search, Addr<Controller>)),
    Queue((controller::Queue, Addr<Controller>)),
    PollState((Uuid, Addr<Controller>)),
}

pub struct SessionAgent {
    rx: UnboundedReceiver<SessionAgentRequest>,
    db: Database,
}

impl SessionAgent {
    pub fn build(settings: Settings) -> (Self, UnboundedSender<SessionAgentRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let agent = Self {
            rx,
            db: Database::new(&settings.database),
        };
        (agent, tx)
    }

    pub async fn run(mut self) -> Result<(), std::io::Error> {
        loop {
            let request = match self.rx.recv().await {
                Some(request) => request,
                None => return Ok(()),
            };

            match request {
                SessionAgentRequest::Search((msg, addr)) => {
                    if let Ok(search_result) = on_search(&msg, &self.db).await {
                        // TODO: have on search return complete SearchComplete strutc
                        addr.do_send(SearchComplete {
                            result: SearchResultPayload {
                                payload: search_result,
                            },
                            connection_id: msg.connection_id,
                        });
                    }
                }
                SessionAgentRequest::Queue((msg, addr)) => {
                    let result = on_queue(msg, &self.db).await;
                    match result {
                        Ok(update) => {
                            addr.do_send(update);
                        }
                        Err(err) => {
                            log::error!("Error on queue {err}");
                        }
                    }
                }
                SessionAgentRequest::PollState((id, addr)) => {
                    match on_poll_state(id, &self.db).await {
                        Ok(update) => {
                            if let Some(update) = update {
                                addr.do_send(update);
                            }
                        },
                        Err(err) => {
                            log::error!("Error on update state {err}");
                        }
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct TrackInfo {
    name: String,
    artists: Vec<String>,
    id: String,
}

impl TryFrom<FullTrack> for TrackInfo {
    type Error = ();

    fn try_from(track: FullTrack) -> Result<Self, Self::Error> {
        let track_id = match track.id {
            Some(tid) => tid,
            None => return Err(()),
        };

        Ok(TrackInfo {
            name: track.name,
            artists: build_artist_string_vec(&track.artists),
            id: track_id.to_string(),
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchResult {
    tracks: Vec<TrackInfo>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct State {
    current_track: Option<TrackInfo>,
    current_queue: Vec<TrackInfo>,
}

fn build_artist_string_vec(artists: &Vec<SimplifiedArtist>) -> Vec<String> {
    let mut artist_string_vec = Vec::new();

    for artist in artists.iter() {
        artist_string_vec.push(artist.name.clone());
    }

    artist_string_vec
}

async fn get_search_results(
    spotify: &AuthCodeSpotify,
    input: &str,
) -> Result<SearchResult, ClientError> {
    let mut search_result = SearchResult { tracks: Vec::new() };

    if let Tracks(track_pages) = spotify
        .search(
            &input,
            &SearchType::Track,
            Some(&Market::FromToken),
            None,
            Some(10),
            None,
        )
        .await?
    {
        for item in track_pages.items {
            let track_info = match TrackInfo::try_from(item) {
                Ok(info) => info,
                Err(_) => continue,
            };
            search_result.tracks.push(track_info);
        }
    }

    Ok(search_result)
}

async fn on_search(msg: &controller::Search, db: &Database) -> Result<SearchResult, ()> {
    if let Ok(spotify) = get_spotify_from_db(msg.session_id, &db).await {
        if let Ok(search_result) = get_search_results(&spotify, &msg.query).await {
            return Ok(search_result);
        }
    }

    Err(())
}

//TODO: should be removed when device is selectable
async fn get_device(spotify: &AuthCodeSpotify) -> Result<Device, anyhow::Error> {
    match spotify.device().await {
        Ok(devices) => {
            for device in devices.into_iter() {
                if device._type == DeviceType::Computer {
                    return Ok(device);
                }
            }
        }
        Err(err) => {
            log::error!("Failed to fetch devices {err}");
        }
    }

    Err(anyhow::Error::msg("No computer device found"))
}

async fn start_playback(spotify: &AuthCodeSpotify, id: TrackId) -> Result<(), anyhow::Error> {
    // TODO: make device selectable when session is created/started
    let device = get_device(&spotify).await?;
    let uri: Box<dyn PlayableId> = Box::new(id);
    spotify
        .start_uris_playback(
            Some(uri.as_ref()),
            Some(device.id.as_deref().unwrap_or("")),
            None,
            None,
        )
        .await?;
    Ok(())
}

async fn get_current_state(
    id: Uuid,
    spotify: &AuthCodeSpotify,
    db: &Database,
) -> Result<StateUpdate, anyhow::Error> {
    let state = db.get_current_state(id).await?;
    let current_track = match state.current_track_uri {
        Some(current_track_id) => {
            let current_track = spotify.track(&current_track_id).await?;
            match TrackInfo::try_from(current_track) {
                Ok(info) => Some(info),
                Err(_) => None,
            }
        }
        None => None,
    };

    let mut current_queue = Vec::new();

    if !state.current_queue.is_empty() {
        let tracks = state.current_queue.iter().collect::<Vec<_>>();
        let queue = spotify.tracks(tracks, None).await?;
        

        for track in queue.iter() {
            match TrackInfo::try_from(track.clone()) {
                Ok(info) => {
                    current_queue.push(info);
                }
                Err(_) => {}
            }
        }
    }    

    let payload = State {
        current_track,
        current_queue,
    };
    Ok(StateUpdate {
        update: StateUpdatePayload { payload },
        session_id: id,
    })
}

async fn on_queue(msg: controller::Queue, db: &Database) -> Result<StateUpdate, anyhow::Error> {
    let spotify = get_spotify_from_db(msg.session_id, &db).await?;

    let (track, transaction) = db.get_current_track(msg.session_id).await?;
    match track {
        Some(_) => {
            db.queue_track(transaction, msg.session_id, msg.track_id)
                .await?;
        }
        None => {
            let _ = start_playback(&spotify, msg.track_id.clone()).await?;
            let _ = db
                .set_current_track(transaction, msg.session_id, Some(msg.track_id))
                .await?;
        }
    }

    let state = get_current_state(msg.session_id, &spotify, &db).await?;
    Ok(state)
}

pub const UPDATE_STATE_INTERVAL: Duration = Duration::from_secs(5);

async fn on_poll_state(id: Uuid, db: &Database) -> Result<Option<StateUpdate>, anyhow::Error> {
    log::info!("update state for {id}");
    let spotify = get_spotify_from_db(id, &db).await?;
    let (track, mut transaction) = db.get_current_track(id).await?;
    match track {
        Some(expected_playing_id) => {
            match spotify
                .current_playing(None, None::<Vec<&AdditionalType>>)
                .await?
            {
                Some(current_playing_context) => match current_playing_context.item {
                    Some(PlayableItem::Track(track)) => {
                        if let Some(actual_playing_id) = track.id {
                            if actual_playing_id != expected_playing_id {
                                let _ = start_playback(&spotify, expected_playing_id).await?;
                            } else {
                                if current_playing_context.is_playing {
                                    match current_playing_context.progress {
                                        Some(duration) => {
                                            if (track.duration - duration) < UPDATE_STATE_INTERVAL {
                                                match db
                                                    .pop_track_from_queue(id, &mut transaction)
                                                    .await?
                                                {
                                                    Some(new_track) => {
                                                        db.set_current_track(
                                                            transaction,
                                                            id,
                                                            Some(new_track.clone()),
                                                        )
                                                        .await?;
                                                        spotify
                                                            .add_item_to_queue(&new_track, None)
                                                            .await?
                                                    }
                                                    None => {
                                                        db.set_current_track(transaction, id, None)
                                                            .await?;
                                                    }
                                                }

                                                let state = get_current_state(id, &spotify, &db).await?;
                                                return Ok(Some(state));
                                            }
                                        }
                                        None => {
                                            log::error!(
                                                "Progress missing for current playing context!"
                                            );
                                            todo!();
                                        }
                                    }
                                } else {
                                    spotify.resume_playback(None, None).await?;
                                }
                            }
                        } else {
                            log::error!("Track id missing for actual currently playing track!");
                            todo!();
                        }
                    }
                    Some(PlayableItem::Episode(_)) => {
                        log::error!("Actual current playing is episode");
                        todo!()
                    }
                    None => {
                        log::error!("Actual current playing item is none");
                        todo!();
                    }
                },
                None => {
                    start_playback(&spotify, expected_playing_id).await?;
                }
            }
        }
        None => {
            log::info!("No current track"); // TODO: check queue?
        }
    }

    Ok(None)
}
