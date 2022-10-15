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
use server::configuration::{get_configuration, DatabaseSettings};
use server::controller::Controller;
use server::middleware::reject_anonymous_users;
use server::routes::{callback, create_session, index, join, session, ws_connect};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

pub fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    let options = PgConnectOptions::new()
        .host(&settings.host)
        .username(&settings.username)
        .password(&settings.password.expose_secret())
        .database(&settings.database_name)
        .port(settings.port);

    PgPoolOptions::new()
        // .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(options)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to get configuration");

    let hmac_secret = configuration.application.hmac_secret;
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let redis_store = RedisSessionStore::new(configuration.redis_uri.expose_secret()).await?;
    let db_pool = Data::new(get_connection_pool(&configuration.database));

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
                    .route("/", web::get().to(session))
                    .route("/ws", web::get().to(ws_connect)),
            )
            .service(fs::Files::new("/static", "."))
            .app_data(db_pool.clone())
            .app_data(web::Data::new(controller.clone()))
            .app_data(web::Data::new(configuration.spotify.clone()))
    })
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
