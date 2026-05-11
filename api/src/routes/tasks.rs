use axum::{Extension, http::HeaderMap};
use chrono::{Duration, Utc};
use common::Adaptor;
use tracing::info;
use utoipa_axum::routes;

use crate::{AdaptorExtension, ConfigExtension, Router, errors::ApiError};

pub fn router() -> Router {
    Router::new().routes(routes!(cleanup))
}

#[utoipa::path(
    get,
    path = "/cleanup",
    responses(
        (status = 200, description = "Cleanup complete"),
        (status = 401, description = "Missing or incorrect X-Cron-Key header"),
        (status = 429, description = "Too many requests"),
    ),
    security((), ("cron-key" = [])),
    tag = "tasks",
)]
/// Delete events older than 3 months
pub async fn cleanup(
    Extension(adaptor): AdaptorExtension,
    Extension(config): ConfigExtension,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    // Check cron key
    let cron_key_header: String = headers
        .get("X-Cron-Key")
        .map(|k| k.to_str().unwrap_or_default().into())
        .unwrap_or_default();

    if let Some(env_key) = config.cron_key.as_ref()
        && env_key != &cron_key_header
    {
        return Err(ApiError::NotAuthorized);
    }

    info!("Running cleanup task");

    let result = adaptor
        .delete_events(Utc::now() - Duration::days(90))
        .await
        .map_err(ApiError::AdaptorError)?;

    info!(
        "Cleanup successful: {} events and {} people removed",
        result.event_count, result.person_count
    );

    Ok(())
}
