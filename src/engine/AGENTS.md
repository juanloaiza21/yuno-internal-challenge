# src/engine/ — Payment Routing Engine

## Purpose

This module implements the intelligent payment routing engine. It orchestrates PSP (Payment Service Provider) selection, retry logic, and decline classification to maximize transaction approval rates while respecting business constraints like cost optimization.

The engine is **stateless** — each routing call is independent, making it safe for concurrent use in Vercel's serverless environment.

## Key Files

- **`mod.rs`** — The `RoutingEngine` struct and core orchestration logic. Exposes two public methods:
  - `route()` — Smart routing with retry: tries up to 3 PSPs (`MAX_ATTEMPTS`), cascading on soft declines, failing fast on hard declines, and skipping unavailable PSPs without counting them as attempts.
  - `route_no_retry()` — Single-attempt baseline: tries only the first available PSP. Used for performance comparison against the smart routing strategy.
  - Also contains `build_attempt()`, a private helper that converts a `PspResponse` into a `RoutingAttempt`.

- **`retry.rs`** — Decline classification functions that determine retry behavior:
  - `is_hard_decline()` — `InsufficientFunds`, `CardExpired`, `InvalidCard`, `StolenCard`. No retry — fail immediately.
  - `is_soft_decline()` — `IssuerUnavailable`, `SuspectedFraud`, `DoNotHonor`, `ProcessorDeclined`. Retry with the next PSP.
  - `is_psp_unavailable()` — `PspUnavailable`. Cascade immediately without counting as a decline attempt.
  - Every `DeclineReason` variant belongs to exactly one category (enforced by tests).

- **`strategy.rs`** — PSP selection strategies via `select_psp_order()`, which sorts PSPs based on the chosen `RoutingStrategy`:
  - `OptimizeForApprovals` — Highest `base_success_rate` first.
  - `OptimizeForCost` — Lowest total fee (`fee_percentage + fee_fixed_cents/100`) first.
  - `Balanced` — Weighted score: `success_rate * 0.7 + (1.0 - normalized_fee) * 0.3`. Higher score wins.

## Dependencies

- **`crate::models::psp`** — `PspResponse`, `PspConfig`, `DeclineReason`.
- **`crate::models::routing`** — `RoutingAttempt`, `RoutingResult`, `RoutingStrategy`.
- **`crate::models::transaction`** — `Transaction`.
- **`crate::simulator`** — `PspSimulator` (processes transactions against a PSP) and `config::get_psps_for_country()` (returns available PSPs per country).

## Conventions

- **Engine stays stateless.** `RoutingEngine` holds only a `PspSimulator` reference. No mutable state, no caching between calls.
- **Thin orchestration in `mod.rs`.** The `route()` method delegates classification to `retry` and ordering to `strategy`. Keep it as a coordinator, not a decision-maker.
- **Exhaustive decline classification.** Every `DeclineReason` variant must be handled by exactly one of `is_hard_decline`, `is_soft_decline`, or `is_psp_unavailable`. Adding a new variant requires updating one of these functions and the exhaustiveness test.
- **Pure functions in `retry.rs` and `strategy.rs`.** No side effects, no state — input in, answer out. This makes them trivially testable.
- **`select_psp_order()` does not mutate the input.** It clones and sorts a new `Vec`, leaving the original untouched.

## Testing

Each file has a `#[cfg(test)]` module with unit tests. Notable patterns:

- `mod.rs` tests exercise the full routing loop across all countries and strategies, verifying attempt counts, latency sums, and `final_psp` semantics.
- `retry.rs` tests assert that every `DeclineReason` maps to exactly one category — no overlap, no gaps.
- `strategy.rs` tests verify sort order for each strategy with controlled PSP configs, plus edge cases (empty list, single PSP, immutability).

Run with `cargo test` from the project root.
