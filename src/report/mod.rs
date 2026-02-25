/// Performance report generation for the routing engine.
///
/// Compares no-retry vs smart-retry routing scenarios and
/// quantifies the business impact of intelligent routing.
use crate::engine::RoutingEngine;
use crate::models::psp::DeclineReason;
use crate::models::report::{
    CountryMetrics, ImprovementMetrics, PerformanceReport, PspMetrics, ScenarioResult,
};
use crate::models::routing::{RoutingResult, RoutingStrategy};
use crate::models::transaction::Transaction;
use std::collections::HashMap;

/// Generate a complete performance report comparing no-retry vs smart-retry.
///
/// Runs every transaction through both scenarios (single-PSP and full routing),
/// then computes aggregate metrics, country breakdowns, PSP breakdowns, and
/// the business impact of switching to smart retry.
pub fn generate_report(
    transactions: &[Transaction],
    engine: &RoutingEngine,
    strategy: &RoutingStrategy,
) -> PerformanceReport {
    let no_retry_results = run_no_retry(transactions, engine);
    let smart_retry_results = run_smart_retry(transactions, engine, strategy);

    let no_retry_metrics = calculate_metrics(&no_retry_results);
    let smart_retry_metrics = calculate_metrics(&smart_retry_results);

    let additional_approvals = smart_retry_metrics
        .approved
        .saturating_sub(no_retry_metrics.approved);
    let rate_lift = smart_retry_metrics.authorization_rate - no_retry_metrics.authorization_rate;

    let avg_transaction_value = if transactions.is_empty() {
        0.0
    } else {
        transactions.iter().map(|t| t.amount).sum::<f64>() / transactions.len() as f64
    };

    let improvement = ImprovementMetrics {
        rate_lift_percentage: round2(rate_lift),
        additional_approvals,
        estimated_revenue_recovered_usd: round2(
            additional_approvals as f64 * avg_transaction_value,
        ),
    };

    let by_country = build_country_breakdown(transactions, &no_retry_results, &smart_retry_results);

    let by_psp = build_psp_breakdown(&smart_retry_results);

    PerformanceReport {
        total_transactions: transactions.len(),
        no_retry: no_retry_metrics,
        smart_retry: smart_retry_metrics,
        improvement,
        by_country,
        by_psp,
    }
}

/// Run all transactions in no-retry mode (single PSP, fail on any decline).
fn run_no_retry(transactions: &[Transaction], engine: &RoutingEngine) -> Vec<RoutingResult> {
    transactions
        .iter()
        .map(|txn| engine.route_no_retry(txn))
        .collect()
}

/// Run all transactions with smart retry (full routing engine).
fn run_smart_retry(
    transactions: &[Transaction],
    engine: &RoutingEngine,
    strategy: &RoutingStrategy,
) -> Vec<RoutingResult> {
    transactions
        .iter()
        .map(|txn| engine.route(txn, strategy))
        .collect()
}

/// Calculate aggregate metrics from a set of routing results.
fn calculate_metrics(results: &[RoutingResult]) -> ScenarioResult {
    if results.is_empty() {
        return ScenarioResult {
            approved: 0,
            declined: 0,
            authorization_rate: 0.0,
            avg_attempts: 0.0,
            avg_latency_ms: 0.0,
        };
    }

    let total = results.len();
    let approved = results.iter().filter(|r| r.approved).count();
    let declined = total - approved;

    let total_attempts: usize = results.iter().map(|r| r.total_attempts).sum();
    let total_latency: u64 = results.iter().map(|r| r.total_latency_ms).sum();

    ScenarioResult {
        approved,
        declined,
        authorization_rate: round2(approved as f64 / total as f64 * 100.0),
        avg_attempts: round2(total_attempts as f64 / total as f64),
        avg_latency_ms: round2(total_latency as f64 / total as f64),
    }
}

/// Build per-country authorization rate breakdown.
fn build_country_breakdown(
    transactions: &[Transaction],
    no_retry_results: &[RoutingResult],
    smart_retry_results: &[RoutingResult],
) -> HashMap<String, CountryMetrics> {
    let mut country_map: HashMap<String, CountryMetrics> = HashMap::new();

    // Index results by transaction_id for fast lookup.
    let no_retry_by_id: HashMap<&str, &RoutingResult> = no_retry_results
        .iter()
        .map(|r| (r.transaction_id.as_str(), r))
        .collect();
    let smart_by_id: HashMap<&str, &RoutingResult> = smart_retry_results
        .iter()
        .map(|r| (r.transaction_id.as_str(), r))
        .collect();

    // Group transactions by country.
    let mut by_country: HashMap<String, Vec<&Transaction>> = HashMap::new();
    for txn in transactions {
        by_country
            .entry(txn.country.to_string())
            .or_default()
            .push(txn);
    }

    for (country, txns) in &by_country {
        let total = txns.len();
        if total == 0 {
            continue;
        }

        let no_retry_approved = txns
            .iter()
            .filter(|t| {
                no_retry_by_id
                    .get(t.id.as_str())
                    .is_some_and(|r| r.approved)
            })
            .count();

        let smart_approved = txns
            .iter()
            .filter(|t| smart_by_id.get(t.id.as_str()).is_some_and(|r| r.approved))
            .count();

        let no_retry_rate = round2(no_retry_approved as f64 / total as f64 * 100.0);
        let smart_retry_rate = round2(smart_approved as f64 / total as f64 * 100.0);

        country_map.insert(
            country.clone(),
            CountryMetrics {
                no_retry_rate,
                smart_retry_rate,
                improvement: round2(smart_retry_rate - no_retry_rate),
                total_transactions: total,
            },
        );
    }

    country_map
}

/// Build per-PSP performance breakdown from smart-retry results.
fn build_psp_breakdown(results: &[RoutingResult]) -> HashMap<String, PspMetrics> {
    let mut psp_map: HashMap<String, (usize, usize, usize, u64)> = HashMap::new();

    for result in results {
        for attempt in &result.attempts {
            let entry = psp_map
                .entry(attempt.psp_name.clone())
                .or_insert((0, 0, 0, 0));
            entry.0 += 1; // total_attempts
            if attempt.approved {
                entry.1 += 1; // approvals
            } else {
                // Only count actual declines, not PSP unavailable cascades.
                let is_cascade = attempt
                    .decline_reason
                    .as_ref()
                    .is_some_and(|r| matches!(r, DeclineReason::PspUnavailable));
                if !is_cascade {
                    entry.2 += 1; // declines
                }
            }
            entry.3 += attempt.latency_ms; // total_latency
        }
    }

    psp_map
        .into_iter()
        .map(|(name, (total, approvals, declines, total_latency))| {
            let approval_rate = if total > 0 {
                round2(approvals as f64 / total as f64 * 100.0)
            } else {
                0.0
            };
            let avg_latency = if total > 0 {
                round2(total_latency as f64 / total as f64)
            } else {
                0.0
            };
            (
                name,
                PspMetrics {
                    total_attempts: total,
                    approvals,
                    declines,
                    approval_rate,
                    avg_latency_ms: avg_latency,
                },
            )
        })
        .collect()
}

/// Round a floating-point value to 2 decimal places.
fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::routing::{RoutingAttempt, RoutingResult};
    use crate::models::transaction::{Country, Currency, Transaction};

    fn make_transaction(id: &str, country: Country, amount: f64) -> Transaction {
        let (currency, bin) = match country {
            Country::Brazil => (Currency::BRL, "411111"),
            Country::Mexico => (Currency::MXN, "424242"),
            Country::Colombia => (Currency::COP, "431940"),
        };
        Transaction {
            id: id.to_string(),
            amount,
            currency,
            country,
            card_bin: bin.to_string(),
            card_last4: "1234".to_string(),
            customer_id: "cust_001".to_string(),
            timestamp: "2025-01-15T10:00:00Z".to_string(),
        }
    }

    fn make_result(txn_id: &str, approved: bool, attempts: usize, latency: u64) -> RoutingResult {
        let attempt_list: Vec<RoutingAttempt> = (1..=attempts)
            .map(|i| RoutingAttempt {
                psp_id: format!("psp_{}", i),
                psp_name: format!("PSP_{}", i),
                approved: i == attempts && approved,
                decline_reason: if i == attempts && approved {
                    None
                } else {
                    Some(DeclineReason::IssuerUnavailable)
                },
                latency_ms: latency / attempts as u64,
                attempt_number: i,
            })
            .collect();

        RoutingResult {
            transaction_id: txn_id.to_string(),
            approved,
            final_psp: if approved {
                Some(format!("PSP_{}", attempts))
            } else {
                None
            },
            attempts: attempt_list,
            total_attempts: attempts,
            total_latency_ms: latency,
        }
    }

    #[test]
    fn test_calculate_metrics_empty() {
        let result = calculate_metrics(&[]);
        assert_eq!(result.approved, 0);
        assert_eq!(result.declined, 0);
        assert_eq!(result.authorization_rate, 0.0);
    }

    #[test]
    fn test_calculate_metrics_basic() {
        let results = vec![
            make_result("txn_1", true, 1, 200),
            make_result("txn_2", false, 1, 300),
            make_result("txn_3", true, 2, 400),
            make_result("txn_4", true, 1, 150),
        ];
        let metrics = calculate_metrics(&results);
        assert_eq!(metrics.approved, 3);
        assert_eq!(metrics.declined, 1);
        assert_eq!(metrics.authorization_rate, 75.0);
        assert_eq!(metrics.avg_attempts, 1.25);
        assert_eq!(metrics.avg_latency_ms, 262.5);
    }

    #[test]
    fn test_round2() {
        assert_eq!(round2(78.123456), 78.12);
        assert_eq!(round2(0.0), 0.0);
        assert_eq!(round2(99.999), 100.0);
    }

    #[test]
    fn test_build_country_breakdown() {
        let transactions = vec![
            make_transaction("txn_1", Country::Brazil, 100.0),
            make_transaction("txn_2", Country::Brazil, 200.0),
            make_transaction("txn_3", Country::Mexico, 150.0),
        ];

        let no_retry = vec![
            make_result("txn_1", true, 1, 200),
            make_result("txn_2", false, 1, 300),
            make_result("txn_3", true, 1, 250),
        ];
        let smart = vec![
            make_result("txn_1", true, 1, 200),
            make_result("txn_2", true, 2, 500),
            make_result("txn_3", true, 1, 250),
        ];

        let breakdown = build_country_breakdown(&transactions, &no_retry, &smart);

        let brazil = breakdown.get("Brazil").unwrap();
        assert_eq!(brazil.no_retry_rate, 50.0);
        assert_eq!(brazil.smart_retry_rate, 100.0);
        assert_eq!(brazil.improvement, 50.0);
        assert_eq!(brazil.total_transactions, 2);

        let mexico = breakdown.get("Mexico").unwrap();
        assert_eq!(mexico.no_retry_rate, 100.0);
        assert_eq!(mexico.smart_retry_rate, 100.0);
    }

    #[test]
    fn test_build_psp_breakdown() {
        let results = vec![
            make_result("txn_1", true, 2, 400),
            make_result("txn_2", false, 1, 200),
        ];
        let breakdown = build_psp_breakdown(&results);
        assert!(breakdown.contains_key("PSP_1"));
        assert!(breakdown.contains_key("PSP_2"));
    }
}
