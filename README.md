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

### Key Design Decisions

1. **Deterministic PSP Simulation**: Uses seeded RNG based on `hash(card_bin + card_last4 + psp_id + amount)` so the same transaction produces the same result per PSP — enabling reproducible demos while maintaining realistic behavior.

2. **Hard vs Soft Decline Classification**: Hard declines (insufficient funds, expired card, stolen card, invalid card) fail immediately — retrying won't help. Soft declines (issuer unavailable, suspected fraud, do not honor, processor declined) trigger automatic retry with the next available PSP.

3. **Strategy-Based PSP Selection**: Three routing strategies allow optimizing for different business goals:
   - `optimize_for_approvals` — highest success rate PSPs first
   - `optimize_for_cost` — cheapest PSPs first
   - `balanced` — weighted score combining success rate and cost

4. **Stateless Routing Engine**: Each routing call is independent — no shared state between requests. This aligns with the serverless deployment model.

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

- **Cost-Aware Routing**: PSP selection considers processing fees alongside approval rates. Three strategies available: `optimize_for_approvals`, `optimize_for_cost`, `balanced`.
- **Real-Time Cascading**: If a PSP is unavailable (timeout/downtime), immediately cascade to the next PSP without counting it as a decline.

---

## Deployment

The project auto-deploys to Vercel on push to `main`. Each `api/*.rs` file is compiled into an independent serverless function by the `vercel-community/rust` runtime.

```bash
# Manual deploy
vercel --prod

# Or just push to main for auto-deploy
git push origin main
```
