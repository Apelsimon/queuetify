use crate::routes::utils::see_other;
use crate::session_state::TypedSession;
use actix_web::HttpResponse;

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.log_out();
    Ok(see_other("/"))
}
