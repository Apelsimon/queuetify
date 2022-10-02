use crate::routes::utils::{get_spotify, see_other};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use rspotify::clients::OAuthClient;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String
}

pub async fn callback(query: web::Query<CallbackQuery>) -> HttpResponse {
    let mut spotify = get_spotify();
    // let res = spotify.request_token(&query.code);
    match spotify.request_token(&query.code).await {
        Ok(_) => {
            log::info!("Request user token successful");
            see_other("/session")
        }
        Err(err) => {
            log::error!("Failed to get user token {:?}", err);
            see_other("/")
        }
    }
}
