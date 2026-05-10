use axum::{Extension, Json};
use common::Adaptor;
use utoipa_axum::routes;

use crate::{
    AdaptorExtension, Router,
    errors::ApiError,
    payloads::{ApiResult, StatsResponse},
};

pub(super) fn router() -> Router {
    Router::new().routes(routes!(get_stats))
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Ok", body = StatsResponse),
        (status = 429, description = "Too many requests"),
    ),
    tag = "info",
)]
/// Get current stats
pub async fn get_stats(Extension(adaptor): AdaptorExtension) -> ApiResult<StatsResponse> {
    let stats = adaptor.get_stats().await.map_err(ApiError::AdaptorError)?;

    Ok(Json(stats.into()))
}
