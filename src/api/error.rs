use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub inner: anyhow::Error,
}

pub trait ApiErrorVariant<T> {
    fn to_apierror(self, code: StatusCode) -> Result<T, ApiError>;
}

impl<T, E> ApiErrorVariant<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn to_apierror(self, code: StatusCode) -> Result<T, ApiError> {
        self.map_err(|e| ApiError::new(code, e.into()))
    }
}

impl ApiError {
    pub fn new(s: StatusCode, e: anyhow::Error) -> Self {
        Self {
            status: s,
            inner: e,
        }
    }
}

// Tell axum how to convert `ApiError` into a response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": self.inner.to_string()
            })),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, ApiError>`. That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            inner: err.into(),
        }
    }
}
