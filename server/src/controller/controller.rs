use crate::controller::messages::{
    ClientActorMessage, Connect, Disconnect, State, Queue, Search, SearchComplete, StateUpdate, WsMessage,
};
use crate::controller::messages::{Response, SearchResultPayload};
use crate::session_agent::{SessionAgentRequest, UPDATE_STATE_INTERVAL};
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix::AsyncContext;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Controller {
    clients: HashMap<Uuid, Socket>,
    sessions: HashMap<Uuid, HashSet<Uuid>>,
    agent_tx: UnboundedSender<SessionAgentRequest>,
}

// TODO: handle all unwraps
// TODO: add logging for errors

impl Controller {
    pub fn new(agent_tx: UnboundedSender<SessionAgentRequest>) -> Self {
        Self {
            clients: HashMap::new(),
            sessions: HashMap::new(),
            agent_tx,
        }
    }
    fn send_message(&self, message: Response, id_to: &Uuid) {
        if let Some(socket_recipient) = self.clients.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message));
        } else {
            log::info!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Controller {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(UPDATE_STATE_INTERVAL, |actor, ctx| {
            for (session_id, _) in actor.sessions.iter() {
                let request = SessionAgentRequest::PollState((*session_id, ctx.address()));
                if let Err(err) = actor.agent_tx.send(request) {
                    log::error!("Failed to send SessionAgentRequest::PollState, {err}");
                }
            }
        });
    }
}

impl Handler<Disconnect> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.clients.remove(&msg.connection_id).is_some() {
            // self.sessions
            //     .get(&msg.session_id)
            //     .unwrap()
            //     .iter()
            //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
            //     .for_each(|user_id| {
            //         self.send_message(&format!("{} disconnected.", &msg.connection_id), user_id)
            //     });
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
        // self.sessions
        //     .get(&msg.session_id)
        //     .unwrap()
        //     .iter()
        //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
        //     .for_each(|conn_id| {
        //         self.send_message(&format!("{} just joined!", msg.connection_id), conn_id)
        //     });

        // store the address
        self.clients.insert(msg.connection_id, msg.client_addr);

        // send self your new uuid
        // self.send_message(
        //     &format!("your id is {}", msg.connection_id),
        //     &msg.connection_id,
        // );
    }
}

impl Handler<ClientActorMessage> for Controller {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        // self.sessions
        //     .get(&msg.session_id)
        //     .unwrap()
        //     .iter()
        //     .filter(|conn_id| *conn_id.to_owned() != msg.connection_id)
        //     .for_each(|client| self.send_message(&msg.msg, client));
    }
}

impl Handler<Search> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Search, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Search((msg, ctx.address()));
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Search, {err}");
        }
    }
}

impl Handler<SearchComplete> for Controller {
    type Result = ();

    fn handle(&mut self, msg: SearchComplete, _ctx: &mut Context<Self>) -> Self::Result {
        let response = Response::SearchResult(msg.result);
        self.send_message(response, &msg.connection_id);
    }
}

impl Handler<Queue> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Queue, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Queue((msg, ctx.address()));
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Queue, {err}");
        }
    }
}

impl Handler<State> for Controller {
    type Result = ();

    fn handle(&mut self, msg: State, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::GetState((msg.session_id, ctx.address()));
            if let Err(err) = self.agent_tx.send(request) {
                log::error!("Failed to send SessionAgentRequest::GetState, {err}");
            }
    }
}

impl Handler<StateUpdate> for Controller {
    type Result = ();

    fn handle(&mut self, msg: StateUpdate, _: &mut Context<Self>) -> Self::Result {
        let response = Response::StateUpdate(msg.update);
        self.sessions
            .get(&msg.session_id)
            .unwrap()
            .iter()
            .for_each(|client| self.send_message(response.clone(), client)); // TODO: clone needed?
    }
}
