use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use crate::db::Database;

pub enum SessionAgentRequest {}
pub struct SessionAgent {
    rx: Receiver<SessionAgentRequest>,
    db: Database
}

impl SessionAgent {
    pub fn new(db: Database) -> (Self, Sender<SessionAgentRequest>) {
        let (tx, rx) = mpsc::channel();
        let agent = Self { 
            rx,
            db
        };
        (agent, tx)
    }

    pub fn run(self) -> Result<(), RecvError> {
        loop {
            let request = self.rx.recv()?;
        }
    }
}
