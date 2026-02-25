# src/simulator/ — PSP Behavior Simulator

## Purpose

Simulates realistic Payment Service Provider (PSP) behavior for the payment orchestration engine. Instead of calling real PSPs, this module produces deterministic, reproducible transaction outcomes — approval/decline decisions, decline reasons, latency, and unavailability — that vary across PSPs for the same card. This makes smart routing and retry logic measurably valuable.

---

## Files

| File | Role |
|------|------|
| `mod.rs` | `PspSimulator` struct and all simulation logic: hard/soft decline decisions, decline reason selection, latency simulation, PSP unavailability checks, and deterministic seeding functions. |
| `config.rs` | PSP configuration data: `get_psps_for_country()`, `get_all_psps()`, and `get_decline_distribution()` returning per-PSP decline reason weights. Defines the `DeclineWeight` struct. |

---

## PSP Configuration (9 PSPs)

### Brazil

| ID | Name | Success Rate | Latency (ms) | Fee % | Fixed (cents) | Decline Bias |
|----|------|:------------:|:------------:|:-----:|:-------------:|:------------:|
| `psp_br_1` | PagSeguro | 78% | 200–400 | 2.9% | 30 | IssuerUnavailable |
| `psp_br_2` | Cielo | 82% | 150–250 | 3.2% | 25 | SuspectedFraud |
| `psp_br_3` | Stone | 68% | 300–600 | 2.5% | 35 | DoNotHonor |

### Mexico

| ID | Name | Success Rate | Latency (ms) | Fee % | Fixed (cents) | Decline Bias |
|----|------|:------------:|:------------:|:-----:|:-------------:|:------------:|
| `psp_mx_1` | Conekta | 75% | 180–350 | 2.8% | 28 | ProcessorDeclined |
| `psp_mx_2` | OpenPay | 80% | 200–300 | 3.1% | 22 | IssuerUnavailable |
| `psp_mx_3` | SR Pago | 70% | 250–500 | 2.6% | 32 | SuspectedFraud |

### Colombia

| ID | Name | Success Rate | Latency (ms) | Fee % | Fixed (cents) | Decline Bias |
|----|------|:------------:|:------------:|:-----:|:-------------:|:------------:|
| `psp_co_1` | PayU | 76% | 190–380 | 2.7% | 29 | DoNotHonor |
| `psp_co_2` | Wompi | 83% | 160–280 | 3.3% | 20 | IssuerUnavailable |
| `psp_co_3` | Bold | 65% | 280–550 | 2.4% | 38 | ProcessorDeclined |

Each country has three tiers: a primary (moderate rates/fees), a premium (best rates, highest fees), and a budget (lowest rates, cheapest fees). Decline bias means 45% of that PSP's soft declines are the biased reason; the remaining three reasons share the other 55%.

---

## Simulation Logic

`PspSimulator::process()` follows a 4-step decision pipeline:

1. **Hard decline check (~6%)** — Uses a card-only seed (`card_bin + card_last4`). If the card is a hard-decline card, it fails on every PSP with one of: `InsufficientFunds` (45%), `CardExpired` (30%), `InvalidCard` (15%), `StolenCard` (10%).

2. **PSP unavailability check (8%)** — Uses a `transaction_id + psp_id` seed. Simulates intermittent PSP downtime. Returns `DeclineReason::PspUnavailable`. This enables the real-time cascading stretch goal.

3. **Soft decline roll** — Uses a PSP-dependent seed (`card_bin + card_last4 + psp_id + amount`). Rolls against `psp.base_success_rate`. If the roll exceeds the rate, the transaction is soft-declined.

4. **Decline reason selection** — On soft decline, picks a reason from the PSP's weighted distribution (see Decline Bias column above).

Latency is simulated independently using `transaction_id + psp_id + "latency"` as seed, generating a value within the PSP's `[latency_min_ms, latency_max_ms]` range.

---

## Deterministic Seeding

All randomness is derived from `std::collections::hash_map::DefaultHasher` → `StdRng::seed_from_u64`. The simulator is stateless and produces identical results across runs for the same inputs.

### Seed composition by decision type

| Decision | Seed Inputs | Why |
|----------|-------------|-----|
| Hard decline | `card_bin`, `card_last4`, `"card_seed"` | PSP-independent — same card always hard-declines everywhere |
| Hard decline reason | Same as above + `wrapping_add(1)` | Separate roll, still card-only |
| PSP unavailability | `transaction_id`, `psp_id`, `"unavailable_check"` | Per-request, per-PSP — different PSPs may be up/down |
| Soft decline | `card_bin`, `card_last4`, `psp_id`, `amount` | **Key design**: `psp_id` in the seed means different PSPs get different outcomes for the same card |
| Latency | `transaction_id`, `psp_id`, `"latency"` | Per-request variation within PSP's range |

The inclusion of `psp_id` in the soft decline seed is the core mechanism that makes retry/cascading valuable: PSP #1 may decline a card that PSP #2 approves.

---

## Testing Conventions

Tests live in `#[cfg(test)] mod tests` at the bottom of each file. Helper function `make_test_transaction()` builds a `Transaction` with minimal required fields.

### Key test cases in `mod.rs`

| Test | Verifies |
|------|----------|
| `test_simulator_is_deterministic` | Same inputs → identical `approved` and `decline_reason` |
| `test_different_psps_can_produce_different_results` | Over 100 cards, at least one has different outcomes between two PSPs |
| `test_hard_declines_are_psp_independent` | If PSP #1 hard-declines a card, all other PSPs also hard-decline it |
| `test_latency_within_range` | Every simulated latency falls within `[latency_min_ms, latency_max_ms]` |
| `test_approval_rate_distribution` | Over 1000 transactions, approval rate falls within a reasonable band |

### Key test cases in `config.rs`

| Test | Verifies |
|------|----------|
| `test_each_country_has_three_psps` | 3 PSPs per country |
| `test_all_psps_returns_nine` | 9 total PSPs |
| `test_success_rates_are_valid` | All rates in `(0.0, 1.0)` |
| `test_decline_distributions_sum_to_one` | Decline weights sum to 1.0 for every PSP |

Run all tests: `cargo test --lib simulator`
