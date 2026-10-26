# Technical Specification and Project Plan: Nginx Analytics Dashboard
* **Version:** 1.0.0-ISO
* **Status:** Development
* **Reference Standards:** ISO/IEC 25010 (Product Quality) | ISO/IEC/IEEE 12207 (Software Life Cycle)

---

## 1. Project Overview
A high-performance monolithic web application written in **Rust**, designed to collect, asynchronously process, and display traffic metrics from **Nginx** log files. The system prioritizes memory safety, data isolation via SQLite in WAL mode, and a modern, auditable UI experience.

---

## 2. Technology Stack
* **Backend:** Rust, Axum (Web Framework), SQLx (Async SQL Toolkit)
* **Database:** SQLite (Local single-file storage with WAL mode enabled)
* **Session Management:** Tower-Session (Encrypted and secure cookies)
* **Templates:** Askama (Type-safe template engine)
* **Frontend Assets & Bundler:** **Bun** (for minification, bundling, and CSS/JS cache-busting)
* **Design & Icons:** Lucid Icons, SimpleIcons

---

## 3. System Requirements and Traceability Matrix (RTM)

### Functional Requirements (FR)
* **FR01 (Initial Bootstrap):** Reading `SETUP_USER` and `SETUP_PASS` from `.env` on the first execution to automatically create the administrator if the database is empty.
* **FR02 (Initial Password Security):** Assignment of a temporary password flag; blocking and mandatory redirection for password change on first login.
* **FR03 (Session Management):** Control of authentication state via encrypted session cookies.
* **FR04 (User Management - RBAC):** Exclusive administrative panel for user CRUD operations.
* **FR05 (Log Configuration and Domains):** Support for default logs or domain-specific custom logs (`access_log`), with a dynamic selection interface.
* **FR06 (Processing and Metrics):** Reading log files to calculate daily visits and graphic history.

### Non-Functional Requirements (NFR)
* **NFR01 (Performance and Concurrency):** Pure Rust backend, ensuring zero Garbage Collection pauses and minimal RAM usage.
* **NFR02 (Reliability and Data):** Resilient SQLite database with ACID transaction support via SQLx.
* **NFR03 (Code Quality & Product Quality):** Strict compliance with ISO/IEC 25010 guidelines (maintainability, reliability, and security).
* **NFR04 (Asset Management):** Automated frontend compilation via Bun with integrity hashes.
* **NFR05 (Automated Testing):** Mandatory unit test coverage for the log parser and integration tests for HTTP workflows.
* **NFR06 (AI Governance - `AGENT.md`):** Mandatory maintenance of an `AGENT.md` file in the repository root to ensure consistency in AI-assisted modifications.

---

## 4. Use Cases (UC)

| ID | Use Case Name | Primary Actor | Pre-condition | Summary Description |
| :--- | :--- | :--- | :--- | :--- |
| **UC01** | System Bootstrap | System / DevOps | `.env` configured, empty DB | Automatic creation of the admin based on environment variables. |
| **UC02** | Admin Onboarding | Initial Admin | Login with temporary pass | Validation of reset flag and submission of new permanent password. |
| **UC03** | Traffic Monitoring | Authenticated User | Valid session | Domain selection in dropdown, log parsing, and updated dashboard rendering. |
| **UC04** | User Management | Administrator | Active Admin session | Execution of CRUD operations on the users table. |

---

## 5. Directory Structure and Components

```text
├── documents/
│   └── technical_specification.md  # [This technical specification document]
├── .env.example
├── Cargo.toml
├── package.json
├── AGENT.md                        # Strict operational manual for AI Agents
├── db/
│   └── migrations/                 # Versioned SQLx migrations
├── frontend/
│   ├── css/
│   ├── js/
│   └── assets/                     # Lucid Icons and SimpleIcons
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs                    # Centralized error handling (IntoResponse)
│   ├── db.rs
│   ├── models/                     # User, LogEntry, Domain
│   ├── services/
│   │   ├── auth.rs
│   │   └── nginx_parser.rs         # Optimized log reading engine
│   ├── handlers/                   # Axum controllers (auth, dashboard, users)
│   └── middleware/                 # Session guards and temporary flags
├── templates/                      # Askama Type-safe Templates
│   ├── layouts/
│   │   └── base.html               # Global layout with hash-versioned assets
│   ├── components/                 # Reusable partials (navbar, stat_card, etc.)
│   └── pages/                      # Final views (login, dashboard, users)
├── static/                         # Bun compiled output
└── tests/                          # Integration tests (API and Parser)
    ├── auth_tests.rs
    └── parser_tests.rs

```

---

## 6. Operational Procedure for AI (`AGENT.md`)

The `AGENT.md` file located at the root acts as the primary immutable instruction set for any AI Agent interacting with the repository, stipulating:

1. **Mandatory Commands:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `bun run build`.
2. **Architectural Conventions:** Absolute prohibition of `unsafe` blocks without formal technical justification in code comments; obligation to use typed errors in `src/error.rs`.
3. **Commit Pattern:** Strict use of Conventional Commits associated with requirement IDs (e.g., `feat(parser): implement line streaming [FR06]`).

---

## 7. Quality Gates and Risk Management (ISO/IEC 25010)

* **Build Quality Gate:** 0 warnings allowed in the compiler and linter (`cargo clippy`).
* **Test Quality Gate:** All unit and integration tests must pass successfully before any Continuous Integration (CI) run.
* **Log I/O Risk Management:** To mitigate memory failures with massive log files, the `nginx_parser.rs` service must process files in batches (*streaming buffers*), avoiding loading the full history into RAM at once.
