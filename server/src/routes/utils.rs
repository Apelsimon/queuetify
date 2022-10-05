use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use rspotify::{scopes, AuthCodeSpotify, Credentials, OAuth};

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn get_spotify() -> AuthCodeSpotify {
    // TODO: handle errors
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!("user-read-currently-playing")).unwrap();
    AuthCodeSpotify::new(creds, oauth)
}
