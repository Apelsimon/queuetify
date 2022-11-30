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
    pub require_ssl: bool,
}

#[derive(serde:: Deserialize, Clone)]
pub struct SpotifySettings {
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    pub redirect_uri: Secret<String>,
}

enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
    Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let _ = dotenv::from_filename(".env.secret");

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    // let environment: Environment = std::env::var("APP_ENVIRONMENT")
    //     .unwrap_or_else(|_| "local".into())
    //     .try_into()
    //     .expect("Failed to parse APP_ENVIRONMENT.");
    // let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        // .add_source(config::File::from(
        //     configuration_directory.join(&environment_filename),
        // ))
        .add_source(
            config::Environment::with_prefix("QUEUETIFY_APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    settings.try_deserialize()
}
