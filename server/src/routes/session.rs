use crate::routes::utils::see_other;
use crate::session_state::TypedSession;
use crate::templates::TEMPLATES;
use actix_web::HttpResponse;
use tera::Context;

pub async fn session(session: TypedSession) -> HttpResponse {
    match session.get_user_id() {
        Ok(None) | Err(_) => {
            log::info!("Not authed, redirect");
            return see_other("/");
        }
        _ => {}
    }

    let ctx = Context::new();
    let rendered = TEMPLATES.render("session.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
