use env_logger::Env;
use server::configuration::{get_configuration};
use server::application::Application;
use server::session_agent::SessionAgent;
use std::thread;
use server::db::Database;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let settings = get_configuration().expect("Failed to get configuration");
    let db = Database::new(&settings.database);
    let (agent, agent_tx) = SessionAgent::new(db.clone());
    let application = Application::build(settings, agent_tx, db).await?;

    // TODO: handle agent run exit and thread join
    thread::spawn(|| {
        let _ = agent.run();
    });

    application.run().await?;

    Ok(())
}
