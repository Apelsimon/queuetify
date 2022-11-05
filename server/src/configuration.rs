use config::Config;
use secrecy::Secret;

#[derive(serde:: Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub spotify: SpotifySettings,
    pub redis_uri: Secret<String>,
}

#[derive(serde:: Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub hmac_secret: Secret<String>,
}

#[derive(serde:: Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde:: Deserialize, Clone)]
pub struct SpotifySettings {
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    pub redirect_uri: Secret<String>,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let _ = dotenv::from_filename(".env.local");

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(
            config::Environment::with_prefix("QUEUETIFY_APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    settings.try_deserialize()
}
