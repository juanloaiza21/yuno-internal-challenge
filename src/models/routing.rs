use serde::{Deserialize, Serialize};
use super::psp::DeclineReason;

/// The result of routing a transaction through one or more PSPs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// Transaction ID that was routed.
    pub transaction_id: String,
    /// Whether the transaction was ultimately approved.
    pub approved: bool,
    /// The PSP that approved the transaction (None if all declined).
    pub final_psp: Option<String>,
    /// All PSP attempts made during routing.
    pub attempts: Vec<RoutingAttempt>,
    /// Total number of PSP attempts.
    pub total_attempts: usize,
    /// Total latency across all attempts in milliseconds.
    pub total_latency_ms: u64,
}

/// A single PSP attempt within a routing flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAttempt {
    /// PSP identifier.
    pub psp_id: String,
    /// PSP name.
    pub psp_name: String,
    /// Whether this specific attempt was approved.
    pub approved: bool,
    /// Decline reason for this attempt (None if approved).
    pub decline_reason: Option<DeclineReason>,
    /// Latency of this specific attempt in milliseconds.
    pub latency_ms: u64,
    /// 1-indexed attempt number.
    pub attempt_number: usize,
}

/// Routing strategy that determines PSP selection order.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RoutingStrategy {
    /// Select PSPs with highest approval rates first.
    #[default]
    OptimizeForApprovals,
    /// Select cheapest PSPs first.
    OptimizeForCost,
    /// Balance between approval rate and cost.
    Balanced,
}

/// API request body for the /api/authorize endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    /// Transaction amount in the local currency.
    pub amount: f64,
    /// Currency code (BRL, MXN, COP).
    pub currency: String,
    /// Country name (Brazil, Mexico, Colombia).
    pub country: String,
    /// First 6 digits of the card.
    pub card_bin: String,
    /// Last 4 digits of the card.
    pub card_last4: String,
    /// Customer identifier.
    pub customer_id: String,
    /// Optional routing strategy.
    pub routing_strategy: Option<RoutingStrategy>,
}
