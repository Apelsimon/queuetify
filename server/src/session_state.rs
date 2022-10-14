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
    const CONTEXT_ID_KEY: &'static str = "context_id";

    pub fn renew(&self, id: Uuid, ctx: Context) -> Result<(), serde_json::Error> {
        self.0.renew();

        self.0.insert(Self::ID_KEY, id)?;
        self.0.insert(Self::CONTEXT_ID_KEY, ctx)
    }

    pub fn get_id(&self) -> Result<Option<Uuid>, serde_json::Error> {
        self.0.get(Self::ID_KEY)
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
