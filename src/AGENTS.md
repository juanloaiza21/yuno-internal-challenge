# src/ — Shared Library Crate

## Purpose

This is the shared library crate (`lib.rs`) imported by every serverless handler in `api/` as `yuno_internal_challenge::<module>`. All business logic, domain types, models, and utilities live here. Handlers in `api/` should remain thin wrappers that delegate to this crate.

## Current Modules

- **`lib.rs`** — Crate root. Exposes `version()` helper and re-exports all public modules.

- **`report/mod.rs`** — Performance report generator. Compares no-retry vs smart-retry routing scenarios.
  - `generate_report(transactions, engine, strategy) → PerformanceReport` — Main entry point. Runs both scenarios, computes metrics, builds country and PSP breakdowns.
  - `run_no_retry(transactions, engine) → Vec<RoutingResult>` — (private) Runs each transaction through single-PSP routing.
  - `run_smart_retry(transactions, engine, strategy) → Vec<RoutingResult>` — (private) Runs each transaction through full routing engine.
  - `calculate_metrics(results) → ScenarioResult` — (private) Computes aggregate metrics (approved count, auth rate, avg attempts, avg latency).
  - `build_country_breakdown(...)` — (private) Groups results by country and computes per-country auth rates.
  - `build_psp_breakdown(results)` — (private) Aggregates per-PSP metrics from routing attempts.
  - Includes unit tests for metrics calculation, country breakdown, PSP breakdown, and round2 helper.

## Adding New Modules

1. Create the file: `src/<module_name>.rs` (snake_case, singular noun preferred — e.g. `payment.rs`, `error.rs`, `util.rs`).
2. Declare it in `lib.rs` with `pub mod <module_name>;`.
3. Optionally re-export key types at the crate root for ergonomic imports: `pub use <module_name>::SomeType;`.
4. If a module grows large, promote it to a directory: `src/<module_name>/mod.rs` with sub-modules inside.

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Modules | `snake_case` | `payment_method.rs` |
| Types / Structs / Enums | `PascalCase` | `PaymentMethod` |
| Functions | `snake_case` | `validate_amount()` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT` |

## Code Style

- **Doc comments on all public items.** Use `///` for functions, structs, enums, and their fields. The first line should be a short summary; add a blank `///` line before longer explanations.
- **Result-based error handling.** Return `Result<T, E>` from fallible functions. Define domain error types (enum with `thiserror`) rather than stringly-typed errors.
- **No `.unwrap()` or `.expect()` in production code.** Use `?` propagation or explicit match arms. `unwrap` is acceptable only inside `#[cfg(test)]` blocks.
- **Small, focused functions.** Each function should do one thing. If a function exceeds ~30 lines, consider splitting it.
- **Descriptive names over comments.** Prefer `calculate_total_with_tax()` over `calc()` with a comment explaining what it does.

## Testing

Unit tests live in the same file as the code they test, inside a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
```

- Name test functions `test_<what_it_verifies>`.
- Run tests with `cargo test` from the project root.
- Keep tests fast — no network calls or file I/O in unit tests. Use mocks or trait-based abstractions for external dependencies.
