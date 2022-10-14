use crate::session_state::TypedSession;
use crate::{controller::controller::Controller, ws_connection::WsConnection};
use actix::Addr;
use actix_web::{web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub async fn ws_connect(
    req: HttpRequest,
    stream: Payload,
    session: TypedSession,
    controller: Data<Addr<Controller>>,
) -> Result<HttpResponse, Error> {
    let session_id = session.get_id().unwrap().unwrap(); // TODO: handle error
    let ws = WsConnection::new(session_id, controller.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
