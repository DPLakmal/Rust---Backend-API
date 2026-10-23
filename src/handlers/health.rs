use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::{error::AppError, state::AppState};

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    // Ping SQLite database
    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    let db_status = if row.0 == 1 { "connected" } else { "unreachable" };

    Ok(Json(json!({
        "status": "healthy",
        "database": db_status,
        "timestamp": chrono::Utc::now(),
    })))
}
