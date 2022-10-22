use std::str::FromStr;

use actix_web::web;
use actix_web::Responder;
use rspotify::clients::OAuthClient;
use rspotify::model::SimplifiedArtist;
use rspotify::model::TrackId;
use crate::session_state::TypedSession;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use crate::spotify::get_spotify_from_db;
use crate::routes::utils::e500;
use rspotify::model::{SearchType, SearchResult::Tracks};
use rspotify::clients::BaseClient;
use rspotify::AuthCodeSpotify;
use rspotify::ClientError;
use actix_web::HttpResponse;
use rspotify::model::enums::misc::Market;

// TODO: have websocket api instead to enable async resonses?

#[derive(serde::Deserialize)]
pub struct QueueRequest {
    uri: String,
}

pub async fn queue(typed_session: TypedSession,
    db: web::Data<Database>,
    queue_request: web::Json<QueueRequest>) -> Result<HttpResponse, actix_web::Error> {
    let queue_request = queue_request.into_inner();
    // TODO: Ok to assume id exists here because of protected route?
    let id = typed_session.get_id().unwrap().unwrap();
    let track_id = TrackId::from_str(&queue_request.uri).map_err(e500)?;
    let spotify =  get_spotify_from_db(id, &db).await.map_err(e500)?;

    spotify.add_item_to_queue(&track_id, None).await.map_err(e500)?;
    Ok(HttpResponse::Ok().finish())
}