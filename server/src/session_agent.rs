use crate::configuration::Settings;
use crate::controller;
use crate::controller::messages::{
    DeviceInfo, DevicesComplete, KillComplete, SearchComplete, SearchResultPayload, StateUpdate,
    StateUpdatePayload, TransferComplete, VotedTracksComplete,
};
use crate::controller::{Controller, POLL_STATE_INTERVAL, REFRESH_TOKEN_INTERVAL};
use crate::db::Database;
use actix::Addr;
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::enums::misc::Market;
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

// TODO: kill session if db operations fail?

pub enum SessionAgentRequest {
    Search((controller::Search, Addr<Controller>)),
    Queue((controller::Queue, Addr<Controller>)),
    GetState((Uuid, Option<Uuid>, Addr<Controller>)),
    PollState((Uuid, Addr<Controller>)),
    Vote((controller::Vote, Addr<Controller>)),
    Refresh((Uuid, Addr<Controller>)),
    Kill((Uuid, Addr<Controller>)),
    Devices((controller::Devices, Addr<Controller>)),
    Transfer((controller::Transfer, Addr<Controller>)),
    VotedTracks((controller::VotedTracks, Addr<Controller>)),
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
            db: Database::new(&settings.database, settings.spotify),
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
                SessionAgentRequest::GetState((id, connection_id, addr)) => {
                    let spotify = match self.db.get_spotify(id).await {
                        Ok(spotify) => spotify,
                        Err(_) => continue,
                    };
                    match get_current_state(id, connection_id, &spotify, &self.db).await {
                        Ok(update) => {
                            addr.do_send(update);
                        }
                        Err(err) => {
                            log::error!("Failed to get current state {err}");
                        }
                    }
                }
                SessionAgentRequest::PollState((id, addr)) => {
                    match on_poll_state(id, &self.db).await {
                        Ok(update) => {
                            if let Some(update) = update {
                                addr.do_send(update);
                            }
                        }
                        Err(err) => {
                            log::error!("Error on poll state {err}");
                        }
                    }
                }
                SessionAgentRequest::Vote((msg, addr)) => match on_vote(msg, &self.db).await {
                    Ok(update) => {
                        if let Some(update) = update {
                            addr.do_send(update);
                        }
                    }
                    Err(err) => {
                        log::error!("Error on vote {err}");
                    }
                },
                SessionAgentRequest::Refresh((id, addr)) => {
                    match on_refresh(id, &self.db).await {
                        Ok(()) => addr.do_send(controller::Refresh {
                            duration: REFRESH_TOKEN_INTERVAL,
                            session_id: id,
                        }),
                        Err(_) => {
                            addr.do_send(controller::Refresh {
                                duration: Duration::from_secs(10), //TODO: exponential backoff wait? kill session after some number of tries?
                                session_id: id,
                            })
                        }
                    }
                }
                SessionAgentRequest::Kill((id, addr)) => match self.db.delete_session(id).await {
                    Ok(()) => addr.do_send(KillComplete { session_id: id }),
                    Err(err) => {
                        log::error!("Error on kill {err}");
                    }
                },
                // TODO: make endpoint of this instead
                SessionAgentRequest::Devices((msg, addr)) => {
                    let connection_id = msg.connection_id;
                    match on_devices(msg, &self.db).await {
                        Ok(devices) => addr.do_send(DevicesComplete {
                            connection_id,
                            devices,
                        }),
                        Err(err) => {
                            log::error!("Error on devices {err}")
                        }
                    }
                }
                SessionAgentRequest::Transfer((msg, addr)) => {
                    let connection_id = msg.connection_id;
                    match on_transfer(msg, &self.db).await {
                        Ok(()) => addr.do_send(TransferComplete {
                            connection_id,
                            result: "OK".to_string(),
                        }),
                        Err(_) => addr.do_send(TransferComplete {
                            connection_id,
                            result: "Err".to_string(),
                        }),
                    }
                }
                SessionAgentRequest::VotedTracks((msg, addr)) => {
                    match on_voted_tracks(msg.clone(), &self.db).await {
                        Ok(tracks) => addr.do_send(VotedTracksComplete {
                            connection_id: msg.connection_id,
                            tracks,
                        }),
                        Err(err) => {
                            log::error!("Error on devices {err}");
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
    track: Option<TrackInfo>,
    queue: Vec<TrackInfo>,
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
    if let Ok(spotify) = db.get_spotify(msg.session_id).await {
        if let Ok(search_result) = get_search_results(&spotify, &msg.query).await {
            return Ok(search_result);
        }
    }

    Err(())
}

async fn start_playback(spotify: &AuthCodeSpotify, id: TrackId) -> Result<(), anyhow::Error> {
    let uri: Box<dyn PlayableId> = Box::new(id);
    spotify
        .start_uris_playback(Some(uri.as_ref()), None, None, None)
        .await?;
    Ok(())
}

async fn get_current_state(
    id: Uuid,
    connection_id: Option<Uuid>,
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
        track: current_track,
        queue: current_queue,
    };
    Ok(StateUpdate {
        update: StateUpdatePayload { payload },
        session_id: id,
        connection_id,
    })
}

async fn on_queue(msg: controller::Queue, db: &Database) -> Result<StateUpdate, anyhow::Error> {
    let spotify = db.get_spotify(msg.session_id).await?;

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

    let state = get_current_state(msg.session_id, None, &spotify, &db).await?;
    Ok(state)
}

async fn on_poll_state(id: Uuid, db: &Database) -> Result<Option<StateUpdate>, anyhow::Error> {
    let spotify = db.get_spotify(id).await?;
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
                                            if (track.duration - duration) < POLL_STATE_INTERVAL {
                                                match db
                                                    .pop_track_from_queue(id, &mut transaction)
                                                    .await?
                                                {
                                                    Some(new_track) => {
                                                        db.remove_votes(
                                                            &mut transaction,
                                                            id,
                                                            new_track.clone(),
                                                        )
                                                        .await?;
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

                                                let state =
                                                    get_current_state(id, None, &spotify, &db)
                                                        .await?;
                                                return Ok(Some(state));
                                            }
                                        }
                                        None => {
                                            log::error!(
                                                "Progress missing for current playing context!"
                                            );
                                            // TODO
                                        }
                                    }
                                } else {
                                    spotify.resume_playback(None, None).await?;
                                }
                            }
                        } else {
                            log::error!("Track id missing for actual currently playing track!");
                            // TODO
                        }
                    }
                    Some(PlayableItem::Episode(_)) => {
                        log::error!("Actual current playing is episode");
                        // TODO
                    }
                    None => {
                        log::error!("Actual current playing item is none");
                        // TODO
                        let _ = start_playback(&spotify, expected_playing_id).await?;
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

async fn on_vote(
    msg: controller::Vote,
    db: &Database,
) -> Result<Option<StateUpdate>, anyhow::Error> {
    match db.add_vote(&msg).await {
        Ok(()) => {
            let spotify = db.get_spotify(msg.session_id).await?;
            let state = get_current_state(msg.session_id, None, &spotify, &db).await?;
            Ok(Some(state))
        }
        Err(_) => Ok(None),
    }
}

async fn on_refresh(id: Uuid, db: &Database) -> Result<(), anyhow::Error> {
    let spotify = db.get_spotify(id).await?;
    spotify.refresh_token().await?;
    db.set_spotify(id, &spotify).await?;
    Ok(())
}

async fn on_devices(
    msg: controller::Devices,
    db: &Database,
) -> Result<Vec<DeviceInfo>, anyhow::Error> {
    let spotify = db.get_spotify(msg.session_id).await?;
    let devices = spotify.device().await?;

    let mut device_infos = Vec::new();

    for device in devices.iter() {
        let dev_info = match DeviceInfo::try_from(device.clone()) {
            Ok(info) => info,
            Err(_) => continue,
        };

        device_infos.push(dev_info);
    }

    Ok(device_infos)
}

async fn on_transfer(msg: controller::Transfer, db: &Database) -> Result<(), anyhow::Error> {
    let spotify = db.get_spotify(msg.session_id).await?;
    spotify
        .transfer_playback(&msg.device_id, Some(false))
        .await?;
    Ok(())
}

async fn on_voted_tracks(
    msg: controller::VotedTracks,
    db: &Database,
) -> Result<Vec<String>, anyhow::Error> {
    let voted_tracks = db.voted_tracks(msg.session_id, msg.connection_id).await?;
    Ok(voted_tracks)
}
