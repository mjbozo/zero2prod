// //! src/routes/newsletter.rs

// use actix_web::http::header::{HeaderMap, HeaderValue};
// use actix_web::http::StatusCode;
// use actix_web::{web, HttpResponse};
// use actix_web::{HttpRequest, ResponseError};
// use anyhow::Context;
// use base64::Engine;
// use reqwest::header;
// use secrecy::Secret;
// use sqlx::PgPool;

// use crate::authentication::{validate_credentials, AuthError, Credentials};
// use crate::domain::SubscriberEmail;
// use crate::email_client::EmailClient;

// use super::error_chain_fmt;

// #[derive(serde::Deserialize)]
// pub struct BodyData {
//     title: String,
//     content: Content,
// }

// #[derive(serde::Deserialize)]
// pub struct Content {
//     html: String,
//     text: String,
// }

// struct ConfirmedSubscriber {
//     email: SubscriberEmail,
// }

// #[derive(thiserror::Error)]
// pub enum PublishError {
//     #[error(transparent)]
//     UnexpectedError(#[from] anyhow::Error),
//     #[error("Authentication failed")]
//     AuthError(#[source] anyhow::Error),
// }

// impl std::fmt::Debug for PublishError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         return error_chain_fmt(self, f);
//     }
// }

// impl ResponseError for PublishError {
//     fn error_response(&self) -> HttpResponse {
//         return match self {
//             PublishError::UnexpectedError(_) => {
//                 HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
//             }
//             PublishError::AuthError(_) => {
//                 let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
//                 let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
//                 response
//                     .headers_mut()
//                     .insert(header::WWW_AUTHENTICATE, header_value);
//                 response
//             }
//         };
//     }
// }

// #[tracing::instrument(
//     name = "Publish a newsletter issue",
//     skip(body, pool, email_client, request),
//     fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
// )]
// pub async fn publish_newsletter(
//     body: web::Json<BodyData>,
//     pool: web::Data<PgPool>,
//     email_client: web::Data<EmailClient>,
//     request: HttpRequest,
// ) -> Result<HttpResponse, PublishError> {
//     let credentials = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
//     tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
//     let user_id = validate_credentials(credentials, &pool)
//         .await
//         .map_err(|e| match e {
//             AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
//             AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
//         })?;
//     tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

//     let subscribers = get_confirmed_subscribers(&pool).await?;
//     for subscriber in subscribers {
//         match subscriber {
//             Ok(subscriber) => {
//                 email_client
//                     .send_email(
//                         &subscriber.email,
//                         &body.title,
//                         &body.content.html,
//                         &body.content.text,
//                     )
//                     .await
//                     .with_context(|| {
//                         format!("Failed to send newsletter isue to {}", subscriber.email)
//                     })?;
//             }
//             Err(error) => {
//                 tracing::warn!(
//                     error.cause_chain = ?error,
//                     "Skipping a confirmed subscriber. \
//                     Their stored contact details are invalid"
//                 );
//             }
//         }
//     }
//     return Ok(HttpResponse::Ok().finish());
// }

// fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
//     let header_value = headers
//         .get("Authorization")
//         .context("The 'Authorization' header was missing")?
//         .to_str()
//         .context("The 'Authorization' header was not a valid UTF8 string")?;

//     let base64encoded_segment = header_value
//         .strip_prefix("Basic ")
//         .context("The authorisation scheme was not basic")?;
//     let decoded_bytes = base64::engine::general_purpose::STANDARD
//         .decode(base64encoded_segment)
//         .context("Failed to base64-decode 'Basic' credentials")?;
//     let decoded_credentials = String::from_utf8(decoded_bytes)
//         .context("The decoded credential string is not valid UTF8")?;

//     let mut credentials = decoded_credentials.splitn(2, ":");
//     let username = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth"))?
//         .to_string();
//     let password = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
//         .to_string();

//     return Ok(Credentials {
//         username,
//         password: Secret::new(password),
//     });
// }

// #[tracing::instrument(name = "get confirmed subscribers", skip(pool))]
// async fn get_confirmed_subscribers(
//     pool: &PgPool,
// ) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
//     let confirmed_subscribers = sqlx::query!(
//         r#"
//         SELECT email
//         FROM subscriptions
//         WHERE status = 'confirmed'
//         "#,
//     )
//     .fetch_all(pool)
//     .await?
//     .into_iter()
//     .map(|r| match SubscriberEmail::parse(r.email) {
//         Ok(email) => Ok(ConfirmedSubscriber { email }),
//         Err(error) => Err(anyhow::anyhow!(error)),
//     })
//     .collect();
//     return Ok(confirmed_subscribers);
// }
