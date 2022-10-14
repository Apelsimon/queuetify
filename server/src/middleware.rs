use crate::routes::utils::see_other;
use crate::session_state::TypedSession;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::FromRequest;
use actix_web_lab::middleware::Next;

pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(http_request, payload).await
    }?;

    if let Ok(Some(_)) = session.get_id() {
        log::info!("Valid session!");
        return next.call(req).await;
    }

    log::error!("Invalid session!");
    let response = see_other("/");
    let e = anyhow::anyhow!("The user has no associated session");
    Err(InternalError::from_response(e, response).into())
}
