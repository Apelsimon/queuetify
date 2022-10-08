use crate::routes::utils::{get_spotify, see_other};
use crate::session_state::TypedSession;
use actix_web::{web, HttpResponse};
use rspotify::clients::OAuthClient;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(query: web::Query<CallbackQuery>, session: TypedSession) -> HttpResponse {
    let CallbackQuery {
        code,
        state: _state,
    } = query.into_inner();
    let mut spotify = get_spotify();

    if let Err(err) = spotify.request_token(&code).await {
        log::error!("Failed to get user token {:?}", err);
        return see_other("/");
    }

    session.renew();
    session.insert_user_id(Uuid::new_v4()).ok(); // TODO: handle error
    session.insert_context("host".to_string()).ok(); // TODO: handle error and make context enum
    see_other("/session")
}
