use crate::configuration::DatabaseSettings;
use rspotify::model::TrackId;
use secrecy::ExposeSecret;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

pub struct Session {
    pub token: String, // TODO: make secret
}

// TODO: take TrackId references instead?
impl Database {
    pub fn new(settings: &DatabaseSettings) -> Self {
        Self {
            pool: get_connection_pool(settings),
        }
    }

    pub async fn session_exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let (ok,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1)")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(ok)
    }

    pub async fn new_session(&self, id: Uuid, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO sessions (
                    id, token, created_at
                )
                VALUES ($1, $2, now())
            "#,
            id,
            token
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

    pub async fn has_current_track(
        &self,
        id: Uuid,
    ) -> Result<(bool, Transaction<'static, Postgres>), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT current_track_uri FROM sessions WHERE id = $1 and current_track_uri is not null)")
            .bind(id)
            .fetch_one(&mut transaction)
            .await?;

        Ok((exists, transaction))
    }

    pub async fn set_current_track(
        &self,
        mut transaction: Transaction<'static, Postgres>,
        id: Uuid,
        track_id: TrackId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE sessions 
                SET 
                    current_track_uri = $2
                WHERE
                    id = $1
            "#,
            id,
            track_id.to_string()
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn queue_track(
        &self,
        mut transaction: Transaction<'static, Postgres>,
        id: Uuid,
        track_id: TrackId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO queued_tracks
                    (track_uri, session_id)
                VALUES ($1, $2)
                ON CONFLICT (track_uri, session_id) DO NOTHING
            "#,
            track_id.to_string(),
            id
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
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
