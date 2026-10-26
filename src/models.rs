use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Serialize, FromRow)]
pub struct LogEntry {
    pub domain: Option<String>,
    pub ip: String,
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status: i64,
    pub size: i64,
    pub referer: String,
    pub user_agent: String,
}

#[derive(Clone, Serialize, FromRow)]
pub struct MetricItem {
    pub label: String,
    pub count: i64,
}

#[derive(Clone, Serialize, FromRow)]
pub struct StatusItem {
    pub status: i64,
    pub count: i64,
}

#[derive(Clone, Serialize)]
pub struct DomainOption {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct DashboardQuery {
    pub domain: Option<String>,
    pub page: Option<i64>,
    pub search: Option<String>,
    pub use_date_filter: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}