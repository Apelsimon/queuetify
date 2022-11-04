use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::mpsc::error::TryRecvError;
use crate::db::Database;
use crate::controller;
use crate::controller::Controller;
use actix::Addr;
use std::time::Duration;

pub enum SessionAgentRequest {
    Search((controller::Search, Addr<Controller>))
}

pub struct SessionAgent {
    rx: UnboundedReceiver<SessionAgentRequest>,
    db: Database
}

impl SessionAgent {
    pub fn new(db: Database) -> (Self, UnboundedSender<SessionAgentRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let agent = Self { 
            rx,
            db
        };
        (agent, tx)
    }

    pub async fn run(mut self) {
        loop {
            let request = match self.rx.recv().await {
                Some(request) => request,
                None => return, 
            };

            match request {
                SessionAgentRequest::Search((msg, addr)) => {
                    log::info!("Received search req {}", msg.query);
            }
        }
        }
    }
}