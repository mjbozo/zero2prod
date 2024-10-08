//! src/routes/home/mod.rs

use actix_web::{http::header::ContentType, HttpResponse};

pub async fn home() -> HttpResponse {
    return HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home.html"));
}
