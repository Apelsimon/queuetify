use crate::routes::utils::get_spotify;

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

pub async fn create_session() -> HttpResponse {
    // TODO: don't want to enter here if already authenticated

    let spotify = get_spotify();
    let url = spotify.get_authorize_url(false).unwrap(); // TODO: handle error

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(url)
}
