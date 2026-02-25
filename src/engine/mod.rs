/// Routing Engine — intelligent transaction routing with retry logic.
///
/// Routes transactions through multiple PSPs, retrying on soft declines
/// and failing fast on hard declines.

pub mod retry;
pub mod strategy;

use crate::models::routing::{RoutingAttempt, RoutingResult, RoutingStrategy};
use crate::models::transaction::Transaction;
use crate::simulator::config::get_psps_for_country;
use crate::simulator::PspSimulator;

/// The core routing engine that orchestrates PSP selection and retry logic.
pub struct RoutingEngine {
    simulator: PspSimulator,
}

impl RoutingEngine {
    /// Creates a new routing engine with the given PSP simulator.
    pub fn new(simulator: PspSimulator) -> Self {
        RoutingEngine { simulator }
    }

    /// Route a transaction with smart retry logic.
    ///
    /// # Stub Implementation
    /// Currently returns a dummy approved result. Will be replaced
    /// by Instance 2 (feature/routing-engine branch).
    pub fn route(&self, transaction: &Transaction, strategy: &RoutingStrategy) -> RoutingResult {
        let psps = get_psps_for_country(&transaction.country);
        let psp = psps.first().expect("No PSPs configured for country");
        let response = self.simulator.process(transaction, psp);
        let _ = strategy;

        RoutingResult {
            transaction_id: transaction.id.clone(),
            approved: response.approved,
            final_psp: if response.approved {
                Some(response.psp_name.clone())
            } else {
                None
            },
            attempts: vec![RoutingAttempt {
                psp_id: response.psp_id,
                psp_name: response.psp_name,
                approved: response.approved,
                decline_reason: response.decline_reason,
                latency_ms: response.latency_ms,
                attempt_number: 1,
            }],
            total_attempts: 1,
            total_latency_ms: response.latency_ms,
        }
    }

    /// Route with no retry — single PSP attempt only.
    /// Used for comparison in performance reports.
    pub fn route_no_retry(&self, transaction: &Transaction) -> RoutingResult {
        self.route(transaction, &RoutingStrategy::OptimizeForApprovals)
    }
}
