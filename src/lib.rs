/// Yuno Internal Challenge — Shared Library
///
/// This crate contains the shared business logic, models,
/// utilities, and domain types used across all API handlers.
///
/// Each serverless function in `api/` imports from this library
/// to keep handlers thin and logic reusable.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
