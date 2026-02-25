# Yuno Internal Challenge — Payment Routing Engine

> **Solve FashionForward's 22% Decline Storm** — A smart payment routing engine with multi-PSP retry logic, built in Rust and deployed on Vercel.

---

## The Challenge

FashionForward is a rapidly growing fashion marketplace operating across **Brazil, Mexico, and Colombia**, processing approximately **45,000 transactions per day** through Yuno's payment orchestration platform.

**The problem:** 22% of their transactions are being declined. Their current routing logic is primitive — when a customer tries to pay, the system always tries PSP #1 first. If that fails, it gives up, even though they have 2-3 backup PSPs configured per country.

**Our solution:** A proof-of-concept routing engine that demonstrates intelligent retry logic:
- **Soft declines** (issuer unavailable, suspected fraud, do not honor) → **retry with backup PSP**
- **Hard declines** (insufficient funds, expired card, invalid card) → **fail immediately**
- **Performance reporting** showing measurable improvement in authorization rates

---

## Evaluation Criteria

| Criteria | Points |
|---|---|
| Core routing engine correctly implements retry logic (soft declines retry, hard declines fail fast) | 25 |
| PSP simulation is realistic (different success rates, decline reasons, latency per PSP) | 20 |
| Performance report clearly demonstrates authorization rate improvement and quantifies business impact | 20 |
| Code quality: well-structured, readable, with clear separation of concerns | 15 |
| Documentation: README is clear, complete, and enables reviewer to run the solution easily | 10 |
| Stretch goals: cost-aware routing, real-time cascading, or other creative enhancements | 10 |
| **Total** | **100** |

---

## Tech Stack

| Technology | Purpose |
|---|---|
| **Rust** (2021 edition) | Core language for performance and type safety |
| **Vercel** | Serverless deployment platform |
| **vercel-community/rust** | Rust runtime for Vercel serverless functions |
| **tokio** | Async runtime |
| **serde / serde_json** | Serialization and JSON handling |
| **rand** | Deterministic RNG with `StdRng` for reproducible PSP simulation |

---

## Architecture

```
src/
├── lib.rs                    # Module exports + shared helpers
├── models/                   # Domain types (shared by all modules)
│   ├── transaction.rs        # Transaction, Currency, Country
│   ├── psp.rs                # PspConfig, PspResponse, DeclineReason
│   ├── routing.rs            # RoutingResult, RoutingAttempt, RoutingStrategy
│   └── report.rs             # PerformanceReport, ScenarioResult, metrics
├── simulator/                # PSP behavior simulation
│   ├── mod.rs                # PspSimulator (deterministic, seeded RNG)
│   └── config.rs             # 9 PSP configs (3 per country)
├── engine/                   # Core routing engine
│   ├── mod.rs                # RoutingEngine (orchestrates retry flow)
│   ├── retry.rs              # Hard/soft decline classification
│   └── strategy.rs           # PSP selection strategies
├── data/                     # Test data generation
│   └── mod.rs                # 200+ transaction generator
└── report/                   # Performance reporting
    └── mod.rs                # No-retry vs smart-retry comparison

api/
├── health.rs                 # GET  /api/health
├── authorize.rs              # POST /api/authorize
└── report.rs                 # POST /api/report
```

### Data Flow

```
                        ┌─────────────────────────────┐
                        │   POST /api/authorize        │
                        │   (incoming transaction)     │
                        └──────────────┬──────────────┘
                                       │
                                       ▼
                        ┌─────────────────────────────┐
                        │       RoutingEngine          │
                        │  ┌─────────────────────┐    │
                        │  │ Strategy: select PSP │    │
                        │  │ order based on goal  │    │
                        │  └──────────┬──────────┘    │
                        └─────────────┼───────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                  ▼
             ┌───────────┐    ┌───────────┐      ┌───────────┐
             │  PSP #1   │    │  PSP #2   │      │  PSP #3   │
             │ (primary) │    │ (backup)  │      │ (backup)  │
             └─────┬─────┘    └─────┬─────┘      └─────┬─────┘
                   │                │                   │
                   ▼                ▼                   ▼
              PspSimulator    PspSimulator         PspSimulator
              (seeded RNG)    (seeded RNG)         (seeded RNG)
```

**Retry flow:**

1. The engine selects PSPs in order based on the routing strategy.
2. The transaction is sent to PSP #1.
3. If **approved** — return success immediately.
4. If **hard decline** (InsufficientFunds, CardExpired, InvalidCard, StolenCard) — stop. The card genuinely cannot be charged. No retry.
5. If **soft decline** (IssuerUnavailable, SuspectedFraud, DoNotHonor, ProcessorDeclined) — retry with PSP #2.
6. If **PSP unavailable** — cascade immediately to the next PSP without counting it as a decline attempt.
7. Repeat until approved or all PSPs exhausted (up to 3 attempts).

---

## Key Design Decisions

### 1. Deterministic PSP Simulation

The simulator uses `hash(card_bin + card_last4 + psp_id + amount_as_cents)` as a seed for `StdRng`. This means:

- **Same card at the same PSP always produces the same result.** Running the engine twice with identical input yields identical output — critical for reproducible demos, testing, and debugging.
- **Different PSPs produce different results for the same card.** Because `psp_id` is part of the hash, a card declined at PSP #1 may succeed at PSP #2. This is what makes retry valuable: each PSP has a distinct relationship with issuing banks, and the simulator reflects that reality.
- **Determinism does not sacrifice realism.** Each PSP still has its own configured success rate, latency range, and decline reason distribution. The seed simply ensures consistency across runs.

### 2. Hard vs Soft Decline Classification

The distinction between hard and soft declines is the foundation of the retry logic:

**Hard declines** (do NOT retry):
- `InsufficientFunds` — the cardholder does not have the money
- `CardExpired` — the card is no longer valid
- `InvalidCard` — the card number is wrong
- `StolenCard` — the card has been reported stolen

These are **permanent**. The card genuinely cannot be charged regardless of which PSP processes it. Retrying wastes time, incurs additional PSP fees, and cannot change the outcome.

**Soft declines** (retry with next PSP):
- `IssuerUnavailable` — the issuing bank is temporarily unreachable
- `SuspectedFraud` — the transaction was flagged by fraud detection
- `DoNotHonor` — a generic decline from the issuer
- `ProcessorDeclined` — the PSP's processor rejected the transaction

These are often **PSP-specific**. A different PSP may have a different connection to the issuing bank, different fraud scoring models, or different processing infrastructure. What fails at one PSP frequently succeeds at another.

### 3. PSP Selection Strategy Tradeoffs

Three routing strategies allow the merchant to optimize for different business goals:

| Strategy | Sorting Logic | Best For |
|---|---|---|
| `OptimizeForApprovals` | PSPs sorted by highest `base_success_rate` first | Maximizing authorization rate; merchants who prioritize conversion over cost |
| `OptimizeForCost` | PSPs sorted by lowest effective fee (`fee_percentage + fee_fixed / avg_amount`) first | Minimizing processing costs; high-volume merchants with thin margins |
| `Balanced` | PSPs scored by `success_rate * 0.7 + (1 - normalized_fee) * 0.3` | Practical middle ground; most merchants in production |

The tradeoff is real: the cheapest PSP is rarely the one with the highest approval rate. `Balanced` weights approval rate at 70% and cost at 30%, reflecting that a declined transaction generates zero revenue regardless of how cheap the PSP is.

### 4. Revenue Impact Calculation

The business impact is calculated as:

```
additional_approvals = smart_retry_approved - no_retry_approved
estimated_revenue    = additional_approvals * average_transaction_value
```

For FashionForward's scale:
- ~45,000 daily transactions with a ~22% decline rate = ~9,900 declines/day
- Even a **9 percentage point improvement** in authorization rate means ~4,050 additional daily approvals
- At an average order value of $50, that is approximately **$200K/day in recovered revenue**

This is the core value proposition: smart retry does not just improve a metric — it recovers real revenue that was being lost to primitive routing.

### 5. Stateless Architecture

Each routing request is fully independent. The `RoutingEngine` holds no shared state between requests — no in-memory caches, no session data, no accumulated statistics. This design:

- Aligns perfectly with **serverless deployment** where each function invocation is isolated
- Enables **horizontal scaling** with zero coordination overhead
- Eliminates an entire class of **concurrency bugs** (no mutexes, no race conditions)
- Makes the system **trivially testable** — every test is a pure function from input to output

---

## Getting Started

### Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Vercel CLI** (optional): `npm i -g vercel`

### Build

```bash
cargo build
cargo clippy   # Lint check
cargo test     # Run tests
```

### Run Locally

```bash
# Using Vercel CLI (recommended)
vercel dev

# Or build and test with cargo
cargo build
```

---

## API Endpoints

### `GET /api/health`

Health check endpoint.

```bash
curl https://your-app.vercel.app/api/health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### `POST /api/authorize`

Route a single transaction through PSPs with smart retry logic.

```bash
curl -X POST https://your-app.vercel.app/api/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 150.00,
    "currency": "BRL",
    "country": "Brazil",
    "card_bin": "411111",
    "card_last4": "1234",
    "customer_id": "cust_001",
    "routing_strategy": "OptimizeForApprovals"
  }'
```

**Response (approved after retry):**
```json
{
  "transaction_id": "txn_a1b2c3",
  "approved": true,
  "final_psp": "Cielo",
  "total_attempts": 2,
  "total_latency_ms": 450,
  "attempts": [
    {
      "psp_id": "psp_br_1",
      "psp_name": "PagSeguro",
      "approved": false,
      "decline_reason": "issuer_unavailable",
      "latency_ms": 320,
      "attempt_number": 1
    },
    {
      "psp_id": "psp_br_2",
      "psp_name": "Cielo",
      "approved": true,
      "decline_reason": null,
      "latency_ms": 180,
      "attempt_number": 2
    }
  ]
}
```

**Response (hard decline — no retry):**
```json
{
  "transaction_id": "txn_x9y8z7",
  "approved": false,
  "final_psp": null,
  "total_attempts": 1,
  "total_latency_ms": 210,
  "attempts": [
    {
      "psp_id": "psp_br_1",
      "psp_name": "PagSeguro",
      "approved": false,
      "decline_reason": "insufficient_funds",
      "latency_ms": 210,
      "attempt_number": 1
    }
  ]
}
```

### `POST /api/report`

Generate a batch performance report comparing no-retry vs smart-retry.

```bash
curl -X POST https://your-app.vercel.app/api/report \
  -H "Content-Type: application/json" \
  -d '{
    "transaction_count": 200,
    "routing_strategy": "OptimizeForApprovals"
  }'
```

**Response:**
```json
{
  "total_transactions": 200,
  "no_retry": {
    "approved": 156,
    "declined": 44,
    "authorization_rate": 78.0,
    "avg_attempts": 1.0,
    "avg_latency_ms": 265.0
  },
  "smart_retry": {
    "approved": 174,
    "declined": 26,
    "authorization_rate": 87.0,
    "avg_attempts": 1.45,
    "avg_latency_ms": 385.0
  },
  "improvement": {
    "rate_lift_percentage": 9.0,
    "additional_approvals": 18,
    "estimated_revenue_recovered_usd": 4500.00
  },
  "by_country": { "...": "breakdown per country" },
  "by_psp": { "...": "breakdown per PSP" }
}
```

---

## How the Report Works

The `/api/report` endpoint runs a **two-scenario comparison** to quantify the impact of smart routing. Both scenarios process the exact same set of transactions through the same deterministic PSP simulator, ensuring a fair comparison.

### Scenario 1: No Retry (baseline)

Each transaction is sent to PSP #1 only. If the PSP declines the transaction for any reason — hard or soft — the decline is final. This represents FashionForward's current routing behavior.

### Scenario 2: Smart Retry (routing engine)

Each transaction is routed through the full engine:
1. PSPs are ordered according to the selected strategy.
2. The transaction is sent to the first PSP.
3. On a **soft decline**, the engine retries with the next PSP in the list.
4. On a **hard decline**, the engine stops immediately — retrying cannot help.
5. On **PSP unavailable**, the engine cascades to the next PSP without counting it as a decline.
6. Up to 3 PSPs are tried per transaction.

### Metrics Calculated

| Metric | Description |
|---|---|
| `authorization_rate` | Percentage of transactions approved (approved / total * 100) |
| `avg_attempts` | Mean number of PSP attempts per transaction |
| `avg_latency_ms` | Mean total latency across all attempts per transaction |
| `by_country` | Per-country breakdown: no-retry rate, smart-retry rate, improvement |
| `by_psp` | Per-PSP breakdown: total attempts, approvals, declines, approval rate, avg latency |

### Business Impact

The `improvement` section quantifies the real-world value:

- **`rate_lift_percentage`**: Authorization rate improvement in percentage points (e.g., 78% to 87% = 9 points).
- **`additional_approvals`**: Absolute number of transactions recovered by retry (smart approved - baseline approved).
- **`estimated_revenue_recovered_usd`**: `additional_approvals * average_transaction_value` from the batch. This extrapolates directly to daily revenue at production scale.

---

## PSP Configuration

| PSP | Country | Success Rate | Latency | Fee |
|---|---|---|---|---|
| PagSeguro | Brazil | 78% | 200-400ms | 2.9% + $0.30 |
| Cielo | Brazil | 82% | 150-250ms | 3.2% + $0.25 |
| Stone | Brazil | 68% | 300-600ms | 2.5% + $0.35 |
| Conekta | Mexico | 75% | 180-350ms | 2.8% + $0.28 |
| OpenPay | Mexico | 80% | 200-300ms | 3.1% + $0.22 |
| SR Pago | Mexico | 70% | 250-500ms | 2.6% + $0.32 |
| PayU | Colombia | 76% | 190-380ms | 2.7% + $0.29 |
| Wompi | Colombia | 83% | 160-280ms | 3.3% + $0.20 |
| Bold | Colombia | 65% | 280-550ms | 2.4% + $0.38 |

---

## Stretch Goals

### Cost-Aware Routing

PSP selection goes beyond approval rates. Three strategies are available, each sorting the PSP list differently before the retry loop begins:

- **`OptimizeForApprovals`**: PSPs sorted by `base_success_rate` descending. Maximizes authorization rate at the potential expense of higher processing fees.
- **`OptimizeForCost`**: PSPs sorted by effective fee ascending, calculated as `fee_percentage + fee_fixed / average_amount`. Chooses the cheapest processing path first, accepting a potentially lower approval rate.
- **`Balanced`**: PSPs scored with a weighted formula: `success_rate * 0.7 + (1 - normalized_fee) * 0.3`. This weights approval rate at 70% (because a declined transaction generates zero revenue regardless of fee) while still favoring cheaper PSPs when approval rates are comparable.

### Real-Time Cascading

If a PSP returns `PspUnavailable` (simulated approximately 10% of the time), the engine immediately cascades to the next PSP in the list. This cascade is handled differently from a normal decline:

- It does **not** count as a decline attempt in the retry budget.
- It does **not** appear as a decline in the per-PSP metrics (PSP unavailable events are excluded from decline counts).
- It **does** add latency, which is reflected in the total latency for the transaction.

This prevents transient PSP downtime from artificially inflating decline rates and ensures that the merchant's authorization rate is not penalized by infrastructure issues outside their control.

---

## Deployment

The project auto-deploys to Vercel on push to `main`. Each `api/*.rs` file is compiled into an independent serverless function by the `vercel-community/rust` runtime.

```bash
# Manual deploy
vercel --prod

# Or just push to main for auto-deploy
git push origin main
```

---

## Project Structure

The project was developed incrementally across feature branches, each focused on a distinct layer of the system:

| Branch | Scope |
|---|---|
| `feature/psp-simulator` | PSP simulation engine: deterministic seeded RNG, 9 PSP configurations, decline reason distributions, latency simulation, and test data generation |
| `feature/routing-engine` | Core routing logic: hard/soft decline classification, retry orchestration, PSP selection strategies (approval, cost, balanced), and cascading on PSP unavailability |
| `feature/api-reports` | API endpoints (`/authorize`, `/report`) and performance reporting: two-scenario comparison, per-country and per-PSP breakdowns, business impact calculation |

Each branch was merged into `main` sequentially. The commit history follows the [Gitmoji](https://gitmoji.dev/) convention, providing a clear narrative of how the system was built from the ground up.
