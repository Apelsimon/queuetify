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

struct Session {
    pub token: String,
}

pub async fn session(
    typed_session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    match typed_session.get_user_id() {
        Ok(None) | Err(_) => {
            log::info!("Not authed, redirect");
            return Ok(see_other("/"));
        }
        _ => {}
    }

    // TODO: handle error
    let id = typed_session.get_id().unwrap().unwrap();

    // TODO: redirect on query miss
    let session = sqlx::query_as!(
        Session,
        r#"
            SELECT token FROM sessions where id = $1
        "#,
        id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(e500)?;

    let token = serde_json::from_str::<Token>(&session.token).unwrap(); // TODO: handle error
    let spotify = AuthCodeSpotify::from_token(token);
    let mut render_context = RenderContext::new();

    if let Ok(user_info) = spotify.me().await {
        //TODO: handle error
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
