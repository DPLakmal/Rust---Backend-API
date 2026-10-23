use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemDto {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemDto {
    pub title: Option<String>,
    pub completed: Option<bool>,
}
