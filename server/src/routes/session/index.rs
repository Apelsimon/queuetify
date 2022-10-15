use crate::routes::utils::e500;
use crate::routes::utils::see_other;
use crate::session_state::{Context, TypedSession};
use crate::templates::TEMPLATES;
use actix_web::web;
use actix_web::HttpResponse;
use rspotify::clients::OAuthClient;
use rspotify::{AuthCodeSpotify, Token};
use sqlx::PgPool;
use tera::Context as RenderContext;
use crate::spotify::from_token_string;
use crate::db::Database;

pub async fn session_index(
    typed_session: TypedSession,
    db: web::Data<Database>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Ok to assume id exists here because of protected route?
    let id = typed_session.get_id().unwrap().unwrap();

    let session = db.get_session(id).await.map_err(e500)?;

    let spotify = from_token_string(&session.token).map_err(e500)?;
    let mut render_context = RenderContext::new();

    if let Ok(user_info) = spotify.me().await {
        // TODO: Ok to assume context exists here because of protected route?
        let context = typed_session.get_context().unwrap().unwrap();

        if context == Context::Host {
            render_context.insert("session_url", &id.to_string());
        }

        render_context.insert("display_name", &user_info.display_name.unwrap());
        render_context.insert("context", &context.to_string());
    } else {
        log::info!("Failed to spotify.me()");
        return Ok(see_other("/"));
    }

    let rendered = TEMPLATES.render("session.html", &render_context).unwrap();
    Ok(HttpResponse::Ok().body(rendered))
}
