use crate::session_state::TypedSession;
use crate::{controller::controller::Controller, controller::ws_connection::WsConnection};
use actix::Addr;
use actix_web::{web, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub async fn ws_connect(
    req: HttpRequest,
    stream: Payload,
    session: TypedSession,
    controller: web::Data<Addr<Controller>>,
) -> Result<HttpResponse, Error> {
    // TODO: Ok to assume id exists here because of protected route?
    let session_id = session.get_id().unwrap().unwrap();
    let ws = WsConnection::new(session_id, controller.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
