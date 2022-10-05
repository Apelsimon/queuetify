use crate::session_state::TypedSession;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

pub async fn get_context(session: TypedSession) -> HttpResponse {
    match session.get_context() {
        Ok(Some(context)) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(context),
        Ok(None) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body("none"),
        Err(_err) => HttpResponse::Unauthorized().finish(),
    }
}
