use actix::Actor;
use actix_files as fs;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use env_logger::Env;
use secrecy::ExposeSecret;
use server::configuration::{get_configuration};
use server::controller::Controller;
use server::middleware::reject_anonymous_users;
use server::routes::{callback, create_session, index, join, session_index, ws_connect,
    search, queue};
use server::db::Database;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to get configuration");

    let hmac_secret = configuration.application.hmac_secret;
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let redis_store = RedisSessionStore::new(configuration.redis_uri.expose_secret()).await?;
    let db = Data::new(Database::new(&configuration.database));

    let controller = Controller::default().start();
    let address = format!("127.0.0.1:{}", configuration.application.port);

    HttpServer::new(move || {
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
                    .route("/search", web::get().to(search))
                    .route("/queue", web::post().to(queue))
            )
            .service(fs::Files::new("/static", "."))
            .app_data(db.clone())
            .app_data(web::Data::new(controller.clone()))
            .app_data(web::Data::new(configuration.spotify.clone()))
    })
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
