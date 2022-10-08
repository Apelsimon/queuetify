use crate::session_state::TypedSession;
use actix_web::HttpResponse;
use uuid::Uuid;

use super::utils::see_other;

pub async fn join(session: TypedSession) -> HttpResponse {
    // TODO: authenticate in order to join session

    session.renew();
    session.insert_user_id(Uuid::new_v4()).ok(); // TODO: handle error
    session.insert_context("peer".to_string()).ok(); // TODO: handle error and make context enum
    see_other("/")
}
