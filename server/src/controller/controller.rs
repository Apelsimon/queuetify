use crate::controller::messages::{ClientActorMessage, Connect, Disconnect, WsMessage, Queue, Search,
    SearchComplete};
use actix::{AsyncContext};
use actix::prelude::{Actor, Context, Handler, Recipient};
use serde::__private::de;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use actix_web::{ web};
use crate::db::Database;
use crate::spotify::get_spotify_from_db;
use rspotify::AuthCodeSpotify;
use serde::{Deserialize, Serialize};
use rspotify::model::{SearchType, SearchResult::Tracks};
use rspotify::model::enums::misc::Market;
use rspotify::clients::{BaseClient, OAuthClient};
use rspotify::model::{SimplifiedArtist, PlayableId};
use rspotify::ClientError;
use crate::controller::messages::{SearchResultPayload, Response};
use rspotify::model::enums::types::DeviceType;
use rspotify::model::device::Device;

type Socket = Recipient<WsMessage>;

pub struct Controller {
    clients: HashMap<Uuid, Socket>,
    sessions: HashMap<Uuid, HashSet<Uuid>>,
    db: web::Data<Database>
}

// TODO: handle all unwraps
// TODO: not sure how to handle async in handlers
// TODO: add logging for errors

impl Controller {
    pub fn new(db: web::Data<Database>) -> Self {
        Self {
            clients: HashMap::new(),
            sessions: HashMap::new(),
            db
        }
    }
    fn send_message(&self, message: Response, id_to: &Uuid) {
        if let Some(socket_recipient) = self.clients.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message));
        } else {
            log::info!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Controller {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.clients.remove(&msg.connection_id).is_some() {
            // self.sessions
            //     .get(&msg.session_id)
            //     .unwrap()
            //     .iter()
            //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
            //     .for_each(|user_id| {
            //         self.send_message(&format!("{} disconnected.", &msg.connection_id), user_id)
            //     });
            if let Some(session) = self.sessions.get_mut(&msg.session_id) {
                if session.len() > 1 {
                    session.remove(&msg.connection_id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.sessions.remove(&msg.session_id);
                }
            }
        }
    }
}

impl Handler<Connect> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.sessions
            .entry(msg.session_id)
            .or_insert_with(HashSet::new)
            .insert(msg.connection_id);

        // send to everyone in the room that new uuid just joined
        // self.sessions
        //     .get(&msg.session_id)
        //     .unwrap()
        //     .iter()
        //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
        //     .for_each(|conn_id| {
        //         self.send_message(&format!("{} just joined!", msg.connection_id), conn_id)
        //     });

        // store the address
        self.clients.insert(msg.connection_id, msg.client_addr);

        // send self your new uuid
        // self.send_message(
        //     &format!("your id is {}", msg.connection_id),
        //     &msg.connection_id,
        // );
    }
}

impl Handler<ClientActorMessage> for Controller {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        // self.sessions
        //     .get(&msg.session_id)
        //     .unwrap()
        //     .iter()
        //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
        //     .for_each(|client| self.send_message(&msg.msg, client));
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct TrackInfo {
    name: String,
    artists: Vec<String>,
    id: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResult {
    tracks: Vec<TrackInfo>
}

fn build_artist_string_vec(artists: &Vec<SimplifiedArtist>) -> Vec<String> {
    let mut artist_string_vec = Vec::new();

    for artist in artists.iter() {
        artist_string_vec.push(artist.name.clone());
    }

    artist_string_vec
}

async fn get_search_results(spotify: &AuthCodeSpotify, input: &str) -> Result<SearchResult, ClientError> {
    let mut search_result = SearchResult{
        tracks: Vec::new()
    };

    if let Tracks(track_pages) = spotify.search(
        &input, &SearchType::Track, Some(&Market::FromToken), None, Some(10), None)
        .await? {
        for item in track_pages.items {
            let track_id = match item.id {
                Some(tid) => tid,
                None => continue,
            };
            
            let track = TrackInfo {
                name: item.name,
                artists: build_artist_string_vec(&item.artists),
                id: track_id.to_string()
            };
            search_result.tracks.push(track);
        }
    }    

    Ok(search_result)
}

async fn search(msg: &Search, db: &Database) -> Result<SearchResult, ()> {
    if let Ok(spotify) =  get_spotify_from_db(msg.session_id, &db).await {
        if let Ok(search_result) =  get_search_results(&spotify, &msg.query).await{
            return Ok(search_result);
        }
    }

    Err(())
}

impl Handler<Search> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Search, ctx: &mut Context<Self>) -> Self::Result {
        let db = self.db.clone();
        let addr = ctx.address();
        actix_web::rt::spawn(async move {
            if let Ok(search_result) = search(&msg, &db).await {
                addr.do_send(SearchComplete{
                    result: search_result,
                    connection_id: msg.connection_id
                });
            }
        });
    }
}

impl Handler<SearchComplete> for Controller {
    type Result = ();

    fn handle(&mut self, msg: SearchComplete, _ctx: &mut Context<Self>) -> Self::Result {
        let response = Response::SearchResult(
            SearchResultPayload{
                payload: msg.result
            }
        );
        self.send_message(response, &msg.connection_id);
    }
}

async fn get_device(spotify: &AuthCodeSpotify) -> Result<Device, ()> {
     match spotify.device().await {
         Ok(devices) => {
             for device in devices.into_iter() {
                 if device._type == DeviceType::Computer {
                    return Ok(device);
                 }
             }
         },
         Err(err) => { log::error!("Failed to fetch devices {err}"); }
     }

    Err(())
}

// 1. om först i queuen, start playing track (add to db queue)
// 2. om inte, lägg till i db queue
// 3. behöver separat tråd som pollar spotify current playing track och ser om det stämmer med db queue
impl Handler<Queue> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Queue, _: &mut Context<Self>) -> Self::Result {
        log::info!("queue track: {}", msg.track_id.to_string());
        let db = self.db.clone();
        actix_web::rt::spawn(async move {
            if let Ok(spotify) =  get_spotify_from_db(msg.session_id, &db).await {
                let session = match db.get_session(msg.session_id).await {
                    Ok(session) => session,
                    Err(err) => { log::error!("no session found, {err}"); return; }
                };
                // TODO: make device selectable when session is created/started
                let device = match get_device(&spotify).await {
                    Ok(device) => device,
                    Err(()) => { log::error!("Failed to fetch device"); return; }
                };

                if let Ok((exists, transaction)) = db.has_current_track(session.queue_id).await {
                    if !exists {
                        log::info!("No current track exists, start playback of {}", msg.track_id);
                        let uri: Box<dyn PlayableId> = Box::new(msg.track_id.clone());
                        match spotify.start_uris_playback(Some(uri.as_ref()), Some(device.id.as_deref().unwrap_or("")), None, None).await {
                            Ok(_) => { log::info!("Playback of uri started!")},
                            Err(err) => { log::error!("Playback start error {err}"); return; }
                        }
                        
                        let _ = db.set_current_track(transaction, session.queue_id, msg.track_id).await;
                    } else {
                        log::info!("Current track exists, queue track {}", msg.track_id)
                    }
                } else {
                    log::error!("Error checking for current track");
                }
                // match spotify.add_item_to_queue(&msg.track_id, None).await {
                //     Ok(_) => log::info!("item succesfully queued!"),
                //     Err(err) => log::error!("Failed to queue track: {err}")
                // }
            }
        });
    }
}