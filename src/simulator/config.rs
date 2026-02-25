//! PSP configurations for each country.
//!
//! FashionForward operates in Brazil, Mexico, and Colombia,
//! with 3 PSPs configured per country (9 total). Each PSP has
//! distinct success rates, latency profiles, fees, and decline
//! reason distributions based on real-world LatAm payment processors.

use crate::models::psp::{DeclineReason, PspConfig};
use crate::models::transaction::Country;

/// Weighted decline reason for a PSP.
/// The weight determines how likely this reason is relative to others.
#[derive(Debug, Clone)]
pub struct DeclineWeight {
    pub reason: DeclineReason,
    pub weight: f64,
}

/// Returns the soft-decline distribution for a given PSP.
///
/// Each PSP has a "bias" — one soft decline reason that occurs
/// more frequently than others, simulating real-world PSP behavior
/// (e.g., some PSPs have more issuer connectivity issues).
pub fn get_decline_distribution(psp_id: &str) -> Vec<DeclineWeight> {
    match psp_id {
        // Brazil — PagSeguro: issuer_unavailable heavy
        "psp_br_1" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.15 },
        ],
        // Brazil — Cielo: suspected_fraud heavy
        "psp_br_2" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.25 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.15 },
        ],
        // Brazil — Stone: do_not_honor heavy
        "psp_br_3" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.20 },
        ],
        // Mexico — Conekta: processor_declined heavy
        "psp_mx_1" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.45 },
        ],
        // Mexico — OpenPay: issuer_unavailable heavy
        "psp_mx_2" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.20 },
        ],
        // Mexico — SR Pago: suspected_fraud heavy
        "psp_mx_3" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.20 },
        ],
        // Colombia — PayU: do_not_honor heavy
        "psp_co_1" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.20 },
        ],
        // Colombia — Wompi: issuer_unavailable heavy
        "psp_co_2" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.45 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.15 },
        ],
        // Colombia — Bold: processor_declined heavy
        "psp_co_3" => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.15 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.20 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.45 },
        ],
        // Fallback: even distribution
        _ => vec![
            DeclineWeight { reason: DeclineReason::IssuerUnavailable, weight: 0.25 },
            DeclineWeight { reason: DeclineReason::SuspectedFraud, weight: 0.25 },
            DeclineWeight { reason: DeclineReason::DoNotHonor, weight: 0.25 },
            DeclineWeight { reason: DeclineReason::ProcessorDeclined, weight: 0.25 },
        ],
    }
}

/// Returns the list of PSP configurations for a given country.
///
/// Each country has 3 PSPs with different characteristics:
/// - A "primary" with moderate-to-good rates
/// - A "premium" with the best rates but higher fees
/// - A "budget" with lower rates but cheaper fees
pub fn get_psps_for_country(country: &Country) -> Vec<PspConfig> {
    match country {
        Country::Brazil => vec![
            PspConfig {
                id: "psp_br_1".to_string(),
                name: "PagSeguro".to_string(),
                country: Country::Brazil,
                base_success_rate: 0.78,
                latency_min_ms: 200,
                latency_max_ms: 400,
                fee_percentage: 2.9,
                fee_fixed_cents: 30,
            },
            PspConfig {
                id: "psp_br_2".to_string(),
                name: "Cielo".to_string(),
                country: Country::Brazil,
                base_success_rate: 0.82,
                latency_min_ms: 150,
                latency_max_ms: 250,
                fee_percentage: 3.2,
                fee_fixed_cents: 25,
            },
            PspConfig {
                id: "psp_br_3".to_string(),
                name: "Stone".to_string(),
                country: Country::Brazil,
                base_success_rate: 0.68,
                latency_min_ms: 300,
                latency_max_ms: 600,
                fee_percentage: 2.5,
                fee_fixed_cents: 35,
            },
        ],
        Country::Mexico => vec![
            PspConfig {
                id: "psp_mx_1".to_string(),
                name: "Conekta".to_string(),
                country: Country::Mexico,
                base_success_rate: 0.75,
                latency_min_ms: 180,
                latency_max_ms: 350,
                fee_percentage: 2.8,
                fee_fixed_cents: 28,
            },
            PspConfig {
                id: "psp_mx_2".to_string(),
                name: "OpenPay".to_string(),
                country: Country::Mexico,
                base_success_rate: 0.80,
                latency_min_ms: 200,
                latency_max_ms: 300,
                fee_percentage: 3.1,
                fee_fixed_cents: 22,
            },
            PspConfig {
                id: "psp_mx_3".to_string(),
                name: "SR Pago".to_string(),
                country: Country::Mexico,
                base_success_rate: 0.70,
                latency_min_ms: 250,
                latency_max_ms: 500,
                fee_percentage: 2.6,
                fee_fixed_cents: 32,
            },
        ],
        Country::Colombia => vec![
            PspConfig {
                id: "psp_co_1".to_string(),
                name: "PayU".to_string(),
                country: Country::Colombia,
                base_success_rate: 0.76,
                latency_min_ms: 190,
                latency_max_ms: 380,
                fee_percentage: 2.7,
                fee_fixed_cents: 29,
            },
            PspConfig {
                id: "psp_co_2".to_string(),
                name: "Wompi".to_string(),
                country: Country::Colombia,
                base_success_rate: 0.83,
                latency_min_ms: 160,
                latency_max_ms: 280,
                fee_percentage: 3.3,
                fee_fixed_cents: 20,
            },
            PspConfig {
                id: "psp_co_3".to_string(),
                name: "Bold".to_string(),
                country: Country::Colombia,
                base_success_rate: 0.65,
                latency_min_ms: 280,
                latency_max_ms: 550,
                fee_percentage: 2.4,
                fee_fixed_cents: 38,
            },
        ],
    }
}

/// Returns all PSP configurations across all countries (9 total).
pub fn get_all_psps() -> Vec<PspConfig> {
    let mut psps = Vec::with_capacity(9);
    psps.extend(get_psps_for_country(&Country::Brazil));
    psps.extend(get_psps_for_country(&Country::Mexico));
    psps.extend(get_psps_for_country(&Country::Colombia));
    psps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_each_country_has_three_psps() {
        assert_eq!(get_psps_for_country(&Country::Brazil).len(), 3);
        assert_eq!(get_psps_for_country(&Country::Mexico).len(), 3);
        assert_eq!(get_psps_for_country(&Country::Colombia).len(), 3);
    }

    #[test]
    fn test_all_psps_returns_nine() {
        assert_eq!(get_all_psps().len(), 9);
    }

    #[test]
    fn test_success_rates_are_valid() {
        for psp in get_all_psps() {
            assert!(psp.base_success_rate > 0.0 && psp.base_success_rate < 1.0,
                "PSP {} has invalid success rate: {}", psp.name, psp.base_success_rate);
        }
    }

    #[test]
    fn test_decline_distributions_sum_to_one() {
        for psp in get_all_psps() {
            let dist = get_decline_distribution(&psp.id);
            let total: f64 = dist.iter().map(|d| d.weight).sum();
            assert!((total - 1.0).abs() < 0.01,
                "PSP {} decline weights sum to {}, expected 1.0", psp.id, total);
        }
    }
}
