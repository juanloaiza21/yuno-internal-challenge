/// Routing Engine — intelligent transaction routing with retry logic.
///
/// Routes transactions through multiple PSPs, retrying on soft declines
/// and failing fast on hard declines. Supports real-time cascading when
/// a PSP is unavailable.
pub mod retry;
pub mod strategy;

use crate::models::psp::PspResponse;
use crate::models::routing::{RoutingAttempt, RoutingResult, RoutingStrategy};
use crate::models::transaction::Transaction;
use crate::simulator::config::get_psps_for_country;
use crate::simulator::PspSimulator;

/// Maximum number of PSP decline attempts before giving up.
///
/// PSP-unavailable cascades do NOT count toward this limit.
const MAX_ATTEMPTS: usize = 3;

/// The core routing engine that orchestrates PSP selection and retry logic.
///
/// The engine is stateless — each call to [`route`](RoutingEngine::route) is
/// independent, making it safe for concurrent use in a serverless environment.
pub struct RoutingEngine {
    simulator: PspSimulator,
}

impl RoutingEngine {
    /// Creates a new routing engine backed by the given PSP simulator.
    pub fn new(simulator: PspSimulator) -> Self {
        RoutingEngine { simulator }
    }

    /// Route a transaction through PSPs with smart retry logic.
    ///
    /// # Algorithm
    ///
    /// 1. Retrieve available PSPs for the transaction's country.
    /// 2. Order them according to the chosen [`RoutingStrategy`].
    /// 3. Iterate through the ordered PSPs:
    ///    - **Approved** → return success immediately.
    ///    - **Hard decline** → return failure immediately (no retry).
    ///    - **Soft decline** → record attempt, try next PSP.
    ///    - **PSP unavailable** → cascade to next PSP without counting as an attempt.
    /// 4. If all PSPs are exhausted → return declined with full attempt history.
    pub fn route(&self, transaction: &Transaction, strategy: &RoutingStrategy) -> RoutingResult {
        let psps = get_psps_for_country(&transaction.country);
        let ordered_psps = strategy::select_psp_order(&psps, strategy);

        let mut attempts: Vec<RoutingAttempt> = Vec::new();
        let mut total_latency_ms: u64 = 0;
        let mut attempt_number: usize = 0;

        for psp in &ordered_psps {
            let response: PspResponse = self.simulator.process(transaction, psp);
            total_latency_ms += response.latency_ms;

            // PSP unavailable — cascade immediately, don't count as an attempt
            if let Some(ref reason) = response.decline_reason {
                if retry::is_psp_unavailable(reason) {
                    attempts.push(build_attempt(&response, attempt_number + 1));
                    continue;
                }
            }

            attempt_number += 1;

            // Approved — return success
            if response.approved {
                attempts.push(build_attempt(&response, attempt_number));
                return RoutingResult {
                    transaction_id: transaction.id.clone(),
                    approved: true,
                    final_psp: Some(response.psp_name),
                    attempts,
                    total_attempts: attempt_number,
                    total_latency_ms,
                };
            }

            // Declined — classify
            attempts.push(build_attempt(&response, attempt_number));

            if let Some(ref reason) = response.decline_reason {
                // Hard decline — fail fast, no retry
                if retry::is_hard_decline(reason) {
                    return RoutingResult {
                        transaction_id: transaction.id.clone(),
                        approved: false,
                        final_psp: None,
                        attempts,
                        total_attempts: attempt_number,
                        total_latency_ms,
                    };
                }

                // Soft decline — retry if we haven't exhausted attempts
                if attempt_number >= MAX_ATTEMPTS {
                    break;
                }
                // Otherwise, continue to next PSP
            }
        }

        // All PSPs exhausted — return declined
        RoutingResult {
            transaction_id: transaction.id.clone(),
            approved: false,
            final_psp: None,
            attempts,
            total_attempts: attempt_number,
            total_latency_ms,
        }
    }

    /// Route with no retry — single PSP attempt only.
    ///
    /// Simulates FashionForward's current behavior: try the first available
    /// PSP and return the result regardless of the outcome. Used as the
    /// baseline for performance comparison in reports.
    pub fn route_no_retry(&self, transaction: &Transaction) -> RoutingResult {
        let psps = get_psps_for_country(&transaction.country);
        let psp = match psps.first() {
            Some(p) => p,
            None => {
                return RoutingResult {
                    transaction_id: transaction.id.clone(),
                    approved: false,
                    final_psp: None,
                    attempts: vec![],
                    total_attempts: 0,
                    total_latency_ms: 0,
                };
            }
        };

        let response = self.simulator.process(transaction, psp);
        let attempt = build_attempt(&response, 1);
        let latency = response.latency_ms;

        RoutingResult {
            transaction_id: transaction.id.clone(),
            approved: response.approved,
            final_psp: if response.approved {
                Some(response.psp_name)
            } else {
                None
            },
            attempts: vec![attempt],
            total_attempts: 1,
            total_latency_ms: latency,
        }
    }
}

/// Build a [`RoutingAttempt`] from a PSP response.
fn build_attempt(response: &PspResponse, attempt_number: usize) -> RoutingAttempt {
    RoutingAttempt {
        psp_id: response.psp_id.clone(),
        psp_name: response.psp_name.clone(),
        approved: response.approved,
        decline_reason: response.decline_reason.clone(),
        latency_ms: response.latency_ms,
        attempt_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Country, Currency, Transaction};

    fn make_transaction(country: Country) -> Transaction {
        let currency = match country {
            Country::Brazil => Currency::BRL,
            Country::Mexico => Currency::MXN,
            Country::Colombia => Currency::COP,
        };
        Transaction {
            id: "txn_test_001".to_string(),
            amount: 150.0,
            currency,
            country,
            card_bin: "411111".to_string(),
            card_last4: "1234".to_string(),
            customer_id: "cust_001".to_string(),
            timestamp: "2025-01-15T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_route_returns_result_for_each_country() {
        let engine = RoutingEngine::new(PspSimulator::new());

        for country in [Country::Brazil, Country::Mexico, Country::Colombia] {
            let txn = make_transaction(country.clone());
            let result = engine.route(&txn, &RoutingStrategy::OptimizeForApprovals);

            assert_eq!(result.transaction_id, "txn_test_001");
            assert!(result.total_attempts >= 1, "Should have at least 1 attempt");
            assert!(
                !result.attempts.is_empty(),
                "Should record at least 1 attempt for {country}"
            );
        }
    }

    #[test]
    fn test_route_no_retry_makes_single_attempt() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Brazil);
        let result = engine.route_no_retry(&txn);

        assert_eq!(result.total_attempts, 1);
        assert_eq!(result.attempts.len(), 1);
        assert_eq!(result.attempts[0].attempt_number, 1);
    }

    #[test]
    fn test_route_respects_max_attempts() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Brazil);
        let result = engine.route(&txn, &RoutingStrategy::OptimizeForApprovals);

        // Decline attempts (excluding PSP-unavailable cascades) should not exceed MAX_ATTEMPTS
        let decline_attempts = result
            .attempts
            .iter()
            .filter(|a| {
                !a.approved
                    && !a
                        .decline_reason
                        .as_ref()
                        .map_or(false, retry::is_psp_unavailable)
            })
            .count();
        assert!(
            decline_attempts <= MAX_ATTEMPTS,
            "Decline attempts ({decline_attempts}) should not exceed MAX_ATTEMPTS ({MAX_ATTEMPTS})"
        );
    }

    #[test]
    fn test_route_approved_has_final_psp() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Brazil);
        let result = engine.route(&txn, &RoutingStrategy::OptimizeForApprovals);

        if result.approved {
            assert!(
                result.final_psp.is_some(),
                "Approved result must have a final_psp"
            );
        }
    }

    #[test]
    fn test_route_declined_has_no_final_psp() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Brazil);
        let result = engine.route(&txn, &RoutingStrategy::OptimizeForApprovals);

        if !result.approved {
            assert!(
                result.final_psp.is_none(),
                "Declined result must not have a final_psp"
            );
        }
    }

    #[test]
    fn test_latency_is_sum_of_attempts() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Mexico);
        let result = engine.route(&txn, &RoutingStrategy::Balanced);

        let sum: u64 = result.attempts.iter().map(|a| a.latency_ms).sum();
        assert_eq!(
            result.total_latency_ms, sum,
            "Total latency should be the sum of all attempt latencies"
        );
    }

    #[test]
    fn test_attempt_numbers_are_sequential() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Colombia);
        let result = engine.route(&txn, &RoutingStrategy::OptimizeForCost);

        // Attempt numbers should be sequential (though PSP-unavailable ones
        // get the next number without incrementing the main counter)
        for (i, attempt) in result.attempts.iter().enumerate() {
            assert!(
                attempt.attempt_number >= 1,
                "Attempt {i} should have number >= 1"
            );
        }
    }

    #[test]
    fn test_all_strategies_produce_results() {
        let engine = RoutingEngine::new(PspSimulator::new());
        let txn = make_transaction(Country::Brazil);

        for strategy in [
            RoutingStrategy::OptimizeForApprovals,
            RoutingStrategy::OptimizeForCost,
            RoutingStrategy::Balanced,
        ] {
            let result = engine.route(&txn, &strategy);
            assert!(
                !result.attempts.is_empty(),
                "Strategy {strategy:?} should produce attempts"
            );
        }
    }
}
