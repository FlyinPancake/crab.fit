use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use common::AdaptorError;

pub enum ApiError {
    AdaptorError(AdaptorError),
    NotFound,
    NotAuthorized,
}

// Define what the error types above should return
impl IntoResponse for ApiError {
    fn into_response(self) -> Response<axum::body::Body> {
        match self {
            ApiError::AdaptorError(AdaptorError::Conflict) => StatusCode::CONFLICT.into_response(),
            ApiError::AdaptorError(AdaptorError::InvalidInput(_)) => {
                StatusCode::UNPROCESSABLE_ENTITY.into_response()
            }
            ApiError::AdaptorError(AdaptorError::Internal(e)) => {
                tracing::error!(?e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ApiError::NotFound => StatusCode::NOT_FOUND.into_response(),
            ApiError::NotAuthorized => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}
