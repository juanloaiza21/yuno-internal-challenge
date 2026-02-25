/// PSP Simulator — simulates realistic payment processor behavior.
///
/// Each PSP has different success rates, decline reason distributions,
/// and response latencies. The simulator uses deterministic seeding
/// for reproducible results.

pub mod config;

use crate::models::psp::{PspConfig, PspResponse};
use crate::models::transaction::Transaction;

/// Simulates PSP behavior for transaction processing.
pub struct PspSimulator;

impl PspSimulator {
    /// Creates a new PSP simulator instance.
    pub fn new() -> Self {
        PspSimulator
    }

    /// Simulate a PSP processing a transaction.
    ///
    /// Returns a PspResponse with approval/decline decision,
    /// decline reason (if applicable), and simulated latency.
    ///
    /// # Stub Implementation
    /// Currently returns a dummy approved response. Will be replaced
    /// by Instance 1 (feature/psp-simulator branch).
    pub fn process(&self, transaction: &Transaction, psp: &PspConfig) -> PspResponse {
        let _ = transaction;
        PspResponse {
            psp_id: psp.id.clone(),
            psp_name: psp.name.clone(),
            approved: true,
            decline_reason: None,
            latency_ms: 100,
        }
    }
}

impl Default for PspSimulator {
    fn default() -> Self {
        Self::new()
    }
}
