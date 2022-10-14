use super::utils::{e500, see_other};
use crate::session_state::{Context::Peer, TypedSession};
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn join(
    path: web::Path<Uuid>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    let (ok,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(e500)?;

    if !ok {
        log::error!("No session found with id {}", id);
        return Ok(see_other("/"));
    }

    log::info!("Found valid id {}", id);

    session.renew(id, Peer).map_err(e500)?;
    Ok(see_other("/session/"))
}
