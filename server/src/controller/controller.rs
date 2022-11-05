use crate::controller::messages::{
    ClientActorMessage, Connect, Disconnect, Queue, Search, SearchComplete, WsMessage,
};
use crate::controller::messages::{Response, SearchResultPayload};
use crate::db::Database;
use crate::session_agent::SessionAgentRequest;
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix::AsyncContext;
use actix_web::web;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Controller {
    clients: HashMap<Uuid, Socket>,
    sessions: HashMap<Uuid, HashSet<Uuid>>,
    db: web::Data<Database>,
    agent_tx: UnboundedSender<SessionAgentRequest>,
}

// TODO: handle all unwraps
// TODO: not sure how to handle async in handlers
// TODO: add logging for errors

impl Controller {
    pub fn new(db: web::Data<Database>, agent_tx: UnboundedSender<SessionAgentRequest>) -> Self {
        Self {
            clients: HashMap::new(),
            sessions: HashMap::new(),
            db,
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

const VERIFY_QUEUE_STATE_INTERVAL: Duration = Duration::from_secs(5);

// queue state poller:
// - iterate over sessions
// - if current playing track in spotify is not current_track_uri or non is playing
// -- play current_track_uri
// - if current playing track has less than 3s left in spotify:
// -- add first in queue to spotify queue, if successful remove from db queue, and replace current_track_uri with first one in queue

impl Actor for Controller {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(VERIFY_QUEUE_STATE_INTERVAL, |actor, ctx| {
            for (session_id, _) in actor.sessions.iter() {
                log::info!("verify queues state for {session_id}");
                // // actix_web::rt::spawn(async move {
                // //     log::info!("verify queues state for {id}");
                // // });
                // if let Ok(rt) = actix_web::rt::Runtime::new() {
                //     rt.block_on(async_fn(*session_id))
                // }
                // failes with: "thread 'main' panicked at 'Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks.', /home/apelsimon/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-1.21.2/src/runtime/scheduler/current_thread.rs:516:26
                // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
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
        let response = Response::SearchResult(SearchResultPayload {
            payload: msg.result,
        });
        self.send_message(response, &msg.connection_id);
    }
}

impl Handler<Queue> for Controller {
    type Result = ();

    fn handle(&mut self, msg: Queue, _: &mut Context<Self>) -> Self::Result {
        log::info!("queue track: {}", msg.track_id.to_string());
        let request = SessionAgentRequest::Queue(msg);
        if let Err(err) = self.agent_tx.send(request) {
            log::error!("Failed to send SessionAgentRequest::Queue, {err}");
        }
    }
}
