use crate::session_state::{Context, TypedSession};
use crate::templates::TEMPLATES;
use actix_web::HttpResponse;
use tera::Context as RenderContext;

pub async fn session_index(typed_session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Ok to assume id exists here because of protected route?
    let id = typed_session.get_id().unwrap().unwrap();
    let mut render_context = RenderContext::new();

    // TODO: Ok to assume context exists here because of protected route?
    let context = typed_session.get_context().unwrap().unwrap();

    if context == Context::Host {
        render_context.insert("session_id", &id.to_string());
    }

    render_context.insert("context", &context.to_string());

    let rendered = TEMPLATES.render("session.html", &render_context).unwrap();
    Ok(HttpResponse::Ok().body(rendered))
}
