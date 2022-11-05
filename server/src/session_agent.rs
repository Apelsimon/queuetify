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
use rspotify::model::{PlayableId, SimplifiedArtist};
use rspotify::model::{SearchResult::Tracks, SearchType};
use rspotify::AuthCodeSpotify;
use rspotify::ClientError;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub enum SessionAgentRequest {
    Search((controller::Search, Addr<Controller>)),
    Queue(controller::Queue),
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
                    if let Ok(search_result) = search(&msg, &self.db).await {
                        addr.do_send(SearchComplete {
                            result: search_result,
                            connection_id: msg.connection_id,
                        });
                    }
                }
                SessionAgentRequest::Queue(msg) => {
                    on_queue(msg, &self.db).await;
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

async fn search(msg: &controller::Search, db: &Database) -> Result<SearchResult, ()> {
    if let Ok(spotify) = get_spotify_from_db(msg.session_id, &db).await {
        if let Ok(search_result) = get_search_results(&spotify, &msg.query).await {
            return Ok(search_result);
        }
    }

    Err(())
}

async fn get_device(spotify: &AuthCodeSpotify) -> Result<Device, ()> {
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

    Err(())
}

// 1. om först i queuen, start playing track (add to db queue)
// 2. om inte, lägg till i db queue
// 3. behöver separat tråd som pollar spotify current playing track och ser om det stämmer med db queue
async fn on_queue(msg: controller::Queue, db: &Database) {
    if let Ok(spotify) = get_spotify_from_db(msg.session_id, &db).await {
        let session = match db.get_session(msg.session_id).await {
            Ok(session) => session,
            Err(err) => {
                log::error!("no session found, {err}");
                return;
            }
        };
        // TODO: make device selectable when session is created/started
        let device = match get_device(&spotify).await {
            Ok(device) => device,
            Err(()) => {
                log::error!("Failed to fetch device");
                return;
            }
        };

        if let Ok((exists, transaction)) = db.has_current_track(msg.session_id).await {
            if !exists {
                log::info!(
                    "No current track exists, start playback of {}",
                    msg.track_id
                );
                let uri: Box<dyn PlayableId> = Box::new(msg.track_id.clone());
                if let Err(err) = spotify
                    .start_uris_playback(
                        Some(uri.as_ref()),
                        Some(device.id.as_deref().unwrap_or("")),
                        None,
                        None,
                    )
                    .await
                {
                    log::error!("Playback start error {err}");
                    return;
                }

                let _ = db
                    .set_current_track(transaction, msg.session_id, msg.track_id)
                    .await;
            } else {
                log::info!("Current track exists, queue track {}", msg.track_id);
                if let Err(err) = db
                    .queue_track(transaction, msg.session_id, msg.track_id)
                    .await
                {
                    log::error!("Failed to queue track: {}", err);
                    return;
                }
            }
        } else {
            log::error!("Error checking for current track");
        }
    }
}
