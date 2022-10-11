use actix_files as fs;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use dotenv;
use env_logger::Env;
use server::routes::{callback, create_session, index, join, session};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use actix_web_lab::middleware::from_fn;
use server::middleware::reject_anonymous_users;

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
                web::resource("/session").route(web::get().to(session))
                        .wrap(from_fn(reject_anonymous_users))
            )
            .service(fs::Files::new("/", "."))
            .app_data(db_pool.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
