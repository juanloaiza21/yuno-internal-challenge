/// PSP configurations for each country.
///
/// FashionForward operates in Brazil, Mexico, and Colombia,
/// with 3 PSPs configured per country (9 total).

use crate::models::psp::PspConfig;
use crate::models::transaction::Country;

/// Returns the list of PSP configurations for a given country.
///
/// # Stub Implementation
/// Returns a minimal set of dummy PSPs. Will be replaced
/// by Instance 1 (feature/psp-simulator branch).
pub fn get_psps_for_country(country: &Country) -> Vec<PspConfig> {
    let (prefix, names) = match country {
        Country::Brazil => ("br", vec!["PSP_BR_1", "PSP_BR_2", "PSP_BR_3"]),
        Country::Mexico => ("mx", vec!["PSP_MX_1", "PSP_MX_2", "PSP_MX_3"]),
        Country::Colombia => ("co", vec!["PSP_CO_1", "PSP_CO_2", "PSP_CO_3"]),
    };

    names
        .into_iter()
        .enumerate()
        .map(|(i, name)| PspConfig {
            id: format!("psp_{}_{}", prefix, i + 1),
            name: name.to_string(),
            country: country.clone(),
            base_success_rate: 0.75,
            latency_min_ms: 150,
            latency_max_ms: 400,
            fee_percentage: 2.9,
            fee_fixed_cents: 30,
        })
        .collect()
}

/// Returns all PSP configurations across all countries.
pub fn get_all_psps() -> Vec<PspConfig> {
    let mut psps = Vec::new();
    psps.extend(get_psps_for_country(&Country::Brazil));
    psps.extend(get_psps_for_country(&Country::Mexico));
    psps.extend(get_psps_for_country(&Country::Colombia));
    psps
}
