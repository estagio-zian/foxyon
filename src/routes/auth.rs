use super::get_circuit_id;

use crate::session::{Session, SessionCache};

use actix_web::{HttpResponse, HttpRequest, Result, web};

#[cfg(feature = "debug")]
use tracing::info;

// Handler for requests validated through Nginx `auth_subrequest`.
//
// - Returns HTTP 204 if the session is valid (user already authenticated).
// - Returns HTTP 401 if the client must solve the PoW challenge.
pub async fn auth(req: HttpRequest, session: web::Data<SessionCache>) -> Result<HttpResponse> {
    let circuit_id = get_circuit_id(req.headers().get("X-Circuit-ID"))?;
    if session.get_ref().contains(circuit_id).await {
        #[cfg(feature = "debug")]
        info!("Circuit ID: {circuit_id} authenticated successfully");
        return Ok(HttpResponse::Ok().finish());
    }
    #[cfg(feature = "debug")]
    info!("Circuit ID: {circuit_id} not authenticated");
    Ok(HttpResponse::Unauthorized().finish())
}