use crate::controller::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Controller {
    clients: HashMap<Uuid, Socket>,
    sessions: HashMap<Uuid, HashSet<Uuid>>,
}

// TODO: handle all unwraps

impl Controller {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.clients.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            log::info!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Default for Controller {
    fn default() -> Controller {
        Controller {
            clients: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

impl Actor for Controller {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.clients.remove(&msg.connection_id).is_some() {
            self.sessions
                .get(&msg.session_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
                .for_each(|user_id| {
                    self.send_message(&format!("{} disconnected.", &msg.connection_id), user_id)
                });
            if let Some(session) = self.sessions.get_mut(&msg.session_id) {
                if session.len() > 1 {
                    session.remove(&msg.connection_id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.sessions.remove(&msg.session_id);
                }
            }
        }
    }
}

impl Handler<Connect> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.sessions
            .entry(msg.session_id)
            .or_insert_with(HashSet::new)
            .insert(msg.connection_id);

        // send to everyone in the room that new uuid just joined
        self.sessions
            .get(&msg.session_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
            .for_each(|conn_id| {
                self.send_message(&format!("{} just joined!", msg.connection_id), conn_id)
            });

        // store the address
        self.clients.insert(msg.connection_id, msg.client_addr);

        // send self your new uuid
        self.send_message(
            &format!("your id is {}", msg.connection_id),
            &msg.connection_id,
        );
    }
}

impl Handler<ClientActorMessage> for Controller {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        self.sessions
            .get(&msg.session_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
            .for_each(|client| self.send_message(&msg.msg, client));
    }
}
