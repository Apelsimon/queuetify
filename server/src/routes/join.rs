use super::utils::{e500, see_other};
use crate::session_state::{Context::Peer, TypedSession};
use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::db::Database;

pub async fn join(
    path: web::Path<Uuid>,
    session: TypedSession,
    db: web::Data<Database>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    match db.session_exists(id).await {
        Ok(false) | Err(_) => {
            log::error!("No session found with id {}", id);
            return Ok(see_other("/"));
        },
        _ => {}
    }

    log::info!("Found valid id {}", id);

    session.renew(id, Peer).map_err(e500)?;
    Ok(see_other("/session/"))
}
