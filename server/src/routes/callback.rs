use crate::configuration::SpotifySettings;
use crate::routes::utils::e500;
use crate::routes::utils::see_other;
use crate::session_state::{Context::Host, TypedSession};
use crate::spotify::{get_default_spotify, get_token_string};
use actix_web::{web, HttpResponse};
use rspotify::clients::OAuthClient;
use serde::Deserialize;
use uuid::Uuid;
use crate::db::Database;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(
    query: web::Query<CallbackQuery>,
    session: TypedSession,
    db: web::Data<Database>,
    settings: web::Data<SpotifySettings>,
) -> Result<HttpResponse, actix_web::Error> {
    let CallbackQuery {
        code,
        state: _state,
    } = query.into_inner();
    let mut spotify = get_default_spotify(&settings);

    if let Err(err) = spotify.request_token(&code).await {
        log::error!("Failed to get user token {:?}", err);
        return Ok(see_other("/"));
    }

    let session_id = Uuid::new_v4();
    let token = get_token_string(&spotify).await?;
    let queue_id = Uuid::new_v4();

    db.insert_session(session_id, &token, queue_id).await
        .map_err(e500)?;

    session.renew(session_id, Host).map_err(e500)?;
    Ok(see_other("/session/"))
}
