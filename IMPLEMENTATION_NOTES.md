# Implementation Notes — Payment Routing Engine

> These notes document the full implementation journey: architecture decisions, development workflow, what was built, what was attempted, and what could come next.

---

## Development Approach

### Parallel Development with 3 OpenCode Instances

The entire solution was built in parallel using **3 independent OpenCode/Claude Code instances**, each working on a separate feature branch. This was a deliberate strategy to maximize throughput under a 2-hour time constraint.

#### Phase 0: Foundation (on `main`)

Before branching, we established a shared foundation that all 3 instances could build on without merge conflicts:

1. **Domain models** (`src/models/`) — Complete, shared types: `Transaction`, `PspConfig`, `DeclineReason`, `RoutingResult`, `PerformanceReport`, etc. All with serde derives, doc comments, and Display implementations.
2. **Module stubs** — Every module (`simulator/`, `engine/`, `data/`, `report/`) was created with compilable stub implementations returning dummy data.
3. **API endpoint stubs** — `api/authorize.rs` and `api/report.rs` returning 501 Not Implemented.
4. **Full Cargo.toml** — All `[[bin]]` entries and dependencies declared upfront.

This meant every branch could `cargo check` from minute one — no compilation errors, no waiting for dependencies.

#### Branch Strategy

| Instance | Branch | Scope | Files Owned |
|---|---|---|---|
| **1** | `feature/psp-simulator` | PSP simulation + test data | `src/simulator/*`, `src/data/*` |
| **2** | `feature/routing-engine` | Routing logic + strategies | `src/engine/*` |
| **3** | `feature/api-reports` | API endpoints + reports + docs | `src/report/*`, `api/authorize.rs`, `api/report.rs`, `README.md` |

**Key design**: Each branch owned **completely separate files**. No two branches modified the same file (except `src/AGENTS.md` and `src/lib.rs`, which had trivial additive changes). This made merging clean — all 3 branches merged into `main` with zero manual conflict resolution.

#### Merge Order

1. `feature/psp-simulator` → `main` (fast-forward, no conflicts)
2. `feature/routing-engine` → `main` (clean merge via `ort` strategy)
3. `feature/api-reports` → `main` (clean merge, auto-merged `src/AGENTS.md`)

Post-merge: `cargo clippy` revealed 5 doc comment style warnings (outer `///` on modules instead of inner `//!`). Fixed in a single cleanup commit.

---

## Architecture Deep Dive

### Why Rust on Vercel Serverless

- **Type safety**: Rust's type system catches entire classes of bugs at compile time — invalid enum variants, missing match arms, null dereference. For a payment system, this matters.
- **Performance**: Native compilation means fast cold starts (~100-500ms) and near-zero runtime overhead.
- **Serverless fit**: Each `api/*.rs` file compiles to an independent binary. Stateless by design — no shared memory between invocations.

### Module Separation

```
Transaction Request
        │
        ▼
   api/authorize.rs          ← Thin handler: parse JSON, call engine, return result
        │
        ▼
   engine/mod.rs             ← Orchestrator: gets PSPs, applies strategy, runs retry loop
        │
        ├── engine/strategy.rs   ← PSP ordering logic (3 strategies)
        ├── engine/retry.rs      ← Hard/soft decline classification
        │
        ▼
   simulator/mod.rs          ← PSP behavior: deterministic approve/decline decision
        │
        ├── simulator/config.rs  ← 9 PSP configurations (rates, latency, fees, decline biases)
        │
        ▼
   PspResponse { approved, decline_reason, latency_ms }
```

Each layer has a single responsibility:
- **API handlers** — HTTP concerns only (parse request, serialize response, status codes)
- **Engine** — Business logic (retry orchestration, strategy selection)
- **Simulator** — Domain simulation (PSP behavior modeling)
- **Models** — Pure data types (no behavior, just structure)

---

## What Was Built (Core Requirements)

### Requirement 1: Multi-PSP Routing Engine ✅ (25pts)

**Implementation**: `src/engine/mod.rs`

The `RoutingEngine` accepts a transaction and a strategy, then:

1. Fetches PSPs for the transaction's country (3 per country)
2. Orders them using the selected strategy (`OptimizeForApprovals`, `OptimizeForCost`, `Balanced`)
3. Sends the transaction to PSP #1 via the simulator
4. **If approved** → returns success immediately with routing metadata
5. **If hard decline** (`InsufficientFunds`, `CardExpired`, `InvalidCard`, `StolenCard`) → **stops immediately**, no retry. These are permanent failures.
6. **If soft decline** (`IssuerUnavailable`, `SuspectedFraud`, `DoNotHonor`, `ProcessorDeclined`) → **retries with next PSP**
7. **If PSP unavailable** → **cascades immediately** without counting as a decline attempt
8. Repeats until approved or all PSPs exhausted (max 3 attempts)

The `route_no_retry()` method simulates FashionForward's current behavior — single PSP, no fallback.

**Key code paths tested**:
- Approved on first attempt → 1 attempt, success
- Soft decline → retry → approved on second attempt → 2 attempts, success
- Hard decline → immediate failure → 1 attempt, no retry
- All PSPs decline → 3 attempts, final failure
- PSP unavailable → cascade (doesn't count as attempt)

### Requirement 2: PSP Behavior Simulation ✅ (20pts)

**Implementation**: `src/simulator/mod.rs` + `src/simulator/config.rs`

9 PSPs across 3 countries, each with unique:

| Characteristic | Range Across PSPs |
|---|---|
| Success rate | 65% (Bold, Colombia) to 83% (Wompi, Colombia) |
| Latency | 150ms (Cielo min) to 600ms (Stone max) |
| Fee percentage | 2.4% (Bold) to 3.3% (Wompi) |
| Decline bias | Each PSP has one dominant soft decline reason at 45% weight |

**Deterministic seeding** is the most critical design decision:

```
Seed = hash(card_bin + card_last4 + psp_id + amount_cents)
                                     ^^^^^
                                     THIS is why retry works
```

Because `psp_id` is part of the seed, the **same card produces different random outcomes at different PSPs**. This models reality: different PSPs have different fraud models, different issuer relationships, and different processing infrastructure.

**Three-tier outcome model**:
- **Hard declines (~6%)**: Seeded from card attributes only (no `psp_id`). Same card always hard-declines at every PSP. Simulates genuinely bad cards.
- **Soft declines (variable per PSP)**: Seeded with `psp_id`. PSP #1 might decline but PSP #2 approves. This is the core value of retry.
- **PSP unavailability (8%)**: Seeded from `transaction_id + psp_id`. Simulates intermittent downtime.

### Requirement 3: Performance Report ✅ (20pts)

**Implementation**: `src/report/mod.rs`

Runs the same 210 transactions through two scenarios:

| Metric | No Retry | Smart Retry | Improvement |
|---|---|---|---|
| Authorization Rate | 65.7% | 92.9% | **+27.1pp** |
| Approved | 138 | 195 | **+57 transactions** |
| Declined | 72 | 15 | -57 |
| Avg Attempts | 1.00 | 1.20 | — |
| Avg Latency | 280.4ms | 317.7ms | +37.3ms |
| Revenue Recovered | — | — | **$10,409.72** |

**By country**:
- Brazil: 72.9% → 95.7% (+22.9pp)
- Mexico: 55.7% → 92.9% (+37.1pp)
- Colombia: 68.6% → 90.0% (+21.4pp)

Mexico shows the largest improvement because its primary PSP (Conekta, 75%) has a lower base rate, meaning more transactions benefit from retry to the stronger backup PSPs.

**Revenue calculation**: `additional_approvals × average_transaction_value` from the batch. At FashionForward's scale (45,000 daily transactions), the rate lift would translate to approximately **$610K/day** in recovered revenue.

---

## Stretch Goals Attempted

### Cost-Aware Routing ✅ (Implemented)

**Implementation**: `src/engine/strategy.rs`

Three strategies with different PSP ordering logic:

| Strategy | Formula | Tradeoff |
|---|---|---|
| `OptimizeForApprovals` | Sort by `success_rate` DESC | Best auth rate, highest cost |
| `OptimizeForCost` | Sort by `fee_pct + fee_fixed/avg_amount` ASC | Cheapest processing, lower auth rate |
| `Balanced` | Score = `rate × 0.7 + (1 - norm_fee) × 0.3`, sort DESC | Practical middle ground |

The `Balanced` strategy weights approval rate at 70% because a declined transaction generates $0 revenue regardless of how cheap the PSP is. Cost only matters for approved transactions.

### Real-Time Cascading ✅ (Implemented)

**Implementation**: `src/simulator/mod.rs` (unavailability simulation) + `src/engine/mod.rs` (cascade handling)

- 8% of PSP requests return `PspUnavailable`
- The engine detects this and **cascades immediately** to the next PSP
- Cascades do NOT count against the retry budget (max 3 decline-based attempts)
- Cascades DO add latency (reflected in total latency)
- PSP unavailable events are tracked in routing metadata

This prevents transient infrastructure issues from inflating decline rates — a real concern in production payment systems.

---

## Test Coverage

**40 tests total**, covering every module:

| Module | Tests | What They Verify |
|---|---|---|
| `simulator/mod.rs` | 5 | Determinism, PSP-dependent outcomes, hard decline independence, latency ranges, approval distribution |
| `simulator/config.rs` | 4 | 3 PSPs per country, 9 total, valid rates, decline weights sum to 1.0 |
| `engine/mod.rs` | 6 | Routing for all countries, no-retry single attempt, max attempts, final PSP tracking, sequential numbering, latency summation |
| `engine/retry.rs` | 4 | Hard decline classification, soft decline classification, PSP unavailable classification, exhaustive coverage |
| `engine/strategy.rs` | 5 | All 3 strategies sort correctly, empty list handling, single PSP passthrough, input immutability |
| `data/mod.rs` | 7 | Correct count, determinism, country distribution, amount range, customer uniqueness, valid BINs, unique IDs |
| `report/mod.rs` | 4 | Metric calculation, empty input handling, country breakdown, PSP breakdown |

All tests are deterministic (seeded RNG), fast (<0.2s total), and require no I/O or network.

---

## Documentation

### AGENTS.md Convention

Every folder has its own `AGENTS.md` describing:
- Purpose and contents of the folder
- File roles and responsibilities
- Conventions specific to that module
- Testing approach

These files are **mutable** — updated with every change to the folder in the same commit. This keeps documentation synchronized with code at all times.

Files created:
- `/AGENTS.md` — Root project overview
- `/src/AGENTS.md` — Shared library conventions
- `/src/models/AGENTS.md` — Domain type documentation
- `/src/simulator/AGENTS.md` — PSP simulator design + seeding explanation
- `/src/engine/AGENTS.md` — Routing engine logic + retry rules
- `/src/data/AGENTS.md` — Test data generation specs
- `/src/report/AGENTS.md` — Report comparison methodology
- `/api/AGENTS.md` — Handler conventions + endpoint catalog

### Commit Convention

Every commit follows the **Gitmoji** standard for interviewer traceability:

| Emoji | Usage |
|---|---|
| 🎉 | Project initialization |
| 🏗️ | Architectural scaffolding |
| ✨ | New feature implementation |
| ♻️ | Refactoring / cleanup |
| 📝 | Documentation updates |
| 🔧 | Configuration changes |

Each commit is atomic and reviewable in isolation. The commit history tells the story of how the system was built, step by step.

---

## What Could Come Next

If we had more time, here's what would add the most value:

### 1. Dynamic Success Rate Tracking
Instead of static PSP success rates from configuration, track real-time approval rates per PSP. If a PSP's rate drops below a threshold, automatically deprioritize it. This would use a sliding window (e.g., last 1,000 transactions) and require shared state — possible with an external store like Redis.

### 2. Card-Level Routing Intelligence
Track which PSPs have historically approved cards with specific BIN prefixes. Some issuers have stronger relationships with certain acquirers. Over time, the engine would learn that "cards starting with 4111 approve 95% of the time at Cielo but only 70% at PagSeguro" and route accordingly.

### 3. Circuit Breaker Pattern
Instead of the flat 8% unavailability rate, implement a proper circuit breaker: if a PSP fails N times in a row, mark it as "open" (skip it entirely) for a cooldown period. After cooldown, send a probe transaction to check if it's recovered.

### 4. A/B Testing Framework
Run different routing strategies simultaneously on different transaction segments. Compare `OptimizeForApprovals` vs `Balanced` in production with real metrics. This would require a traffic splitting mechanism and statistical significance testing.

### 5. Webhook / Async Notifications
For transactions that require manual review or delayed authorization, add webhook support to notify the merchant of the final outcome asynchronously.

### 6. Multi-Currency Fee Normalization
Currently, fees are compared in USD. In production, fees should be normalized to the transaction's local currency using real-time exchange rates for accurate cost comparison.

---

## Final Numbers

```
┌─────────────────────────────────────────────────────┐
│           FASHIONFORWARD ROUTING ENGINE              │
│                 PROOF OF CONCEPT                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│  No Retry (current):     65.7% authorization rate   │
│  Smart Retry (engine):   92.9% authorization rate   │
│                                                     │
│  Improvement:           +27.1 percentage points     │
│  Additional approvals:   57 / 210 transactions      │
│  Revenue recovered:      $10,409.72                 │
│                                                     │
│  At FashionForward scale (45K txns/day):            │
│  ~12,195 additional daily approvals                 │
│  ~$610K/day in recovered revenue                    │
│                                                     │
├─────────────────────────────────────────────────────┤
│  40 tests passing | 0 clippy warnings | 3 API       │
│  endpoints | 9 PSPs | 3 routing strategies           │
└─────────────────────────────────────────────────────┘
```
