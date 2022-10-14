use crate::controller::controller::Controller;
use crate::controller::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::ActorFutureExt;
use actix::{fut, ActorContext};
use actix::{Actor, Addr, ContextFutureSpawner, Running, StreamHandler, WrapFuture};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConnection {
    session_id: Uuid,
    controller_addr: Addr<Controller>,
    last_heartbeat_timestamp: Instant,
    connection_id: Uuid,
}

impl WsConnection {
    pub fn new(session_id: Uuid, controller_addr: Addr<Controller>) -> Self {
        Self {
            session_id,
            controller_addr,
            last_heartbeat_timestamp: Instant::now(),
            connection_id: Uuid::new_v4(),
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat_timestamp) > CLIENT_TIMEOUT {
                log::info!("Disconnecting failed heartbeat");
                act.controller_addr.do_send(Disconnect {
                    session_id: act.session_id,
                    connection_id: act.connection_id,
                });
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        let addr = ctx.address();
        self.controller_addr
            .send(Connect {
                client_addr: addr.recipient(),
                session_id: self.session_id,
                connection_id: self.connection_id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.controller_addr.do_send(Disconnect {
            session_id: self.session_id,
            connection_id: self.connection_id,
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat_timestamp = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat_timestamp = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(Text(s)) => self.controller_addr.do_send(ClientActorMessage {
                session_id: self.session_id,
                connection_id: self.connection_id,
                msg: s.to_string(),
            }),
            Err(e) => panic!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
