use server::routes::{callback, create_session, join, session};
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: crash if not available + read different file dependent on config
    dotenv::from_filename(".env.local").ok();

    HttpServer::new(|| {
        App::new()
            .route("/create", web::get().to(create_session))
            .route("/callback", web::get().to(callback))
            .route("/session", web::get().to(session))
            .route("/join", web::get().to(join))
            .service(fs::Files::new("/", "./public").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
