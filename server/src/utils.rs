use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth};

use crate::configuration::SpotifySettings;
use secrecy::ExposeSecret;

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
