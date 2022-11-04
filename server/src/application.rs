use actix_web::dev::Server;
use actix_files as fs;
use crate::configuration::Settings;
use actix_web::cookie::Key;
use actix_session::storage::RedisSessionStore;
use actix_web::{web, App, HttpServer};
use secrecy::ExposeSecret;
use crate::db::Database;
use crate::controller::Controller;
use actix_session::SessionMiddleware;
use actix::Actor;
use crate::routes::{callback, create_session, index, join, session_index, ws_connect};
use actix_web_lab::middleware::from_fn;
use crate::middleware::reject_anonymous_users;
use std::sync::mpsc::Sender;
use crate::session_agent::SessionAgentRequest;
use tokio::sync::mpsc::UnboundedSender;

pub struct Application {
    server: Server
}

impl Application {
    pub async fn build(settings: Settings, agent_tx: UnboundedSender<SessionAgentRequest>, 
        db: Database) -> Result<Self, anyhow::Error> {
        let hmac_secret = settings.application.hmac_secret;
        let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
        let redis_store = RedisSessionStore::new(settings.redis_uri.expose_secret()).await?;
        let db = web::Data::new(db);

        let controller = Controller::new(db.clone(), agent_tx).start();
        let address = format!("127.0.0.1:{}", settings.application.port);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                ))
                .route("/", web::get().to(index))
                .route("/create", web::get().to(create_session))
                .route("/callback", web::get().to(callback))
                .route("/join/{id}", web::get().to(join))
                .service(
                    web::scope("/session")
                        .wrap(from_fn(reject_anonymous_users))
                        .route("/", web::get().to(session_index))
                        .route("/ws", web::get().to(ws_connect))
                )
                .service(fs::Files::new("/static", "."))
                .app_data(db.clone())
                .app_data(web::Data::new(controller.clone()))
                .app_data(web::Data::new(settings.spotify.clone()))
        })
        .bind(address)?
        .run();

        Ok(Self{server})
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}