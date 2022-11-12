use crate::configuration::Settings;
use crate::controller::Controller;
use crate::db::Database;
use crate::middleware::reject_anonymous_users;
use crate::routes::{callback, create_session, index, logout, join, session_index, ws_connect};
use crate::session_agent::SessionAgentRequest;
use actix::Actor;
use actix_files as fs;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use secrecy::ExposeSecret;
use tokio::sync::mpsc::UnboundedSender;

pub struct Application {
    server: Server,
}

// TODO: redirect valid sessions away from non /session paths
impl Application {
    pub async fn build(
        settings: Settings,
        agent_tx: UnboundedSender<SessionAgentRequest>,
    ) -> Result<Self, anyhow::Error> {
        let hmac_secret = settings.application.hmac_secret;
        let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
        let redis_store = RedisSessionStore::new(settings.redis_uri.expose_secret()).await?;
        let db = web::Data::new(Database::new(&settings.database, settings.spotify.clone()));
        let controller = Controller::new(agent_tx).start();
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
                        .route("/logout", web::get().to(logout))
                )
                .service(fs::Files::new("/static", "."))
                .app_data(db.clone())
                .app_data(web::Data::new(controller.clone()))
                .app_data(web::Data::new(settings.spotify.clone()))
        })
        .bind(address)?
        .run();

        Ok(Self { server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
