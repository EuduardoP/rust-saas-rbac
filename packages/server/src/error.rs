use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(status: StatusCode, error: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            status,
            Json(Self {
                error: error.into(),
            }),
        )
    }

    pub fn internal_error() -> (StatusCode, Json<Self>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Self {
                error: "Internal Server Error".to_string(),
            }),
        )
    }

    pub fn unauthorized() -> (StatusCode, Json<Self>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(Self {
                error: "Unauthorized".to_string(),
            }),
        )
    }
}
