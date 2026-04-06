use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use chrono::DateTime;
use chrono::Utc;

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProgress {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProgress {
    pub content: String,
    pub progress_percentage: Option<i32>,
    pub update_date: Option<chrono::NaiveDate>,
}



#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNeed {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNeed {
    pub title: String,
    pub description: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNeed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
}