use env_logger::Env;
use queuetify::application::Application;
use queuetify::configuration::get_configuration;
use queuetify::session_agent::SessionAgent;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let settings = get_configuration().expect("Failed to get configuration");
    let (agent, agent_tx) = SessionAgent::build(settings.clone());
    let application = Application::build(settings, agent_tx).await?;
    let application_task = tokio::spawn(application.run());
    let agent_task = tokio::spawn(agent.run());

    // TODO: handle agent run exit and thread join
    tokio::select! {
        _ = application_task => {log::info!("Application task");},
        _ = agent_task =>  { log::info!("Application task"); }
    };

    Ok(())
}
