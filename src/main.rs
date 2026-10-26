mod handlers;
mod models;
mod parser;
mod state;

use dotenvy::dotenv;
use handlers::{
    dashboard_handler, login_get_handler, login_post_handler, logout_handler, setup_get_handler,
    setup_post_handler,
};
use parser::parse_and_store_logs;
use sqlx::SqlitePool;
use state::{AppState, LogSource};
use std::{env, sync::Arc};
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://nginx_analytics.db?mode=rwc".into());
    let db = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to SQLite");

    // Initialize log_entries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS log_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT,
            ip TEXT,
            timestamp TEXT,
            method TEXT,
            path TEXT,
            status INTEGER,
            size INTEGER,
            referer TEXT,
            user_agent TEXT
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("Failed to create log_entries table");

    // Initialize log_states table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS log_states (
            path TEXT PRIMARY KEY,
            inode INTEGER NOT NULL,
            last_position INTEGER NOT NULL
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("Failed to create log_states table");

    // Initialize users table with UUID7 primary key and admin role support
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'admin'
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("Failed to create users table");

    let log_sources_env =
        env::var("LOG_SOURCES").unwrap_or_else(|_| "default=access.log".into());
    let log_sources: Vec<LogSource> = log_sources_env
        .split(',')
        .filter_map(|s| {
            let mut parts = s.splitn(2, '=');
            let name = parts.next()?.trim().to_string();
            let path = parts.next()?.trim().to_string();
            Some(LogSource { name, path })
        })
        .collect();

    let log_sources = if log_sources.is_empty() {
        vec![LogSource {
            name: "default".into(),
            path: "access.log".into(),
        }]
    } else {
        log_sources
    };

    let state = Arc::new(AppState {
        provisional_pass: env::var("PROVISIONAL_PASS").unwrap_or_else(|_| "power".into()),
        css_path: env::var("CSS_PATH").unwrap_or_else(|_| "/static/css/style.css".into()),
        js_path: env::var("JS_PATH").unwrap_or_else(|_| "/static/js/app.js".into()),
        log_sources,
        db,
    });

    // Background Worker for continuous parsing and retention cleanup
    let worker_state = state.clone();
    tokio::spawn(async move {
        let mut cleanup_counter: u32 = 0;

        loop {
            for source in &worker_state.log_sources {
                parse_and_store_logs(&worker_state.db, &source.name, &source.path).await;
            }

            cleanup_counter += 1;
            if cleanup_counter >= 120 {
                cleanup_counter = 0;
                let _ = sqlx::query(
                    "DELETE FROM log_entries WHERE timestamp < datetime('now', '-30 days')"
                )
                .execute(&worker_state.db)
                .await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let app = Router::new()
        .route("/", get(login_get_handler).post(login_post_handler))
        .route("/setup", get(setup_get_handler).post(setup_post_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/logout", post(logout_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(session_layer)
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}