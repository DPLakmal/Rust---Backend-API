use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Database(err) => {
                tracing::error!("Database query error: {:?}", err);
                match err {
                    sqlx::Error::RowNotFound => {
                        (StatusCode::NOT_FOUND, "Requested record not found".to_string())
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected internal database error occurred".to_string(),
                    ),
                }
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status_code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
