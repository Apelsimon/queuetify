use crate::routes::utils::{get_spotify, see_other};

use actix_web::HttpResponse;

pub async fn create_session() -> HttpResponse {
    // TODO: don't want to enter here if already authenticated
    
    let spotify = get_spotify();
    let url = spotify.get_authorize_url(false).unwrap();

    see_other(&url)
}
