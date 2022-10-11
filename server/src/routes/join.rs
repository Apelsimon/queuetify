use crate::session_state::{Context::Peer, TypedSession};
use actix_web::{web, HttpResponse};
use uuid::Uuid;
use sqlx::PgPool;
use super::utils::{e500, see_other};

pub async fn join(
    path: web::Path<Uuid>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let id = path.into_inner();

    let (ok,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)")
        .bind(id)
        .fetch_one(pool.get_ref()).await.map_err(e500)?;

    if !ok {
        log::error!("No session found with id {}", id);
        return Ok(see_other("/"));    
    }

    log::info!("Found valid id {}", id);
    // TODO: handle errors
    session.renew();
    session.insert_id(id).ok();
    session.insert_user_id(Uuid::new_v4()).ok(); 
    session.insert_context(Peer).ok();
    Ok(see_other("/session"))
}
