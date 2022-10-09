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

    let _ = sqlx::query!(
        r#"
            SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)
        "#,
        id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(e500)?;

    session.renew();
    session.insert_id(id).ok();
    session.insert_user_id(Uuid::new_v4()).ok(); // TODO: handle error
    session.insert_context(Peer).ok();
    Ok(see_other("/session"))
}
