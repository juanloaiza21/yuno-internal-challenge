# AGENTS.md — Project Root

## Project Overview

**Yuno Internal Challenge** — A Rust API deployed as serverless functions on Vercel using the `vercel-community/rust` community runtime.

- **Language:** Rust (edition 2021)
- **Runtime:** `vercel-community/rust@latest`
- **Platform:** Vercel Serverless Functions
- **Crate name:** `yuno-internal-challenge`

---

## Architecture

```
yuno-internal-challenge/
├── api/              # Serverless function handlers (one .rs file = one endpoint)
│   └── health.rs     # GET /api/health
├── src/
│   └── lib.rs        # Shared library crate (business logic, models, utilities)
├── Cargo.toml        # Workspace manifest with [lib] + [[bin]] entries
├── vercel.json       # Vercel routing and runtime configuration
└── AGENTS.md         # This file
```

### Serverless Model

Each `.rs` file inside `api/` is declared as a `[[bin]]` in `Cargo.toml` and compiled into an independent serverless function by the Vercel Rust runtime. Every handler:

1. Defines a `#[tokio::main] async fn main()` that calls `vercel_runtime::run(handler)`.
2. Implements a `handler` function with signature `async fn(Request) -> Result<Response<Body>, Error>`.
3. Imports shared logic from the library crate (`yuno_internal_challenge::*`) — handlers stay thin, logic stays in `src/`.

### Adding a New Endpoint

1. Create `api/<name>.rs` with the handler boilerplate.
2. Add a `[[bin]]` entry in `Cargo.toml` pointing to the new file.
3. Add a rewrite rule in `vercel.json` if the route needs customization.
4. Put any business logic in `src/lib.rs` (or submodules) and import it from the handler.

---

## Code Conventions

### Rust Idioms

- Use `Result<T, E>` for all fallible operations. Propagate errors with `?`.
- Prefer `impl` blocks and associated functions over free-standing functions when grouping behavior.
- Use `serde` derive macros (`Serialize`, `Deserialize`) for all data structures that cross the API boundary.
- Keep handlers in `api/` as thin dispatchers — no business logic, no direct data access.
- All shared code lives in `src/lib.rs` (and its submodules). This is the library crate.

### Clean Code

- Functions do one thing. Name them after what they do.
- No dead code, no commented-out blocks, no TODO comments without a tracking issue.
- Every public item has a doc comment (`///`).
- Keep files short and focused. Split into modules when a file exceeds ~150 lines.

### Error Handling

- Return structured JSON error responses with appropriate HTTP status codes.
- Use `vercel_runtime::Error` as the error type in handlers.
- Map domain errors to HTTP responses explicitly — do not leak internal details.

### Formatting and Linting

- Run `cargo fmt` before every commit.
- Run `cargo clippy` and resolve all warnings before every commit.

---

## Commit Strategy

This project follows the **Gitmoji** commit convention with strict traceability requirements.

### Gitmoji Standard

Every commit message starts with a gitmoji that classifies the change:

| Gitmoji | Meaning                      |
|---------|------------------------------|
| 🎉      | Initial commit / project init |
| ✨      | New feature                   |
| 🐛      | Bug fix                       |
| 📝      | Documentation                 |
| 🔧      | Configuration / tooling       |
| ♻️      | Refactor                      |
| ✅      | Tests                         |
| 🚀      | Deployment                    |
| 🔥      | Remove code / files           |
| 🏗️      | Architectural changes         |
| 💄      | UI / cosmetic changes         |
| 🔒      | Security fix                  |

### Rules

- **One commit per meaningful change.** Each commit is atomic and reviewable in isolation.
- **Descriptive messages.** Format: `<gitmoji> <imperative verb> <what changed>`. Example: `✨ Add health check endpoint`.
- **Interviewer traceability.** The commit history tells the story of how the project was built, step by step. Every commit should make sense on its own.

---

## Folder AGENTS.md Policy

Every folder in this project has its own `AGENTS.md` file that describes:

- The folder's purpose and contents.
- Conventions specific to that folder.
- Key files and their roles.

### Maintenance Rule

**When any file in a folder is added, removed, or significantly changed, that folder's `AGENTS.md` must be updated in the same commit.** This keeps documentation synchronized with the code at all times.

---

## Dependencies

Defined in `Cargo.toml`:

| Crate            | Version | Purpose                                              |
|------------------|---------|------------------------------------------------------|
| `vercel_runtime` | 1       | Vercel serverless function SDK (request/response, runtime entry point) |
| `tokio`          | 1 (full)| Async runtime required by `vercel_runtime` handlers  |
| `serde`          | 1 (derive)| Serialization/deserialization with derive macros    |
| `serde_json`     | 1       | JSON parsing and generation for API payloads         |

When adding a dependency:
- Pin to a major version (`"1"`) unless a specific minor/patch is required.
- Document the reason in this table.
- Prefer well-maintained, minimal crates. Avoid pulling large dependency trees for small tasks.

---

## Deployment

### How It Works

1. Vercel detects `vercel.json` and the `vercel-community/rust@latest` runtime configuration.
2. Each `api/*.rs` file matching the `"api/**/*.rs"` glob is compiled into a standalone serverless function.
3. The Rust runtime handles compilation using the `Cargo.toml` manifest — both the shared `[lib]` and each `[[bin]]` target.
4. Functions are deployed as AWS Lambda–compatible binaries behind Vercel's edge network.

### Route Mapping

- By default, `api/health.rs` maps to `GET /api/health`.
- Custom routing is configured via the `"rewrites"` array in `vercel.json`.

### Environment Variables

- Managed through the Vercel dashboard or `.env` files (never committed — see `.gitignore`).
- Access in Rust via `std::env::var("VAR_NAME")`.

### Local Development

```bash
# Build and check the project
cargo build
cargo clippy
cargo test

# For local Vercel emulation (requires Vercel CLI)
vercel dev
```
