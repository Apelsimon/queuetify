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

#[derive(Deserialize)]
pub struct SearchQuery {
    input: String
}

#[derive(Serialize)]
struct TrackInfo {
    name: String,
    artists: Vec<String>,
    id: String
}

#[derive(Serialize)]
struct SearchResult {
    tracks: Vec<TrackInfo>
}

pub async fn search(
    typed_session: TypedSession,
    db: web::Data<Database>,
    query: web::Query<SearchQuery>
) -> Result<impl Responder, actix_web::Error> {
    let SearchQuery{input} = query.into_inner();
    log::info!("Search for: {}", input);

    let spotify =  get_spotify_from_db(&typed_session, &db).await.map_err(e500)?;
    let search_result = get_search_results(&spotify, &input)
        .await.map_err(e500)?; 

    Ok(web::Json(search_result))
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

#[derive(serde::Deserialize)]
pub struct QueueRequest {
    uri: String,
}

pub async fn queue(typed_session: TypedSession,
    db: web::Data<Database>,
    queue_request: web::Json<QueueRequest>) -> Result<HttpResponse, actix_web::Error> {
    let queue_request = queue_request.into_inner();

    let track_id = TrackId::from_str(&queue_request.uri).map_err(e500)?;
    let spotify =  get_spotify_from_db(&typed_session, &db).await.map_err(e500)?;

    spotify.add_item_to_queue(&track_id, None).await.map_err(e500)?;
    Ok(HttpResponse::Ok().finish())
}