/// PSP selection strategies for the routing engine.
///
/// Determines the order in which PSPs are tried for a transaction,
/// optimizing for different business objectives: approval rate, cost,
/// or a balanced combination of both.
use crate::models::psp::PspConfig;
use crate::models::routing::RoutingStrategy;

/// Order PSPs based on the chosen routing strategy.
///
/// Returns a new `Vec<PspConfig>` sorted according to the strategy:
/// - [`RoutingStrategy::OptimizeForApprovals`]: Highest success rate first.
/// - [`RoutingStrategy::OptimizeForCost`]: Lowest total fee first.
/// - [`RoutingStrategy::Balanced`]: Weighted score combining success rate (70%) and cost (30%).
pub fn select_psp_order(psps: &[PspConfig], strategy: &RoutingStrategy) -> Vec<PspConfig> {
    let mut sorted = psps.to_vec();

    match strategy {
        RoutingStrategy::OptimizeForApprovals => {
            sorted.sort_by(|a, b| {
                b.base_success_rate
                    .partial_cmp(&a.base_success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        RoutingStrategy::OptimizeForCost => {
            sorted.sort_by(|a, b| {
                let cost_a = total_fee(a);
                let cost_b = total_fee(b);
                cost_a
                    .partial_cmp(&cost_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        RoutingStrategy::Balanced => {
            let max_fee = sorted.iter().map(total_fee).fold(0.0_f64, f64::max);

            sorted.sort_by(|a, b| {
                let score_a = balanced_score(a, max_fee);
                let score_b = balanced_score(b, max_fee);
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    sorted
}

/// Calculate the total effective fee for a PSP.
///
/// Combines the percentage-based fee with the fixed fee (converted from cents to
/// a comparable unit) to produce a single cost metric for sorting.
fn total_fee(psp: &PspConfig) -> f64 {
    psp.fee_percentage + (psp.fee_fixed_cents as f64 / 100.0)
}

/// Calculate the balanced score for a PSP.
///
/// Score = `success_rate * 0.7 + (1.0 - normalized_fee) * 0.3`
///
/// The normalized fee is `total_fee / max_fee` across all PSPs in the set,
/// ensuring the cost component falls in `[0.0, 1.0]`. A higher score is better.
fn balanced_score(psp: &PspConfig, max_fee: f64) -> f64 {
    let normalized_fee = if max_fee > 0.0 {
        total_fee(psp) / max_fee
    } else {
        0.0
    };
    psp.base_success_rate * 0.7 + (1.0 - normalized_fee) * 0.3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::Country;

    fn make_psp(id: &str, success_rate: f64, fee_pct: f64, fee_fixed: u64) -> PspConfig {
        PspConfig {
            id: id.to_string(),
            name: id.to_string(),
            country: Country::Brazil,
            base_success_rate: success_rate,
            latency_min_ms: 100,
            latency_max_ms: 300,
            fee_percentage: fee_pct,
            fee_fixed_cents: fee_fixed,
        }
    }

    #[test]
    fn test_optimize_for_approvals_sorts_by_success_rate_descending() {
        let psps = vec![
            make_psp("low", 0.65, 2.5, 30),
            make_psp("high", 0.85, 3.5, 20),
            make_psp("mid", 0.75, 3.0, 25),
        ];

        let ordered = select_psp_order(&psps, &RoutingStrategy::OptimizeForApprovals);
        assert_eq!(ordered[0].id, "high");
        assert_eq!(ordered[1].id, "mid");
        assert_eq!(ordered[2].id, "low");
    }

    #[test]
    fn test_optimize_for_cost_sorts_by_total_fee_ascending() {
        let psps = vec![
            make_psp("expensive", 0.80, 3.5, 40), // 3.5 + 0.40 = 3.90
            make_psp("cheap", 0.70, 2.0, 20),     // 2.0 + 0.20 = 2.20
            make_psp("mid", 0.75, 2.8, 30),       // 2.8 + 0.30 = 3.10
        ];

        let ordered = select_psp_order(&psps, &RoutingStrategy::OptimizeForCost);
        assert_eq!(ordered[0].id, "cheap");
        assert_eq!(ordered[1].id, "mid");
        assert_eq!(ordered[2].id, "expensive");
    }

    #[test]
    fn test_balanced_strategy_weighs_success_and_cost() {
        // PSP A: high success, highest cost → fee normalized to 1.0, cost bonus = 0
        // PSP B: low success, lowest cost   → best cost bonus but weak success
        // PSP C: moderate both              → good success + decent cost bonus
        //
        // Scores (fee_max = 3.90):
        //   A: 0.90*0.7 + (1 - 1.000)*0.3 = 0.630
        //   B: 0.60*0.7 + (1 - 0.564)*0.3 = 0.551
        //   C: 0.78*0.7 + (1 - 0.705)*0.3 = 0.635
        let psps = vec![
            make_psp("A", 0.90, 3.5, 40), // total fee 3.90
            make_psp("B", 0.60, 2.0, 20), // total fee 2.20
            make_psp("C", 0.78, 2.5, 25), // total fee 2.75
        ];

        let ordered = select_psp_order(&psps, &RoutingStrategy::Balanced);

        // C edges out A because A's max fee zeroes out its cost bonus
        assert_eq!(ordered[0].id, "C");
        assert_eq!(ordered[1].id, "A");
        assert_eq!(ordered[2].id, "B");
    }

    #[test]
    fn test_empty_psp_list_returns_empty() {
        let psps: Vec<PspConfig> = vec![];
        let ordered = select_psp_order(&psps, &RoutingStrategy::OptimizeForApprovals);
        assert!(ordered.is_empty());
    }

    #[test]
    fn test_single_psp_returns_unchanged() {
        let psps = vec![make_psp("solo", 0.80, 3.0, 25)];
        let ordered = select_psp_order(&psps, &RoutingStrategy::Balanced);
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].id, "solo");
    }

    #[test]
    fn test_original_list_is_not_mutated() {
        let psps = vec![
            make_psp("low", 0.65, 2.5, 30),
            make_psp("high", 0.85, 3.5, 20),
        ];

        let _ = select_psp_order(&psps, &RoutingStrategy::OptimizeForApprovals);

        // Original order preserved
        assert_eq!(psps[0].id, "low");
        assert_eq!(psps[1].id, "high");
    }
}
