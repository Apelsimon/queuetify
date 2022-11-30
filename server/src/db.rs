use std::str::FromStr;

use crate::configuration::{DatabaseSettings, SpotifySettings};
use crate::controller::Vote;
use crate::spotify::{create_token_from_string, get_default_spotify, get_token_string};
use rspotify::model::TrackId;
use rspotify::AuthCodeSpotify;
use secrecy::ExposeSecret;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use sqlx::postgres::PgSslMode;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    spotify_settings: SpotifySettings,
}

pub struct Session {
    pub token: String, // TODO: make secret
    pub current_track_uri: Option<String>,
}

pub struct State {
    pub current_track_uri: Option<TrackId>,
    pub current_queue: Vec<TrackId>,
}

// TODO: take TrackId references instead?
impl Database {
    pub fn new(settings: &DatabaseSettings, spotify_settings: SpotifySettings) -> Self {
        Self {
            pool: get_connection_pool(settings),
            spotify_settings,
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

    pub async fn delete_session(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query!(
            r#"
                DELETE FROM votes 
                WHERE session_id = $1
            "#,
            id
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query!(
            r#"
                DELETE FROM queued_tracks 
                WHERE session_id = $1
            "#,
            id
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query!(
            r#"
                DELETE FROM sessions 
                WHERE id = $1
            "#,
            id
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn get_session_impl(
        &self,
        transaction: &mut Transaction<'static, Postgres>,
        id: Uuid,
    ) -> Result<Session, sqlx::Error> {
        let session = sqlx::query_as!(
            Session,
            r#"
                SELECT token, current_track_uri FROM sessions where id = $1
            "#,
            id
        )
        .fetch_one(transaction)
        .await?;
        Ok(session)
    }

    pub async fn get_session(&self, id: Uuid) -> Result<Session, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let session = self.get_session_impl(&mut transaction, id).await?;
        transaction.commit().await?;
        Ok(session)
    }

    async fn get_current_track_impl(
        &self,
        transaction: &mut Transaction<'static, Postgres>,
        id: Uuid,
    ) -> Result<Option<TrackId>, sqlx::Error> {
        let session = self.get_session_impl(transaction, id).await?;

        let track_id: Option<TrackId> = match session.current_track_uri {
            Some(current_track_uri) => {
                match TrackId::from_str(&current_track_uri) {
                    Ok(id) => Some(id),
                    Err(_) => None, //TODO: handle invalid stored uri
                }
            }
            None => None,
        };

        Ok(track_id)
    }

    pub async fn get_current_track(
        &self,
        id: Uuid,
    ) -> Result<(Option<TrackId>, Transaction<'static, Postgres>), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let track_id = self.get_current_track_impl(&mut transaction, id).await?;
        Ok((track_id, transaction))
    }

    pub async fn set_current_track(
        &self,
        mut transaction: Transaction<'static, Postgres>,
        id: Uuid,
        track_id: Option<TrackId>,
    ) -> Result<(), sqlx::Error> {
        let track_id = match track_id {
            Some(id) => Some(id.to_string()),
            None => None,
        };
        sqlx::query!(
            r#"
                UPDATE sessions 
                SET 
                    current_track_uri = $2
                WHERE
                    id = $1
            "#,
            id,
            track_id
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

    pub async fn pop_track_from_queue(
        &self,
        id: Uuid,
        transaction: &mut Transaction<'static, Postgres>,
    ) -> Result<Option<TrackId>, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"
                DELETE FROM queued_tracks 
                WHERE track_uri = any (array(SELECT track_uri FROM queued_tracks WHERE session_id = $1 ORDER BY votes DESC LIMIT 1)) RETURNING track_uri;
            "#
        )
        .bind(id)
        .fetch_optional(transaction)
        .await?;

        let track_id: Option<TrackId> = match result {
            Some((track_uri,)) => {
                match TrackId::from_str(&track_uri) {
                    Ok(id) => Some(id),
                    Err(_) => None, //TODO: handle invalid stored uri
                }
            }
            None => None,
        };

        Ok(track_id)
    }

    async fn get_queue_impl(
        &self,
        transaction: &mut Transaction<'static, Postgres>,
        id: Uuid,
    ) -> Result<Vec<TrackId>, sqlx::Error> {
        let uris: Vec<(String,)> = sqlx::query_as(
            r#"
                    SELECT track_uri FROM queued_tracks where session_id = $1 ORDER BY votes DESC
                "#,
        )
        .bind(id)
        .fetch_all(transaction)
        .await?;

        let mut queue = Vec::new();
        for (uri,) in uris.iter() {
            match TrackId::from_str(&uri) {
                Ok(id) => {
                    queue.push(id);
                }
                _ => {}
            }
        }

        Ok(queue)
    }

    pub async fn get_current_state(&self, id: Uuid) -> Result<State, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let current_track_uri = self.get_current_track_impl(&mut transaction, id).await?;
        let current_queue = self.get_queue_impl(&mut transaction, id).await?;
        transaction.commit().await?;
        Ok(State {
            current_track_uri,
            current_queue,
        })
    }

    pub async fn add_vote(&self, msg: &Vote) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query!(
            r#"
                INSERT INTO votes
                    (client_id, session_id, track_uri)
                VALUES ($1, $2, $3)
            "#,
            msg.connection_id,
            msg.session_id,
            msg.track_id.to_string(),
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query!(
            r#"
                UPDATE queued_tracks 
                SET 
                    votes = votes + 1
                WHERE
                    track_uri = $1 and session_id = $2
            "#,
            msg.track_id.to_string(),
            msg.session_id,
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn remove_votes(
        &self,
        transaction: &mut Transaction<'static, Postgres>,
        id: Uuid,
        track_id: TrackId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM votes 
            WHERE track_uri = $1 and session_id = $2
            "#,
            track_id.to_string(),
            id
        )
        .execute(transaction)
        .await?;
        Ok(())
    }

    pub async fn set_spotify(
        &self,
        id: Uuid,
        spotify: &AuthCodeSpotify,
    ) -> Result<(), anyhow::Error> {
        let token = get_token_string(&spotify).await?;
        sqlx::query!(
            r#"
            UPDATE sessions
            SET
                token = $2
            WHERE id = $1
            "#,
            id,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_spotify(&self, id: Uuid) -> Result<AuthCodeSpotify, anyhow::Error> {
        let session = self.get_session(id).await?;
        let spotify = get_default_spotify(&self.spotify_settings);
        let token = create_token_from_string(&session.token)?;
        *spotify.token.lock().await.unwrap() = Some(token);
        Ok(spotify)
    }

    pub async fn voted_tracks(
        &self,
        id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let uris: Vec<(String,)> = sqlx::query_as(
            r#"
                    SELECT track_uri FROM votes where session_id = $1 and client_id = $2
                "#,
        )
        .bind(id)
        .bind(client_id)
        .fetch_all(&self.pool)
        .await?;

        let mut voted_tracks = Vec::new();
        for (uri,) in uris.iter() {
            voted_tracks.push(uri.clone());
        }

        Ok(voted_tracks)
    }
}

fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    let ssl_mode = if settings.require_ssl {
        PgSslMode:: Require
    } else {
        // Try an encrypted connection, fallback to unencrypted if it fails
        PgSslMode:: Prefer
    };
    let options = PgConnectOptions::new()
        .host(&settings.host)
        .username(&settings.username)
        .password(&settings.password.expose_secret())
        .database(&settings.database_name)
        .port(settings.port)
        .ssl_mode(ssl_mode);

    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(options)
}
