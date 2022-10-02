use crate::routes::utils::get_spotify;
use actix_web::HttpResponse;
use rspotify::clients::OAuthClient;

use super::utils::see_other;

pub async fn session() -> HttpResponse {
    // let spotify = get_spotify(); // TODO: cant use new spotify instance
    // match spotify.me().await {
    //     Ok(user_info) => {
    //         log::info!("Display name of current host: {}", 
    //             user_info.display_name.unwrap_or_else(|| String::from("No displayname found!")));
    //     }
    //     Err(err) => {
    //         log::error!("Failed for {err}!");
    //     }
    // }
    log::info!("Start new session!");
    HttpResponse::Ok().finish()
}

pub async fn join() -> HttpResponse {
    // TODO: authenticate in order to join session
    see_other("/session")
}