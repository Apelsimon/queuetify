use actix_files as fs;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use dotenv;
use env_logger::Env;
use server::routes::{callback, create_session, index, join, session};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: crash if not available + read different file dependent on config
    dotenv::from_filename(".env.local").ok();

    let hmac_secret = "super-long-and-secret-random-key-needed-to-verify-message-integrity"; // TODO: env/config
    let secret_key = Key::from(hmac_secret.as_bytes());
    let redis_uri = "redis://127.0.0.1:6379"; // TODO: env/config
    let redis_store = RedisSessionStore::new(redis_uri).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .route("/", web::get().to(index))
            .route("/create", web::get().to(create_session))
            .route("/callback", web::get().to(callback))
            .route("/join", web::get().to(join))
            .route("/session", web::get().to(session))
            .service(fs::Files::new("/", "."))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
