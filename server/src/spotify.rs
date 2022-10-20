use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth};

use crate::configuration::SpotifySettings;
use crate::session_state::TypedSession;
use crate::routes::utils::e500;
use crate::db::Database;
use secrecy::ExposeSecret;
use rspotify::clients::BaseClient;
use rspotify::Token;

pub fn get_default_spotify(settings: &SpotifySettings) -> AuthCodeSpotify {
    let config = Config::default();
    let creds = Credentials::new(
        settings.client_id.expose_secret(),
        settings.client_secret.expose_secret(),
    );
    // TODO: limit scopes
    let scopes = scopes!(
        "user-read-email",
        "user-read-private",
        "user-top-read",
        "user-read-recently-played",
        "user-follow-read",
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "playlist-read-collaborative",
        "playlist-read-private",
        "user-follow-modify",
        "user-library-modify",
        "user-modify-playback-state",
        "playlist-modify-public",
        "playlist-modify-private",
        "ugc-image-upload"
    );
    let oauth = OAuth {
        redirect_uri: settings.redirect_uri.expose_secret().to_string(),
        scopes,
        ..Default::default()
    };

    AuthCodeSpotify::with_config(creds, oauth, config)
}

pub async fn get_token_string(spotify: &AuthCodeSpotify) -> Result<String, serde_json::Error> {
    let token = spotify.get_token().lock().await.unwrap().clone();
    serde_json::to_string(&token)
}

fn from_token_string(token: &str) -> Result<AuthCodeSpotify, serde_json::Error> {
    let token = serde_json::from_str::<Token>(token)?;
    Ok(AuthCodeSpotify::from_token(token))
}

pub async fn get_spotify_from_db(typed_session: &TypedSession, db: &Database) -> Result<AuthCodeSpotify, actix_web::Error> {

    // TODO: handle unwrap
    let id = typed_session.get_id().unwrap().unwrap();
    let session = db.get_session(id).await.map_err(e500)?;
    let spotify = from_token_string(&session.token).map_err(e500)?;
    Ok(spotify)
}   