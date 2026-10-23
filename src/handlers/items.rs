use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::{
    error::AppError,
    models::{CreateItemDto, Item, UpdateItemDto},
    state::AppState,
};

/// GET /api/items - Retrieve all items
pub async fn list_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, AppError> {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, title, completed, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(items))
}

/// POST /api/items - Create a new item
pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemDto>,
) -> Result<(StatusCode, Json<Item>), AppError> {
    let trimmed_title = payload.title.trim();
    if trimmed_title.is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    let item_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let completed = false;

    let item = sqlx::query_as::<_, Item>(
        r#"
        INSERT INTO items (id, title, completed, created_at)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(&item_id)
    .bind(trimmed_title)
    .bind(completed)
    .bind(now)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

/// GET /api/items/:id - Retrieve single item by ID
pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Item>, AppError> {
    let item = sqlx::query_as::<_, Item>(
        "SELECT id, title, completed, created_at FROM items WHERE id = ?1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Item with id '{id}' was not found")))?;

    Ok(Json(item))
}

/// PUT /api/items/:id - Update item fields
pub async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateItemDto>,
) -> Result<Json<Item>, AppError> {
    // First verify existence
    let existing = sqlx::query_as::<_, Item>(
        "SELECT id, title, completed, created_at FROM items WHERE id = ?1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Item with id '{id}' was not found")))?;

    let new_title = match payload.title {
        Some(t) => {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::BadRequest("Title cannot be empty".to_string()));
            }
            trimmed
        }
        None => existing.title,
    };

    let new_completed = payload.completed.unwrap_or(existing.completed);

    let updated_item = sqlx::query_as::<_, Item>(
        r#"
        UPDATE items
        SET title = ?1, completed = ?2
        WHERE id = ?3
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(new_title)
    .bind(new_completed)
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated_item))
}

/// DELETE /api/items/:id - Delete an item by ID
pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM items WHERE id = ?1")
        .bind(&id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Item with id '{id}' was not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}
