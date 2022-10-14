use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth};

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn get_default_spotify() -> Option<AuthCodeSpotify> {
    let config = Config {
        token_cached: true,
        ..Default::default()
    };
    let creds = Credentials::from_env()?;
    let oauth = OAuth::from_env(scopes!("user-read-currently-playing"))?;

    Some(AuthCodeSpotify::with_config(creds, oauth, config))
}
