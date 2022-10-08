use crate::session_state::TypedSession;
use crate::templates::TEMPLATES;
use actix_web::HttpResponse;
use tera::Context;

pub async fn index(_session: TypedSession) -> HttpResponse {
    let ctx = Context::new();
    let rendered = TEMPLATES.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
