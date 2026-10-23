# Rust Backend REST API

A modern, robust, and extensible REST API backend built in Rust using **Axum**, **Tokio**, and **SQLite** (via **SQLx**), with built-in **API Gateway** capabilities.

## Features

- ⚡ **High Performance & Async**: Built on Tokio and Axum 0.7.
- 🗄️ **Persistent Database**: SQLite with connection pooling (`sqlx::SqlitePool`) and automatic database migration on startup.
- 🛡️ **API Gateway Layer**:
  - **Auth Guard**: Rejects unauthorized access with `401 Unauthorized` for missing/invalid API keys (`Authorization: Bearer <key>` or `X-API-Key: <key>`).
  - **CORS**: Configurable cross-origin resource sharing via `tower_http::cors::CorsLayer`.
  - **Request Tracing**: Structured logging for incoming requests and status responses via `tower_http::trace::TraceLayer`.
- 🩺 **Health Monitoring**: `GET /health` endpoint verifying both server responsiveness and database connectivity.
- 🔄 **Graceful Shutdown**: Handles `SIGINT` (Ctrl+C) and `SIGTERM` signals cleanly.
- 🧪 **Full Integration Tests**: Fast in-memory SQLite integration tests.

---

## Architecture Overview

```
├── Cargo.toml            # Project dependencies and workspace config
├── .env.example          # Template environment configuration
├── migrations/           # SQLx automated database schema migrations
│   └── 20261024000000_create_items_table.sql
├── src/
│   ├── main.rs           # Application entrypoint & graceful shutdown
│   ├── lib.rs            # Library entrypoint for tests & modularity
│   ├── config.rs         # Environment variable configuration loader
│   ├── db.rs             # SQLite connection pool and automatic migration
│   ├── error.rs          # Centralized error handling (AppError -> HTTP JSON)
│   ├── models.rs         # Domain entities & request/response DTOs
│   ├── state.rs          # Thread-safe shared application state
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs       # Gateway Authentication Guard middleware
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── health.rs     # /health endpoint with database ping
│   │   └── items.rs      # Full CRUD handlers with parameterized queries
│   └── routes/
│       └── mod.rs        # Route definitions, nested protected routes, & CORS
└── tests/
    └── api_tests.rs      # Integration test suite
```

---

## Getting Started

### 1. Environment Configuration

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

Default values in `.env`:
```ini
HOST=0.0.0.0
PORT=3000
DATABASE_URL=sqlite://app.db?mode=rwc
API_KEY=dev-secret-api-key
RUST_LOG=rust_backend_api=debug,tower_http=debug
```

### 2. Run the Application

```bash
cargo run
```

The database (`app.db`) and tables will be created and migrated automatically.

---

## API Documentation

### Public Endpoints

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/` | Basic server welcome response |
| `GET` | `/health` | Health check verifying server and SQLite connectivity |

#### Health Check Example
```bash
curl -i http://localhost:3000/health
```
Response:
```json
{
  "status": "healthy",
  "database": "connected",
  "timestamp": "2026-10-23T19:40:00Z"
}
```

---

### Protected Endpoints (`/api/items`)
All routes under `/api/*` require authentication via either:
- Header: `Authorization: Bearer <API_KEY>`
- Header: `X-API-Key: <API_KEY>`

| Method | Endpoint | Description | Request Body |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/items` | List all items (newest first) | None |
| `POST` | `/api/items` | Create a new item | `{"title": "string"}` |
| `GET` | `/api/items/:id` | Get item by UUID | None |
| `PUT` | `/api/items/:id` | Update item title/status | `{"title": "...", "completed": true}` |
| `DELETE` | `/api/items/:id` | Delete an item | None |

#### Examples with `curl`

**1. Create an Item:**
```bash
curl -i -X POST http://localhost:3000/api/items \
  -H "Authorization: Bearer dev-secret-api-key" \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Rust and Axum"}'
```

**2. List All Items:**
```bash
curl -i http://localhost:3000/api/items \
  -H "Authorization: Bearer dev-secret-api-key"
```

**3. Update Item Completion Status:**
```bash
curl -i -X PUT http://localhost:3000/api/items/<ITEM_ID> \
  -H "Authorization: Bearer dev-secret-api-key" \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

**4. Delete an Item:**
```bash
curl -i -X DELETE http://localhost:3000/api/items/<ITEM_ID> \
  -H "Authorization: Bearer dev-secret-api-key"
```

---

## Running Automated Tests

Run the full integration test suite with:

```bash
cargo test
```
