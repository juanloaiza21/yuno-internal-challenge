/// PSP selection strategies for the routing engine.
///
/// Determines the order in which PSPs are tried for a transaction.

use crate::models::psp::PspConfig;
use crate::models::routing::RoutingStrategy;

/// Order PSPs based on the chosen routing strategy.
///
/// # Stub Implementation
/// Currently returns PSPs in their original order. Will be replaced
/// by Instance 2 (feature/routing-engine branch).
pub fn select_psp_order(psps: &[PspConfig], strategy: &RoutingStrategy) -> Vec<PspConfig> {
    let _ = strategy;
    psps.to_vec()
}
