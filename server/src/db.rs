use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use crate::configuration::DatabaseSettings;
use sqlx::PgPool;
use secrecy::ExposeSecret;
use uuid::Uuid;

pub struct Database {
    pool: PgPool // TODO: make private
}

pub struct Session {
    pub token: String, // TODO: make secret
}

impl Database {
    pub fn new(settings: &DatabaseSettings) -> Self {
        Self {
            pool: get_connection_pool(settings)
        }
    }

    pub async fn session_exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let (ok,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(ok)
    }

    pub async fn insert_session(&self, id: Uuid, token: &str,
        queue_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO sessions (
                    id, token, queue_id, created_at
                )
                VALUES ($1, $2, $3, now())
            "#,
            id,
            token,
            queue_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_session(&self, id: Uuid) -> Result<Session, sqlx::Error> {
        let session = sqlx::query_as!(
            Session,
            r#"
                SELECT token FROM sessions where id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(session)
    }
}

fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    let options = PgConnectOptions::new()
        .host(&settings.host)
        .username(&settings.username)
        .password(&settings.password.expose_secret())
        .database(&settings.database_name)
        .port(settings.port);

    PgPoolOptions::new()
        // .acquire_timeout(std::time::Duration::from_secs(2)) //TODO: enable
        .connect_lazy_with(options)
}