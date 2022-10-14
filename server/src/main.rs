use actix::Actor;
use actix_files as fs;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use dotenv;
use env_logger::Env;
use server::controller::Controller;
use server::middleware::reject_anonymous_users;
use server::routes::{callback, create_session, index, join, session, ws_connect};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

pub fn get_connection_pool() -> PgPool {
    let options = PgConnectOptions::new()
        .host("localhost")
        .username("postgres")
        .password("password")
        .database("queuetify")
        .port(5432);

    PgPoolOptions::new()
        // .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(options)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: crash if not available + read different file dependent on config
    dotenv::from_filename(".env.local").ok();

    let hmac_secret = "super-long-and-secret-random-key-needed-to-verify-message-integrity"; // TODO: env/config
    let secret_key = Key::from(hmac_secret.as_bytes());
    let redis_uri = "redis://127.0.0.1:6379"; // TODO: env/config
    let redis_store = RedisSessionStore::new(redis_uri).await?;
    let db_pool = Data::new(get_connection_pool());

    let controller = Controller::default().start();

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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
