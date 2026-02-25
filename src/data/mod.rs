/// Test data generation for the routing engine.
///
/// Generates realistic transaction data for FashionForward's
/// Brazil, Mexico, and Colombia operations.

use crate::models::transaction::{Country, Currency, Transaction};

/// Generate a batch of test transactions.
///
/// # Stub Implementation
/// Currently generates minimal dummy data. Will be replaced
/// by Instance 1 (feature/psp-simulator branch).
pub fn generate_test_data(count: usize) -> Vec<Transaction> {
    (0..count)
        .map(|i| Transaction {
            id: format!("txn_{:04}", i + 1),
            amount: 100.0,
            currency: Currency::BRL,
            country: Country::Brazil,
            card_bin: "411111".to_string(),
            card_last4: format!("{:04}", i % 10000),
            customer_id: format!("cust_{:03}", (i % 10) + 1),
            timestamp: "2025-01-15T10:00:00Z".to_string(),
        })
        .collect()
}

/// Get the standard test dataset of 200+ transactions.
pub fn get_test_dataset() -> Vec<Transaction> {
    generate_test_data(200)
}
