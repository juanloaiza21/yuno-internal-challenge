# api/ — Vercel Serverless Function Handlers

## Purpose

This directory contains Vercel serverless function entry points. Each `.rs` file compiles to an independent binary that Vercel deploys as an HTTP endpoint. These are **thin wrappers** — all business logic lives in the `yuno_internal_challenge` library crate (`src/lib.rs`).

## Request Routing

The URL path is derived directly from the filename:

| File | Endpoint |
|------|----------|
| `api/health.rs` | `GET /api/health` |
| `api/foo.rs` | `/api/foo` |
| `api/bar.rs` | `/api/bar` |

There is no router. Each file is a standalone binary mapped 1:1 to a path.

## Current Handlers

- **`health.rs`** — `GET /api/health`. Returns JSON with service status and version. Used for uptime checks and deployment verification.

## Handler Pattern

Every handler must follow this exact boilerplate:

```rust
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // handler logic here
}
```

Key imports from `vercel_runtime`: `run`, `Body`, `Error`, `Request`, `Response`, `StatusCode`.

## Conventions

- **Thin handlers.** The handler function should only parse the request, call into `yuno_internal_challenge` for business logic, and format the response. No domain logic in this directory.
- **Import from the library crate.** Use `use yuno_internal_challenge::...` to access shared logic, types, and utilities defined in `src/`.
- **JSON responses.** Return `Content-Type: application/json` for all endpoints. Build the response body with `serde_json`.
- **Proper status codes.** Use `StatusCode::OK` (200) for success, `StatusCode::BAD_REQUEST` (400) for client errors, `StatusCode::INTERNAL_SERVER_ERROR` (500) for failures, etc. Never hardcode numeric status values when a named constant exists.
- **Error handling.** Return structured JSON error bodies (`{"error": "message"}`) rather than plain text. Propagate errors with `?` where appropriate.

## Adding a New Endpoint

1. Create `api/<name>.rs` following the handler pattern above.
2. Add a `[[bin]]` entry in the workspace `Cargo.toml`:
   ```toml
   [[bin]]
   name = "<name>"
   path = "api/<name>.rs"
   ```
3. Import any needed logic from `yuno_internal_challenge` — add new public functions/modules in `src/` if the endpoint requires new business logic.
4. The endpoint will be available at `/api/<name>` after deployment.
