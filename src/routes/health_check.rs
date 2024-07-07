//! src/routes/health_check.rs

use actix_web::{HttpResponse, Responder};

#[tracing::instrument(
    name = "Checking health",
)]
pub async fn health_check() -> impl Responder {
    return HttpResponse::Ok();
}
