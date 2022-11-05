use crate::configuration::Settings;
use crate::controller;
use crate::controller::messages::SearchComplete;
use crate::controller::Controller;
use crate::db::Database;
use crate::spotify::get_spotify_from_db;
use actix::Addr;
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::device::Device;
use rspotify::model::enums::misc::Market;
use rspotify::model::enums::types::DeviceType;
use rspotify::model::TrackId;
use rspotify::model::{AdditionalType, PlayableId, PlayableItem, SimplifiedArtist};
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
    Queue(controller::Queue),
    UpdateState(Uuid),
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
                        addr.do_send(SearchComplete {
                            result: search_result,
                            connection_id: msg.connection_id,
                        });
                    }
                }
                SessionAgentRequest::Queue(msg) => {
                    if let Err(err) = on_queue(msg, &self.db).await {
                        log::error!("Error on queue {err}");
                    }
                }
                SessionAgentRequest::UpdateState(id) => {
                    if let Err(err) = on_update_state(id, &self.db).await {
                        log::error!("Error on update state {err}");
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct TrackInfo {
    name: String,
    artists: Vec<String>,
    id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResult {
    tracks: Vec<TrackInfo>,
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
            let track_id = match item.id {
                Some(tid) => tid,
                None => continue,
            };

            let track = TrackInfo {
                name: item.name,
                artists: build_artist_string_vec(&item.artists),
                id: track_id.to_string(),
            };
            search_result.tracks.push(track);
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

async fn on_queue(msg: controller::Queue, db: &Database) -> Result<(), anyhow::Error> {
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

    Ok(())
}

pub const UPDATE_STATE_INTERVAL: Duration = Duration::from_secs(5);

async fn on_update_state(id: Uuid, db: &Database) -> Result<(), anyhow::Error> {
    log::info!("update state for {id}");
    let spotify = get_spotify_from_db(id, &db).await?;
    let (track, mut transaction) = db.get_current_track(id).await?;
    match track {
        Some(expected_playing_id) => {
            match spotify.current_playing(None, None::<Vec<&AdditionalType>>).await? {
                Some(current_playing_context) => match current_playing_context.item {
                    Some(PlayableItem::Track(track)) => {
                        if let Some(actual_playing_id) = track.id {
                            if actual_playing_id != expected_playing_id {
                                let _ = start_playback(&spotify, expected_playing_id).await?;
                            } else {
                                if current_playing_context.is_playing {
                                    match current_playing_context.progress {
                                        Some(duration) => {
                                            if (track.duration - duration) < UPDATE_STATE_INTERVAL
                                            {
                                                match db
                                                    .pop_track_from_queue(id, &mut transaction)
                                                    .await?
                                                {
                                                    Some(new_track) => {
                                                        db
                                                            .set_current_track(
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
                                            }
                                        }
                                        None => {
                                            log::error!("Progress missing for current playing context!");
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

    Ok(())
}
