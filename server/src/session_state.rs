use actix_session::Session;
use actix_session::SessionExt;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::fmt;
use std::future::{ready, Ready};
use uuid::Uuid;
pub struct TypedSession(Session);

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Context {
    Host,
    Peer,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Host => {
                write!(f, "host")
            }
            Self::Peer => {
                write!(f, "peer")
            }
        }
    }
}

impl TypedSession {
    const ID_KEY: &'static str = "id";
    const USER_ID_KEY: &'static str = "user_id";
    const CONTEXT_ID_KEY: &'static str = "context_id";

    pub fn renew(&self) {
        self.0.renew();
    }
    pub fn insert_id(&self, id: Uuid) -> Result<(), serde_json::Error> {
        self.0.insert(Self::ID_KEY, id)
    }
    pub fn get_id(&self) -> Result<Option<Uuid>, serde_json::Error> {
        self.0.get(Self::ID_KEY)
    }
    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), serde_json::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }
    pub fn get_user_id(&self) -> Result<Option<Uuid>, serde_json::Error> {
        self.0.get(Self::USER_ID_KEY)
    }
    pub fn insert_context(&self, ctx: Context) -> Result<(), serde_json::Error> {
        self.0.insert(Self::CONTEXT_ID_KEY, ctx)
    }
    pub fn get_context(&self) -> Result<Option<Context>, serde_json::Error> {
        self.0.get(Self::CONTEXT_ID_KEY)
    }
    pub fn log_out(self) {
        self.0.purge()
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
