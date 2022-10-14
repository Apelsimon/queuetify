use crate::routes::utils::get_default_spotify;

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

pub async fn create_session() -> Result<HttpResponse, actix_web::Error> {
    let spotify = get_default_spotify().ok_or(actix_web::error::ErrorInternalServerError(
        "Unable to get default spotify",
    ))?;

    if let Ok(url) = spotify.get_authorize_url(false) {
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(url));
    }

    Err(actix_web::error::ErrorInternalServerError(
        "Unable to get authorize url",
    ))
}
