//! Test data generation for the routing engine.
//!
//! Generates realistic transaction data for FashionForward's
//! Brazil, Mexico, and Colombia operations. Uses seeded RNG
//! for reproducible datasets across runs.
//!
//! # Data Distribution
//! - 210 transactions (~70 per country)
//! - 15 unique customers (some with many transactions)
//! - Amount range: $10–$500 USD equivalent
//! - Realistic fake BINs per country
//! - Timestamps spread across a business day

use crate::models::transaction::{Country, Currency, Transaction};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// BINs per country — realistic but fake card prefixes.
const BRAZIL_BINS: [&str; 3] = ["411111", "510510", "376411"];
const MEXICO_BINS: [&str; 3] = ["424242", "551234", "371449"];
const COLOMBIA_BINS: [&str; 3] = ["431940", "520082", "378282"];

/// Data seed for reproducible generation.
const DATA_SEED: u64 = 42;

/// Generate a batch of test transactions with realistic distribution.
///
/// Transactions are split roughly equally across Brazil, Mexico, and Colombia,
/// with varied amounts, multiple customers, and timestamps across a business day.
pub fn generate_test_data(count: usize) -> Vec<Transaction> {
    let mut rng = StdRng::seed_from_u64(DATA_SEED);
    let mut transactions = Vec::with_capacity(count);

    // Country distribution: ~equal thirds
    let countries = [
        (Country::Brazil, Currency::BRL, &BRAZIL_BINS),
        (Country::Mexico, Currency::MXN, &MEXICO_BINS),
        (Country::Colombia, Currency::COP, &COLOMBIA_BINS),
    ];

    for i in 0..count {
        let country_idx = i % 3;
        let (country, currency, bins) = &countries[country_idx];

        // Pick a BIN (rotate through available BINs)
        let bin = bins[rng.gen_range(0..bins.len())];

        // Generate last 4 digits
        let last4 = format!("{:04}", rng.gen_range(0..10000));

        // Customer ID: 15 unique customers, some heavy users
        let customer_id = format!("cust_{:03}", select_customer(&mut rng));

        // Amount: weighted distribution
        // 40% small ($10-100), 35% medium ($100-300), 25% large ($300-500)
        let amount = generate_amount(&mut rng);

        // Timestamp: spread across 2025-01-15 08:00-20:00 UTC
        let hour = 8 + (i * 12 / count);
        let minute = rng.gen_range(0..60);
        let second = rng.gen_range(0..60);
        let timestamp = format!("2025-01-15T{:02}:{:02}:{:02}Z", hour, minute, second);

        transactions.push(Transaction {
            id: format!("txn_{:04}", i + 1),
            amount: round_to_cents(amount),
            currency: currency.clone(),
            country: country.clone(),
            card_bin: bin.to_string(),
            card_last4: last4,
            customer_id,
            timestamp,
        });
    }

    transactions
}

/// Get the standard test dataset of 210 transactions.
///
/// This is the canonical dataset used for performance reports
/// and demos. Always returns the same data (seeded RNG).
pub fn get_test_dataset() -> Vec<Transaction> {
    generate_test_data(210)
}

/// Select a customer ID with realistic distribution.
///
/// Some customers are "heavy users" with many transactions,
/// while others are one-time buyers.
fn select_customer(rng: &mut StdRng) -> u32 {
    let roll: f64 = rng.gen();
    if roll < 0.30 {
        // 30% of transactions come from top 3 customers
        rng.gen_range(1..=3)
    } else if roll < 0.60 {
        // 30% from mid-tier customers (4-8)
        rng.gen_range(4..=8)
    } else {
        // 40% from long-tail customers (9-15)
        rng.gen_range(9..=15)
    }
}

/// Generate a transaction amount with weighted distribution.
///
/// - 40% small: $10–$100
/// - 35% medium: $100–$300
/// - 25% large: $300–$500
fn generate_amount(rng: &mut StdRng) -> f64 {
    let roll: f64 = rng.gen();
    if roll < 0.40 {
        rng.gen_range(10.0..100.0)
    } else if roll < 0.75 {
        rng.gen_range(100.0..300.0)
    } else {
        rng.gen_range(300.0..500.0)
    }
}

/// Round a float to 2 decimal places (cents).
fn round_to_cents(amount: f64) -> f64 {
    (amount * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_correct_count() {
        let data = generate_test_data(200);
        assert_eq!(data.len(), 200);
    }

    #[test]
    fn test_default_dataset_is_210() {
        let data = get_test_dataset();
        assert_eq!(data.len(), 210);
    }

    #[test]
    fn test_is_deterministic() {
        let d1 = generate_test_data(50);
        let d2 = generate_test_data(50);
        for (a, b) in d1.iter().zip(d2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.amount, b.amount);
            assert_eq!(a.card_bin, b.card_bin);
            assert_eq!(a.card_last4, b.card_last4);
        }
    }

    #[test]
    fn test_country_distribution() {
        let data = get_test_dataset();
        let brazil = data.iter().filter(|t| t.country == Country::Brazil).count();
        let mexico = data.iter().filter(|t| t.country == Country::Mexico).count();
        let colombia = data.iter().filter(|t| t.country == Country::Colombia).count();

        // Each should be ~70 (210/3)
        assert_eq!(brazil, 70);
        assert_eq!(mexico, 70);
        assert_eq!(colombia, 70);
    }

    #[test]
    fn test_amount_range() {
        let data = get_test_dataset();
        for tx in &data {
            assert!(tx.amount >= 10.0 && tx.amount <= 500.0,
                "Amount {} out of range for txn {}", tx.amount, tx.id);
        }
    }

    #[test]
    fn test_unique_customer_count() {
        let data = get_test_dataset();
        let mut customers: Vec<&str> = data.iter().map(|t| t.customer_id.as_str()).collect();
        customers.sort();
        customers.dedup();
        assert!(customers.len() >= 10,
            "Expected at least 10 unique customers, got {}", customers.len());
    }

    #[test]
    fn test_valid_bins_per_country() {
        let data = get_test_dataset();
        for tx in &data {
            let valid_bins = match tx.country {
                Country::Brazil => &BRAZIL_BINS[..],
                Country::Mexico => &MEXICO_BINS[..],
                Country::Colombia => &COLOMBIA_BINS[..],
            };
            assert!(valid_bins.contains(&tx.card_bin.as_str()),
                "Invalid BIN {} for country {:?}", tx.card_bin, tx.country);
        }
    }

    #[test]
    fn test_unique_transaction_ids() {
        let data = get_test_dataset();
        let mut ids: Vec<&str> = data.iter().map(|t| t.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Transaction IDs must be unique");
    }
}
