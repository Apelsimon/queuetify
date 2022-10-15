use crate::configuration::SpotifySettings;
use crate::spotify::get_default_spotify;

use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web::HttpResponse;

pub async fn create_session(
    settings: web::Data<SpotifySettings>,
) -> Result<HttpResponse, actix_web::Error> {
    let spotify = get_default_spotify(&settings);

    if let Ok(url) = spotify.get_authorize_url(false) {
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(url));
    }

    Err(actix_web::error::ErrorInternalServerError(
        "Unable to get authorize url",
    ))
}
