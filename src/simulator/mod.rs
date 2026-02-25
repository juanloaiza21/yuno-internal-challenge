/// PSP Simulator — simulates realistic payment processor behavior.
///
/// Each PSP has different success rates, decline reason distributions,
/// and response latencies. The simulator uses deterministic seeding
/// based on transaction + PSP attributes for reproducible results.
///
/// # Simulation Design
///
/// The simulator produces three tiers of outcomes:
/// - **Hard declines (~6%)**: Card-level failures (insufficient funds, expired card).
///   These are PSP-independent — the same card always hard-declines regardless of PSP.
/// - **Soft declines (~20-25%)**: PSP-dependent failures that may succeed on retry.
///   Different PSPs get different seeds, so PSP#1 may decline while PSP#2 approves.
/// - **Approvals (~70-75%)**: Transaction is approved.
///
/// This design ensures that smart retry logic produces measurable improvement
/// over single-PSP routing.

pub mod config;

use crate::models::psp::{DeclineReason, PspConfig, PspResponse};
use crate::models::transaction::Transaction;
use config::get_decline_distribution;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hard decline rate — percentage of cards that always fail regardless of PSP.
const HARD_DECLINE_RATE: f64 = 0.06;

/// PSP unavailability rate — percentage of requests that hit a downed PSP.
/// Used for the real-time cascading stretch goal.
const PSP_UNAVAILABLE_RATE: f64 = 0.08;

/// Simulates PSP behavior for transaction processing.
///
/// The simulator is stateless — all randomness is derived from
/// deterministic seeds, making results reproducible across runs.
pub struct PspSimulator;

impl PspSimulator {
    /// Creates a new PSP simulator instance.
    pub fn new() -> Self {
        PspSimulator
    }

    /// Simulate a PSP processing a transaction.
    ///
    /// Uses deterministic seeding from transaction and PSP attributes
    /// to produce reproducible but realistic outcomes.
    ///
    /// # Decision Flow
    /// 1. Check if the card is a "hard decline card" (PSP-independent)
    /// 2. Check if this PSP is temporarily unavailable (cascading)
    /// 3. Roll against PSP's success rate (PSP-dependent seed)
    /// 4. If declined, select a soft decline reason from the PSP's distribution
    pub fn process(&self, transaction: &Transaction, psp: &PspConfig) -> PspResponse {
        let latency_ms = self.simulate_latency(transaction, psp);

        // Step 1: Check for hard decline (card-level, PSP-independent)
        if self.is_hard_decline_card(&transaction.card_bin, &transaction.card_last4) {
            let reason = self.select_hard_decline_reason(&transaction.card_bin, &transaction.card_last4);
            return PspResponse {
                psp_id: psp.id.clone(),
                psp_name: psp.name.clone(),
                approved: false,
                decline_reason: Some(reason),
                latency_ms,
            };
        }

        // Step 2: Check for PSP unavailability (stretch: cascading)
        if self.is_psp_unavailable(transaction, psp) {
            return PspResponse {
                psp_id: psp.id.clone(),
                psp_name: psp.name.clone(),
                approved: false,
                decline_reason: Some(DeclineReason::PspUnavailable),
                latency_ms,
            };
        }

        // Step 3: Roll against PSP's success rate (PSP-dependent)
        let seed = self.make_psp_seed(
            &transaction.card_bin,
            &transaction.card_last4,
            &psp.id,
            transaction.amount,
        );
        let mut rng = StdRng::seed_from_u64(seed);
        let roll: f64 = rng.gen();

        if roll < psp.base_success_rate {
            // Approved
            PspResponse {
                psp_id: psp.id.clone(),
                psp_name: psp.name.clone(),
                approved: true,
                decline_reason: None,
                latency_ms,
            }
        } else {
            // Step 4: Soft decline — pick reason from PSP's distribution
            let reason = self.select_soft_decline_reason(&mut rng, &psp.id);
            PspResponse {
                psp_id: psp.id.clone(),
                psp_name: psp.name.clone(),
                approved: false,
                decline_reason: Some(reason),
                latency_ms,
            }
        }
    }

    /// Determines if a card always hard-declines regardless of PSP.
    ///
    /// Uses a seed derived only from card attributes (no PSP ID),
    /// ensuring the same card behaves consistently across all PSPs.
    fn is_hard_decline_card(&self, card_bin: &str, card_last4: &str) -> bool {
        let seed = self.make_card_seed(card_bin, card_last4);
        let mut rng = StdRng::seed_from_u64(seed);
        let roll: f64 = rng.gen();
        roll < HARD_DECLINE_RATE
    }

    /// Selects a hard decline reason based on the card.
    fn select_hard_decline_reason(&self, card_bin: &str, card_last4: &str) -> DeclineReason {
        let seed = self.make_card_seed(card_bin, card_last4).wrapping_add(1);
        let mut rng = StdRng::seed_from_u64(seed);
        let roll: f64 = rng.gen();

        if roll < 0.45 {
            DeclineReason::InsufficientFunds
        } else if roll < 0.75 {
            DeclineReason::CardExpired
        } else if roll < 0.90 {
            DeclineReason::InvalidCard
        } else {
            DeclineReason::StolenCard
        }
    }

    /// Checks if a PSP is temporarily unavailable for this request.
    fn is_psp_unavailable(&self, transaction: &Transaction, psp: &PspConfig) -> bool {
        let mut hasher = DefaultHasher::new();
        transaction.id.hash(&mut hasher);
        psp.id.hash(&mut hasher);
        "unavailable_check".hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(seed);
        let roll: f64 = rng.gen();
        roll < PSP_UNAVAILABLE_RATE
    }

    /// Selects a soft decline reason based on the PSP's decline distribution.
    fn select_soft_decline_reason(&self, rng: &mut StdRng, psp_id: &str) -> DeclineReason {
        let distribution = get_decline_distribution(psp_id);
        let roll: f64 = rng.gen();

        let mut cumulative = 0.0;
        for dw in &distribution {
            cumulative += dw.weight;
            if roll < cumulative {
                return dw.reason.clone();
            }
        }

        // Fallback (should not reach here if weights sum to 1.0)
        distribution.last().map(|d| d.reason.clone()).unwrap_or(DeclineReason::ProcessorDeclined)
    }

    /// Simulates response latency within the PSP's configured range.
    fn simulate_latency(&self, transaction: &Transaction, psp: &PspConfig) -> u64 {
        let mut hasher = DefaultHasher::new();
        transaction.id.hash(&mut hasher);
        psp.id.hash(&mut hasher);
        "latency".hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(seed);

        rng.gen_range(psp.latency_min_ms..=psp.latency_max_ms)
    }

    /// Creates a deterministic seed from card attributes only (PSP-independent).
    fn make_card_seed(&self, card_bin: &str, card_last4: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        card_bin.hash(&mut hasher);
        card_last4.hash(&mut hasher);
        "card_seed".hash(&mut hasher);
        hasher.finish()
    }

    /// Creates a deterministic seed from card + PSP + amount (PSP-dependent).
    ///
    /// This is the key to making retry valuable: the same card may produce
    /// different outcomes with different PSPs because the PSP ID changes the seed.
    fn make_psp_seed(&self, card_bin: &str, card_last4: &str, psp_id: &str, amount: f64) -> u64 {
        let mut hasher = DefaultHasher::new();
        card_bin.hash(&mut hasher);
        card_last4.hash(&mut hasher);
        psp_id.hash(&mut hasher);
        ((amount * 100.0) as u64).hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for PspSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Country, Currency, Transaction};
    use crate::simulator::config::get_psps_for_country;

    fn make_test_transaction(bin: &str, last4: &str, amount: f64) -> Transaction {
        Transaction {
            id: format!("test_{}_{}", bin, last4),
            amount,
            currency: Currency::BRL,
            country: Country::Brazil,
            card_bin: bin.to_string(),
            card_last4: last4.to_string(),
            customer_id: "test_cust".to_string(),
            timestamp: "2025-01-15T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_simulator_is_deterministic() {
        let sim = PspSimulator::new();
        let tx = make_test_transaction("411111", "1234", 100.0);
        let psps = get_psps_for_country(&Country::Brazil);
        let psp = &psps[0];

        let r1 = sim.process(&tx, psp);
        let r2 = sim.process(&tx, psp);

        assert_eq!(r1.approved, r2.approved);
        assert_eq!(r1.decline_reason, r2.decline_reason);
    }

    #[test]
    fn test_different_psps_can_produce_different_results() {
        let sim = PspSimulator::new();
        let psps = get_psps_for_country(&Country::Brazil);

        // Try many cards — at least some should differ between PSPs
        let mut found_difference = false;
        for i in 0..100 {
            let tx = make_test_transaction("411111", &format!("{:04}", i), 150.0);
            let r1 = sim.process(&tx, &psps[0]);
            let r2 = sim.process(&tx, &psps[1]);

            // Skip hard declines (same across PSPs)
            if r1.decline_reason.as_ref().map_or(false, |r| r.is_hard_decline()) {
                continue;
            }

            if r1.approved != r2.approved {
                found_difference = true;
                break;
            }
        }
        assert!(found_difference, "Different PSPs should produce different outcomes for some cards");
    }

    #[test]
    fn test_hard_declines_are_psp_independent() {
        let sim = PspSimulator::new();
        let psps = get_psps_for_country(&Country::Brazil);

        for i in 0..200 {
            let tx = make_test_transaction("411111", &format!("{:04}", i), 100.0);
            let r1 = sim.process(&tx, &psps[0]);

            if let Some(ref reason) = r1.decline_reason {
                if reason.is_hard_decline() {
                    // All PSPs should also hard-decline this card
                    for psp in &psps[1..] {
                        let r = sim.process(&tx, psp);
                        assert!(!r.approved, "Hard decline card should fail on all PSPs");
                        assert!(
                            r.decline_reason.as_ref().map_or(false, |r| r.is_hard_decline()
                                || r.is_psp_unavailable()),
                            "Hard decline card should return hard decline reason"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_latency_within_range() {
        let sim = PspSimulator::new();
        let psps = get_psps_for_country(&Country::Brazil);

        for i in 0..50 {
            let tx = make_test_transaction("411111", &format!("{:04}", i), 100.0);
            for psp in &psps {
                let r = sim.process(&tx, psp);
                assert!(r.latency_ms >= psp.latency_min_ms && r.latency_ms <= psp.latency_max_ms,
                    "Latency {} not in range [{}, {}] for PSP {}",
                    r.latency_ms, psp.latency_min_ms, psp.latency_max_ms, psp.name);
            }
        }
    }

    #[test]
    fn test_approval_rate_distribution() {
        let sim = PspSimulator::new();
        let psps = get_psps_for_country(&Country::Brazil);
        let psp = &psps[0]; // PagSeguro, 78% rate

        let mut approved = 0;
        let total = 1000;
        for i in 0..total {
            let tx = make_test_transaction("411111", &format!("{:04}", i), 100.0 + i as f64);
            let r = sim.process(&tx, psp);
            if r.approved {
                approved += 1;
            }
        }

        let rate = approved as f64 / total as f64;
        // Should be roughly in the ballpark (60-90% given hard declines + unavailable)
        assert!(rate > 0.55 && rate < 0.95,
            "Approval rate {} is out of expected range for PSP with 78% base rate", rate);
    }
}
