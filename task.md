# Task Plan — FashionForward Routing Engine

## Challenge Summary

FashionForward processes ~45,000 transactions/day across Brazil, Mexico, and Colombia. Their current system has a 22% decline rate because it only tries one PSP and gives up. We're building a smart routing engine that retries with backup PSPs on soft declines.

## Evaluation Criteria

| Criteria | Points |
|---|---|
| Core routing engine (retry logic: soft retry, hard fail fast) | 25 |
| PSP simulation (different rates, decline reasons, latency) | 20 |
| Performance report (auth rate improvement + business impact) | 20 |
| Code quality (structured, readable, separation of concerns) | 15 |
| Documentation (README, setup, usage guide) | 10 |
| Stretch goals (cost-aware routing, cascading) | 10 |
| **Total** | **100** |

---

## Architecture

```
src/
├── lib.rs                    # Module declarations + helpers
├── models/                   # Shared domain types (FOUNDATION - on main)
│   ├── mod.rs
│   ├── transaction.rs        # Transaction, Currency, Country
│   ├── psp.rs                # PspConfig, PspResponse, DeclineReason
│   ├── routing.rs            # RoutingResult, RoutingAttempt, AuthorizationRequest
│   └── report.rs             # PerformanceReport, ScenarioResult, metrics
├── simulator/                # INSTANCE 1
│   ├── mod.rs                # PspSimulator
│   └── config.rs             # 9 PSP configurations
├── engine/                   # INSTANCE 2
│   ├── mod.rs                # RoutingEngine
│   ├── retry.rs              # Retry logic (hard/soft classification)
│   └── strategy.rs           # PSP selection strategies
├── data/                     # INSTANCE 1
│   └── mod.rs                # Test data generator
└── report/                   # INSTANCE 3
    └── mod.rs                # Performance report generator

api/
├── health.rs                 # GET /api/health (exists)
├── authorize.rs              # POST /api/authorize (INSTANCE 3)
└── report.rs                 # POST /api/report (INSTANCE 3)
```

---

## Phase 0: Foundation (on `main`, DONE before branching)

Already committed to main:
- All domain models in `src/models/` (complete, shared)
- All module stubs (compile but return dummy data)
- All API endpoint stubs (return 501)
- Updated Cargo.toml with all deps + all [[bin]] entries
- Updated vercel.json with all routes
- README.md with full challenge description

**All 3 instances branch FROM this foundation.**

---

## Instance 1 — Branch: `feature/psp-simulator`

### Clone & Setup
```bash
cd ~/Documents/Yuno/yuno-instance-2
git fetch origin
git checkout feature/psp-simulator
```

### Points Targeted: PSP Simulation (20pts) + Code Quality (15pts partial)

### Files to Implement (replace stubs with real code)

#### `src/simulator/config.rs` — PSP Configurations
Create 9 PSPs (3 per country) with these specs:

| PSP ID | Name | Country | Success Rate | Latency (ms) | Fee % | Fee Fixed | Soft Decline Bias |
|---|---|---|---|---|---|---|---|
| psp_br_1 | PagSeguro | Brazil | 78% | 200-400 | 2.9% | $0.30 | issuer_unavailable |
| psp_br_2 | Cielo | Brazil | 82% | 150-250 | 3.2% | $0.25 | suspected_fraud |
| psp_br_3 | Stone | Brazil | 68% | 300-600 | 2.5% | $0.35 | do_not_honor |
| psp_mx_1 | Conekta | Mexico | 75% | 180-350 | 2.8% | $0.28 | processor_declined |
| psp_mx_2 | OpenPay | Mexico | 80% | 200-300 | 3.1% | $0.22 | issuer_unavailable |
| psp_mx_3 | SR Pago | Mexico | 70% | 250-500 | 2.6% | $0.32 | suspected_fraud |
| psp_co_1 | PayU | Colombia | 76% | 190-380 | 2.7% | $0.29 | do_not_honor |
| psp_co_2 | Wompi | Colombia | 83% | 160-280 | 3.3% | $0.20 | issuer_unavailable |
| psp_co_3 | Bold | Colombia | 65% | 280-550 | 2.4% | $0.38 | processor_declined |

Function: `pub fn get_psps_for_country(country: &Country) -> Vec<PspConfig>`

#### `src/simulator/mod.rs` — PSP Simulator
```rust
pub struct PspSimulator;

impl PspSimulator {
    pub fn new() -> Self;
    
    /// Simulate a PSP processing a transaction.
    /// Uses deterministic seeding from card_bin + amount + psp_id hash.
    pub fn process(&self, transaction: &Transaction, psp: &PspConfig) -> PspResponse;
}
```

**Simulation rules:**
- Create a deterministic seed from: hash(card_bin + card_last4 + psp_id + amount_as_cents)
- Use that seed with `rand::rngs::StdRng::seed_from_u64(seed)` for reproducibility
- Roll against PSP's success_rate to decide approve/decline
- If declined, pick a decline reason from the PSP's decline_distribution
- ~70-75% of transactions should be approvable by at least one PSP
- ~20-25% should be "challenging" (declined by PSP#1 but potentially approvable by PSP#2/3)
- ~5-8% should be hard declines regardless of PSP
- Simulate latency by generating a random value in [latency_min, latency_max]
- **Stretch:** 10% of requests randomly return PspUnavailable (for cascading feature)

#### `src/data/mod.rs` — Test Data Generator
```rust
/// Generate a batch of test transactions for the performance report.
pub fn generate_test_data(count: usize) -> Vec<Transaction>;

/// Get pre-built test dataset of 200+ transactions.
pub fn get_test_dataset() -> Vec<Transaction>;
```

**Data requirements:**
- 200+ transactions
- Amount range: $10–$500 USD equivalent
- ~Equal split across Brazil (BRL), Mexico (MXN), Colombia (COP)
- 15+ unique customer IDs (some with multiple transactions)
- Realistic fake BINs: 
  - Brazil: 411111, 510510, 376411
  - Mexico: 424242, 551234, 371449
  - Colombia: 431940, 520082, 378282
- Timestamps: spread across a simulated day

### Commits (gitmoji, one per meaningful change)
1. `✨ Implement PSP configurations for Brazil, Mexico, and Colombia`
2. `✨ Implement PSP simulator with deterministic behavior`
3. `✨ Implement test data generator with 200+ transactions`
4. `📝 Update AGENTS.md files for simulator and data modules`

### After completing, push:
```bash
git push origin feature/psp-simulator
```

---

## Instance 2 — Branch: `feature/routing-engine`

### Clone & Setup
```bash
cd ~/Documents/Yuno/yuno-instance-3
git fetch origin
git checkout feature/routing-engine
```

### Points Targeted: Core Routing (25pts) + Stretch Goals (10pts)

### Files to Implement (replace stubs with real code)

#### `src/engine/retry.rs` — Decline Classification
```rust
use crate::models::psp::DeclineReason;

/// Determines if a decline reason is a hard decline (no retry).
pub fn is_hard_decline(reason: &DeclineReason) -> bool;

/// Determines if a decline reason is a soft decline (retry with next PSP).
pub fn is_soft_decline(reason: &DeclineReason) -> bool;

/// Determines if a PSP response indicates unavailability (cascade immediately).
pub fn is_psp_unavailable(reason: &DeclineReason) -> bool;
```

Hard declines (NO retry): `InsufficientFunds`, `CardExpired`, `InvalidCard`, `StolenCard`
Soft declines (RETRY): `IssuerUnavailable`, `SuspectedFraud`, `DoNotHonor`, `ProcessorDeclined`

#### `src/engine/strategy.rs` — PSP Selection Strategies
```rust
use crate::models::psp::PspConfig;
use crate::models::routing::RoutingStrategy;

/// Order PSPs based on the chosen routing strategy.
pub fn select_psp_order(psps: &[PspConfig], strategy: &RoutingStrategy) -> Vec<PspConfig>;
```

Strategies:
- `OptimizeForApprovals`: Sort by `base_success_rate` descending (best approval rate first)
- `OptimizeForCost`: Sort by total fee ascending (cheapest PSP first)
- `Balanced`: Score = `success_rate * 0.7 + (1.0 - normalized_fee) * 0.3`, sort descending

#### `src/engine/mod.rs` — Core Routing Engine
```rust
use crate::models::transaction::Transaction;
use crate::models::routing::{RoutingResult, RoutingAttempt, RoutingStrategy};
use crate::simulator::PspSimulator;
use crate::simulator::config::get_psps_for_country;

pub struct RoutingEngine {
    simulator: PspSimulator,
}

impl RoutingEngine {
    pub fn new(simulator: PspSimulator) -> Self;
    
    /// Route a transaction through PSPs with smart retry logic.
    /// 1. Get PSPs for the transaction's country
    /// 2. Order them by strategy
    /// 3. Try each PSP in order:
    ///    - If approved → return success
    ///    - If hard decline → return failure immediately (NO retry)
    ///    - If soft decline → try next PSP
    ///    - If PSP unavailable → skip to next PSP (cascade, don't count as attempt)
    /// 4. If all PSPs exhausted → return declined
    pub fn route(&self, transaction: &Transaction, strategy: &RoutingStrategy) -> RoutingResult;
    
    /// Route with no retry (single PSP attempt) — for comparison in reports.
    pub fn route_no_retry(&self, transaction: &Transaction) -> RoutingResult;
}
```

**Key behaviors:**
- `route()` implements full retry logic with up to 3 PSP attempts
- `route_no_retry()` tries only PSP#1 and returns result (simulates FashionForward's current behavior)
- Both methods capture full routing metadata (attempts, latency, decline reasons)
- Latency is summed across all attempts

**Stretch: Cascading**
- If a PSP returns unavailable status, immediately try next PSP
- Don't count unavailable as a "decline attempt"
- Track cascading events in routing metadata

### Commits (gitmoji)
1. `✨ Implement hard/soft decline classification`
2. `✨ Implement PSP selection strategies (approvals, cost, balanced)`
3. `✨ Implement core routing engine with smart retry logic`
4. `✨ Add real-time cascading for PSP unavailability`
5. `📝 Update AGENTS.md files for engine module`

### After completing, push:
```bash
git push origin feature/routing-engine
```

---

## Instance 3 — Branch: `feature/api-reports`

### Clone & Setup
```bash
cd ~/Documents/Yuno/yuno-internal-challenge  (original clone)
git fetch origin
git checkout feature/api-reports
```

### Points Targeted: Performance Report (20pts) + Documentation (10pts)

### Files to Implement (replace stubs with real code)

#### `src/report/mod.rs` — Performance Report Generator
```rust
use crate::models::transaction::Transaction;
use crate::models::routing::{RoutingResult, RoutingStrategy};
use crate::models::report::PerformanceReport;
use crate::engine::RoutingEngine;

/// Generate a complete performance report comparing no-retry vs smart-retry.
pub fn generate_report(
    transactions: &[Transaction],
    engine: &RoutingEngine,
    strategy: &RoutingStrategy,
) -> PerformanceReport;

/// Run all transactions in no-retry mode (single PSP, fail on any decline).
fn run_no_retry(transactions: &[Transaction], engine: &RoutingEngine) -> Vec<RoutingResult>;

/// Run all transactions with smart retry (full routing engine).
fn run_smart_retry(
    transactions: &[Transaction],
    engine: &RoutingEngine,
    strategy: &RoutingStrategy,
) -> Vec<RoutingResult>;

/// Calculate metrics from routing results.
fn calculate_metrics(results: &[RoutingResult], transactions: &[Transaction]) -> ScenarioResult;
```

**Report output must show:**
- Total transactions processed
- No-retry: approved count, declined count, authorization rate, avg attempts (always 1.0)
- Smart-retry: approved count, declined count, authorization rate, avg attempts
- Improvement: rate lift %, additional approvals, estimated revenue recovered (avg transaction value × additional approvals)
- Breakdown by country (auth rate per country, both scenarios)
- Breakdown by PSP (attempts, approvals, declines, approval rate, avg latency)

#### `api/authorize.rs` — POST /api/authorize
Accept a single transaction authorization request and route it.

**Request body:**
```json
{
  "amount": 150.00,
  "currency": "BRL",
  "country": "Brazil",
  "card_bin": "411111",
  "card_last4": "1234",
  "customer_id": "cust_001",
  "routing_strategy": "optimize_for_approvals"
}
```

**Response (success):**
```json
{
  "transaction_id": "txn_<uuid>",
  "approved": true,
  "final_psp": "Cielo",
  "total_attempts": 2,
  "total_latency_ms": 450,
  "attempts": [
    {"psp_id": "psp_br_1", "psp_name": "PagSeguro", "approved": false, "decline_reason": "issuer_unavailable", "latency_ms": 320, "attempt_number": 1},
    {"psp_id": "psp_br_2", "psp_name": "Cielo", "approved": true, "decline_reason": null, "latency_ms": 180, "attempt_number": 2}
  ]
}
```

**Error handling:**
- 400 for missing/invalid fields
- 405 for non-POST methods
- Always return JSON with `Content-Type: application/json`

#### `api/report.rs` — POST /api/report
Generate the batch performance report.

**Request body (optional):**
```json
{
  "transaction_count": 200,
  "routing_strategy": "optimize_for_approvals"
}
```

**Response:** Full PerformanceReport JSON (see models/report.rs)

#### `README.md` — Full Documentation Update
Must include:
- Challenge description (the FashionForward scenario)
- Evaluation criteria table
- Architecture overview with module diagram
- Setup instructions (prerequisites, cargo build)
- API usage with curl examples for both endpoints
- How to generate the performance report
- Key design decisions:
  - Why deterministic PSP simulation
  - How retry logic works (hard vs soft decline)
  - PSP selection strategy tradeoffs
  - Revenue impact calculation methodology
- Example report output
- Stretch goals attempted

### Commits (gitmoji)
1. `✨ Implement performance report generator with comparison logic`
2. `✨ Implement POST /api/authorize endpoint`
3. `✨ Implement POST /api/report endpoint`
4. `📝 Update README with full challenge documentation`
5. `📝 Update AGENTS.md files for report module and api handlers`

### After completing, push:
```bash
git push origin feature/api-reports
```

---

## Phase 4: Integration (after all 3 instances complete)

Run from any instance:

```bash
cd ~/Documents/Yuno/yuno-internal-challenge
git checkout main
git pull origin main

# Merge in order (simulator first, then engine, then API)
git merge feature/psp-simulator
git merge feature/routing-engine
# Resolve src/lib.rs conflicts (keep all pub mod declarations)
git merge feature/api-reports
# Resolve src/lib.rs + Cargo.toml conflicts if any

# Verify
cargo check
cargo clippy
cargo build

# Final integration commit
git add -A
git commit -m "🔧 Integrate all modules: simulator + engine + API + reports"
git push origin main
```

### Integration Wiring
The stubs are already wired with correct imports. Once all 3 branches are merged, the real implementations replace the stubs and everything connects:
- `api/authorize.rs` calls `RoutingEngine::route()` 
- `RoutingEngine` calls `PspSimulator::process()`
- `api/report.rs` calls `report::generate_report()` which uses `RoutingEngine`
- `data::get_test_dataset()` provides the 200+ transactions

---

## Quick Reference: Branch → File Ownership

| File | Instance 1 | Instance 2 | Instance 3 |
|---|---|---|---|
| src/simulator/mod.rs | ✅ OWNS | | |
| src/simulator/config.rs | ✅ OWNS | | |
| src/engine/mod.rs | | ✅ OWNS | |
| src/engine/retry.rs | | ✅ OWNS | |
| src/engine/strategy.rs | | ✅ OWNS | |
| src/data/mod.rs | ✅ OWNS | | |
| src/report/mod.rs | | | ✅ OWNS |
| api/authorize.rs | | | ✅ OWNS |
| api/report.rs | | | ✅ OWNS |
| README.md | | | ✅ OWNS |
| src/models/* | SHARED (don't modify) | SHARED (don't modify) | SHARED (don't modify) |
