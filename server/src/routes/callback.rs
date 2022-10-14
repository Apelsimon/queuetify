use crate::routes::utils::e500;
use crate::routes::utils::{get_spotify, see_other};
use crate::session_state::{Context::Host, TypedSession};
use actix_web::{web, HttpResponse};
use rspotify::clients::BaseClient;
use rspotify::clients::OAuthClient;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(
    query: web::Query<CallbackQuery>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let CallbackQuery {
        code,
        state: _state,
    } = query.into_inner();
    let mut spotify = get_spotify();

    if let Err(err) = spotify.request_token(&code).await {
        log::error!("Failed to get user token {:?}", err);
        return Ok(see_other("/"));
    }

    let session_id = Uuid::new_v4();
    let token = spotify.get_token().lock().await.unwrap().clone();
    let token = serde_json::to_string(&token)?;
    let queue_id = Uuid::new_v4();

    sqlx::query!(
        r#"
            INSERT INTO sessions (
                id, token, queue_id, created_at
            )
            VALUES ($1, $2, $3, now())
        "#,
        session_id,
        token,
        queue_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(e500)?;

    session.renew(session_id, Host).map_err(e500)?;
    Ok(see_other("/session/"))
}
