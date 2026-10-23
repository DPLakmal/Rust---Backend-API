use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::{error::AppError, state::AppState};

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let headers = req.headers();

    let token = headers
        .get("Authorization")
        .and_then(|val| val.to_str().ok())
        .and_then(|val| {
            if val.starts_with("Bearer ") {
                Some(val.trim_start_matches("Bearer ").trim())
            } else {
                None
            }
        })
        .or_else(|| {
            headers
                .get("X-API-Key")
                .and_then(|val| val.to_str().ok())
                .map(|val| val.trim())
        });

    match token {
        Some(t) if t == state.config.api_key => Ok(next.run(req).await),
        _ => Err(AppError::Unauthorized(
            "Missing or invalid API key credentials in Authorization or X-API-Key header".to_string(),
        )),
    }
}
