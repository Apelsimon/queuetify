use crate::controller::messages::{
    Connect, Devices, DevicesComplete, DevicesPayload, Disconnect, Kill, 
    KillComplete, Queue, Refresh, Search, SearchComplete, State, StateUpdate, 
    Transfer, TransferComplete, TransferResponsePayload, Vote, WsMessage,
};
use crate::controller::messages::{Response};
use crate::session_agent::{SessionAgentRequest};
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix::AsyncContext;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;
use std::time::Duration;

type Socket = Recipient<WsMessage>;

pub struct Controller {
    clients: HashMap<Uuid, Socket>,
    sessions: HashMap<Uuid, HashSet<Uuid>>,
    agent_tx: UnboundedSender<SessionAgentRequest>,
}

// TODO: handle all unwraps
// TODO: add logging for errors
// TODO: reuse msg when sending to agent_tx
// TODO: validate session id

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

pub const REFRESH_TOKEN_INTERVAL: Duration = Duration::from_secs(3600);
pub const POLL_STATE_INTERVAL: Duration = Duration::from_secs(5);

impl Actor for Controller {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(POLL_STATE_INTERVAL, |actor, ctx| {
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

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.sessions
            .entry(msg.session_id)
            .or_insert_with(HashSet::new)
            .insert(msg.connection_id);

        // store the address
        self.clients.insert(msg.connection_id, msg.client_addr);

        ctx.address().do_send(Refresh{
            duration: Duration::from_secs(1)/*REFRESH_TOKEN_INTERVAL*/,
            session_id: msg.session_id
        });
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
        let request = SessionAgentRequest::GetState((msg.session_id, Some(msg.connection_id), ctx.address()));
            if let Err(err) = self.agent_tx.send(request) {
                log::error!("Failed to send SessionAgentRequest::GetState, {err}");
            }
    }
}

impl Handler<Vote> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Vote, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Vote((msg, ctx.address()));
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
            .filter(|connection_id| {
                if let Some(id) = msg.connection_id {
                    return id == *connection_id.to_owned();
                }

                return true;
            })
            .for_each(|client| self.send_message(response.clone(), client)); // TODO: clone needed?
    }
}

impl Handler<Refresh> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Refresh, ctx: &mut Context<Self>) -> Self::Result {
        ctx.run_later(msg.duration, move |actor, ctx| {
            let request = SessionAgentRequest::Refresh((msg.session_id, ctx.address()));
            if let Err(err) = actor.agent_tx.send(request) {
                log::error!("Failed to send SessionAgentRequest::Refresh, {err}");
            }
        });        
    }
}

impl Handler<Kill> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Kill, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Kill((msg.session_id, ctx.address()));
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Kill, {err}");
        }
    }
}

impl Handler<KillComplete> for Controller {
    type Result = ();

    fn handle(&mut self, msg: KillComplete, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(session) = self.sessions.get(&msg.session_id) {
            let shutdown = Response::Shutdown;
            session.iter().for_each(|client| { 
                self.send_message(shutdown.clone(), client);
            });
        }
    }
}

impl Handler<Devices> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Devices, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Devices((msg, ctx.address()));
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Devices, {err}");
        }
    }
}

impl Handler<DevicesComplete> for Controller {
    type Result = ();

    fn handle(&mut self, msg: DevicesComplete, ctx: &mut Context<Self>) -> Self::Result {
        let response = Response::Devices(DevicesPayload {
            payload: msg.devices
        });
        self.send_message(response, &msg.connection_id)
    }
}

impl Handler<Transfer> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Transfer, ctx: &mut Context<Self>) -> Self::Result {
        let request = SessionAgentRequest::Transfer((msg, ctx.address()));
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Transfer, {err}");
        }
    }
}

impl Handler<TransferComplete> for Controller {
    type Result = ();

    fn handle(&mut self, msg: TransferComplete, ctx: &mut Context<Self>) -> Self::Result {
        let response = Response::Transfer(TransferResponsePayload {
            payload: msg.result
        });
        self.send_message(response, &msg.connection_id)
    }
}