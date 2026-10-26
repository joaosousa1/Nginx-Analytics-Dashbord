use sqlx::SqlitePool;

#[derive(Clone)]
pub struct LogSource {
    pub name: String,
    pub path: String,
}

#[derive(Clone)]
pub struct AppState {
    pub provisional_pass: String,
    pub css_path: String,
    pub js_path: String,
    pub log_sources: Vec<LogSource>,
    pub db: SqlitePool,
}