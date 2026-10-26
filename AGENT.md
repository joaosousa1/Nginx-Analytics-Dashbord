# AGENTS.md - Nginx Analytics Dashboard

This file provides context, architecture details, and current progress for the **Nginx Analytics Dashboard** project to allow seamless continuation in future development sessions.

## 1. Project Overview
A high-performance, multi-domain Nginx log analyzer built in Rust. It reads access logs incrementally in real-time via a background worker, stores them persistently in SQLite, and serves a clean, responsive web dashboard using Axum and Askama templates.

## 2. Technology Stack
- **Language/Backend:** Rust (Axum, Tokio)
- **Database & ORM:** SQLite via SQLx (Async, using `runtime-tokio-rustls`)
- **Templating:** Askama
- **Security:** Argon2 (Password Hashing), UUIDv7 (User Identifiers)
- **State & Sessions:** `tower-sessions` (MemoryStore), `std::sync::Arc` for shared `AppState`
- **Static Assets:** `tower-http` (ServeDir)

## 3. Project Structure (Rust Modules)
The application code is organized into modular files within `src/`:
- `src/main.rs`: Entry point, database initialization, background worker, and Axum router setup.
- `src/state.rs`: Defines `AppState` and `LogSource` structures.
- `src/models.rs`: Defines data models (`LogEntry`, `MetricItem`, `StatusItem`, `DomainOption`, `LoginForm`, `SetupForm`, `DashboardQuery`).
- `src/parser.rs`: Incremental log parser, Nginx timestamp-to-ISO converter, and Unix inode / logrotate safety logic.
- `src/handlers.rs`: HTTP handlers for login, setup wizard, dashboard metrics calculation, pagination, search, and logout.

## 4. Architecture & Core Components

### Database Schema (`SQLite`)
1. **`log_entries`**: Stores parsed log rows.
   * `id` (INTEGER PRIMARY KEY AUTOINCREMENT)
   * `domain` (TEXT)
   * `ip` (TEXT)
   * `timestamp` (TEXT)
   * `method` (TEXT)
   * `path` (TEXT)
   * `status` (INTEGER / `i64`)
   * `size` (INTEGER / `i64`)
   * `referer` (TEXT)
   * `user_agent` (TEXT)
2. **`log_states`**: Tracks incremental read progress and logrotate safety per log file.
   * `path` (TEXT PRIMARY KEY) - File path
   * `inode` (INTEGER NOT NULL) - File system inode identifier for logrotate/rename detection
   * `last_position` (INTEGER NOT NULL) - Byte offset of the last read position
3. **`users`**: Stores administrator credentials after the initial setup.
   * `id` (TEXT PRIMARY KEY) - UUIDv7 identifier
   * `username` (TEXT UNIQUE NOT NULL) - Forced to `admin`
   * `password_hash` (TEXT NOT NULL) - Argon2 hashed password
   * `role` (TEXT NOT NULL DEFAULT 'admin')

### Incremental Log Parsing (`Inode + Byte-Offset Tracking`)
- Protected against logrotate, file truncation, and restarts.
- The parser fetches file metadata (including Unix inode via `std::os::unix::fs::MetadataExt`) and compares it against `log_states`.
- If the inode changes (logrotate happened) or remains the same, it seeks directly to the correct byte offset, reads only new lines, inserts them into `log_entries`, and safely updates the state.

### Real-Time Ingestion & Retention (`Background Worker`)
- Managed by a Tokio background task (`tokio::spawn`) running independently of HTTP requests.
- Checks log sources every 30 seconds for new entries.
- Automatically cleans up old logs every ~1 hour (120 cycles) by deleting records older than 30 days (`DELETE FROM log_entries WHERE timestamp < datetime('now', '-30 days')`).

### Authentication & Setup Workflow
- **First-Run Setup Flow:** On initial launch (checked via `users` table count), the system prompts for a provisional password (`PROVISIONAL_PASS` from `.env`, defaulting to `power`). Upon successful provisional login, the user is redirected to `/setup` where they must define and confirm a permanent password. This user is stored in SQLite as `admin` with an Argon2 hash and a UUIDv7 ID.
- **Protected route:** `/dashboard`
- **Login route:** `/` (GET/POST)
- **Setup route:** `/setup` (GET/POST)
- **Logout route:** `/logout` (POST)
- Session-based feedback: Failed login/setup attempts store an error message in the session, rendered safely in templates and cleared immediately after.

### Pagination & Search
- Paginated views (default: 10 entries per page).
- Dynamic search/filtering matching `ip`, `path`, `timestamp`, or `method` using SQL `LIKE`.

## 5. Configuration (`.env`)
- `PORT`: Web server listening port (default: `3000`)
- `DOMAIN_NAME`: Target domain for deployment and Nginx/Certbot (default: `analytics.example.com`)
- `DATABASE_URL`: `sqlite://nginx_analytics.db?mode=rwc`
- `PROVISIONAL_PASS`: Temporary password for the first-run login (default: `power`)
- `CSS_PATH`: Path to stylesheet (`/static/css/style.css`)
- `JS_PATH`: Path to frontend script (`/static/js/app.js`)
- `LOG_SOURCES`: Maps domain names to log file paths (e.g., `default=access.log,api=api_access.log`)

## 6. Current Progress & Working State
- Refactored cleanly into standard Rust modular architecture.
- Secure onboarding with provisional-to-permanent password migration via Argon2 and UUIDv7.
- Robust state management with Inode tracking and automated 30-day data retention.

## 7. Future Improvement Ideas (Next Steps)
- Export reports to CSV/JSON.
- Advanced alert triggers for high error rates (status >= 500).
- Install script.
