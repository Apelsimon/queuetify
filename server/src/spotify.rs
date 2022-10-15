use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth};

use crate::configuration::SpotifySettings;
use secrecy::ExposeSecret;
use rspotify::clients::BaseClient;
use rspotify::Token;

pub fn get_default_spotify(settings: &SpotifySettings) -> AuthCodeSpotify {
    let config = Config::default();
    let creds = Credentials::new(
        settings.client_id.expose_secret(),
        settings.client_secret.expose_secret(),
    );
    let oauth = OAuth {
        redirect_uri: settings.redirect_uri.expose_secret().to_string(),
        scopes: scopes!("user-read-currently-playing"),
        ..Default::default()
    };

    AuthCodeSpotify::with_config(creds, oauth, config)
}

pub async fn get_token_string(spotify: &AuthCodeSpotify) -> Result<String, serde_json::Error> {
    let token = spotify.get_token().lock().await.unwrap().clone();
    serde_json::to_string(&token)
}

pub fn from_token_string(token: &str) -> Result<AuthCodeSpotify, serde_json::Error> {
    let token = serde_json::from_str::<Token>(token)?;
    Ok(AuthCodeSpotify::from_token(token))
}