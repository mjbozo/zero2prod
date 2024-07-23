//! src/routes/subscriptions_confirm.rs

use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmSubscriberError {
    #[error("No subscriber with associated token")]
    UnauthorisedError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmSubscriberError {
    fn status_code(&self) -> StatusCode {
        return match self {
            ConfirmSubscriberError::UnauthorisedError => StatusCode::UNAUTHORIZED,
            ConfirmSubscriberError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
    }
}

impl std::fmt::Debug for ConfirmSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return error_chain_fmt(self, f);
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriberError> {
    let subscriber_id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to retrieve the subscriber ID associated with the provided token")?
        .ok_or(ConfirmSubscriberError::UnauthorisedError)?;
    confirm_subscriber(&pool, subscriber_id)
        .await
        .context("Failed to update the subscriber status to `confirmed`")?;
    return Ok(HttpResponse::Ok().finish());
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    return Ok(());
}

#[tracing::instrument(
    name = "Get subscriber_id from token"
    skip(pool, token)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    return Ok(result.map(|r| r.subscriber_id));
}
