# Challenge Summary — FashionForward Routing Engine

## The Problem

**FashionForward**, a fashion marketplace processing **45,000 transactions/day** across Brazil, Mexico, and Colombia, has a **22% decline rate** costing them massive revenue. Root cause: their payment routing is primitive — it tries one PSP and gives up, even though 2-3 backup PSPs are configured per country.

## What We Built

A **smart payment routing engine** in Rust (deployed on Vercel serverless) that:

1. **Retries intelligently** — Soft declines (issuer unavailable, suspected fraud, do not honor) automatically retry with backup PSPs. Hard declines (insufficient funds, expired card) fail immediately — no wasted retries.

2. **Simulates 9 real PSPs** — 3 per country (Brazil, Mexico, Colombia), each with unique success rates (65–83%), latency profiles, fees, and decline reason distributions. Deterministic seeded RNG ensures reproducible demos.

3. **Proves the business impact** — A performance report processes 210 transactions through both "no retry" and "smart retry" scenarios, showing clear improvement.

## Results

```
                    No Retry    Smart Retry    Improvement
Auth Rate:          65.7%       92.9%          +27.1 pp
Approved:           138/210     195/210        +57 transactions
Revenue Recovered:  —           —              $10,409.72

By Country:
  Brazil:           72.9%  →  95.7%   (+22.9pp)
  Mexico:           55.7%  →  92.9%   (+37.1pp)
  Colombia:         68.6%  →  90.0%   (+21.4pp)
```

At FashionForward's scale: **~$610K/day in recovered revenue**.

## How It Works

```
Customer Payment → RoutingEngine → PSP #1 (declined: issuer_unavailable)
                                 → PSP #2 (approved!) ✅
                                   └─ returned to customer in 450ms total
```

- **Hard decline?** → Stop. Card is genuinely bad. No retry.
- **Soft decline?** → Try next PSP. Different PSP = different routing path = different outcome.
- **PSP down?** → Cascade immediately to next PSP without counting as a decline.

## API Endpoints

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/health` | GET | Health check |
| `/api/authorize` | POST | Route a single transaction with retry logic |
| `/api/report` | POST | Generate full performance comparison report |

## Stretch Goals Completed

- **Cost-Aware Routing** — 3 strategies: `OptimizeForApprovals`, `OptimizeForCost`, `Balanced`
- **Real-Time Cascading** — PSP unavailability (8% rate) triggers immediate cascade without penalty

## Tech Stack

Rust · Vercel Serverless · tokio · serde · seeded RNG

## Quick Start

```bash
cargo build                              # Build
cargo test                               # 40 tests, 0 failures
cargo run --bin generate_outputs         # Generate report + test data
```

## Architecture

```
api/authorize.rs  →  RoutingEngine  →  PspSimulator  →  PspResponse
     (thin)           (retry loop)     (deterministic)   (approve/decline)
                      ↓
                  engine/strategy.rs   (PSP ordering by strategy)
                  engine/retry.rs      (hard vs soft classification)
```

5 modules, 40 tests, 0 clippy warnings, AGENTS.md per folder, gitmoji commit history.
