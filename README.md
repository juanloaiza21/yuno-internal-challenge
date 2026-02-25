# Yuno Internal Challenge

Rust API deployed as serverless functions on [Vercel](https://vercel.com) using the [`vercel-community/rust`](https://github.com/vercel-community/rust) runtime.

## Tech Stack

| Layer        | Technology                                      |
| ------------ | ----------------------------------------------- |
| Language     | Rust (2021 edition)                             |
| Runtime      | `vercel-community/rust@latest`                  |
| Async        | tokio                                           |
| Serialization| serde + serde_json                              |
| Platform     | Vercel Serverless Functions                     |

## Project Structure

```
.
├── api/
│   └── health.rs          # GET /api/health — serverless handler
├── src/
│   └── lib.rs             # Shared library (business logic, models, utils)
├── Cargo.toml             # Workspace manifest & dependencies
├── Cargo.lock
├── vercel.json            # Vercel routing & runtime config
└── README.md
```

- **`api/`** — Each `.rs` file is compiled into an independent serverless function. One file = one endpoint.
- **`src/lib.rs`** — Shared crate imported by all handlers. Keeps handlers thin and logic reusable.

## Challenge

> **TBD** — Challenge description will be added here.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Vercel CLI](https://vercel.com/docs/cli) (`npm i -g vercel`)

### Build

```bash
# Type-check the project
cargo check

# Full build
cargo build
```

### Run Locally

```bash
# Start the Vercel dev server (builds & serves Rust functions locally)
vercel dev
```

The API will be available at `http://localhost:3000`.

## API Endpoints

| Method | Path          | Description                          |
| ------ | ------------- | ------------------------------------ |
| GET    | `/api/health` | Health check — returns status & version |

**Example response:**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## Deployment

Vercel handles builds and deployments automatically:

1. **Git push** — Every push to the connected branch triggers a deploy.
2. **Manual** — Run `vercel` (preview) or `vercel --prod` (production) from the project root.

The `vercel.json` config tells Vercel to compile every `api/**/*.rs` file using the `vercel-community/rust` runtime. Each file becomes its own serverless function at the matching URL path.
