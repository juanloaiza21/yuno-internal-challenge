use serde::{Deserialize, Serialize};

/// Reasons why a PSP may decline a transaction.
///
/// Decline reasons are classified as either "hard" (permanent, do not retry)
/// or "soft" (temporary, retry with a different PSP may succeed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeclineReason {
    // --- Hard declines: do NOT retry ---
    /// Customer has insufficient funds.
    InsufficientFunds,
    /// Card has expired.
    CardExpired,
    /// Card number is invalid.
    InvalidCard,
    /// Card has been reported stolen.
    StolenCard,

    // --- Soft declines: RETRY with next PSP ---
    /// Issuing bank is temporarily unavailable.
    IssuerUnavailable,
    /// Transaction flagged as potential fraud.
    SuspectedFraud,
    /// Generic decline — issuer says "do not honor".
    DoNotHonor,
    /// PSP processor declined the transaction.
    ProcessorDeclined,
    /// PSP is temporarily unavailable (for cascading).
    PspUnavailable,
}

impl DeclineReason {
    /// Returns true if this is a hard decline (permanent — do not retry).
    pub fn is_hard_decline(&self) -> bool {
        matches!(
            self,
            DeclineReason::InsufficientFunds
                | DeclineReason::CardExpired
                | DeclineReason::InvalidCard
                | DeclineReason::StolenCard
        )
    }

    /// Returns true if this is a soft decline (temporary — retry may succeed).
    pub fn is_soft_decline(&self) -> bool {
        !self.is_hard_decline()
    }

    /// Returns true if the PSP itself is unavailable (cascade immediately).
    pub fn is_psp_unavailable(&self) -> bool {
        matches!(self, DeclineReason::PspUnavailable)
    }
}

impl std::fmt::Display for DeclineReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclineReason::InsufficientFunds => write!(f, "insufficient_funds"),
            DeclineReason::CardExpired => write!(f, "card_expired"),
            DeclineReason::InvalidCard => write!(f, "invalid_card"),
            DeclineReason::StolenCard => write!(f, "stolen_card"),
            DeclineReason::IssuerUnavailable => write!(f, "issuer_unavailable"),
            DeclineReason::SuspectedFraud => write!(f, "suspected_fraud"),
            DeclineReason::DoNotHonor => write!(f, "do_not_honor"),
            DeclineReason::ProcessorDeclined => write!(f, "processor_declined"),
            DeclineReason::PspUnavailable => write!(f, "psp_unavailable"),
        }
    }
}

/// Configuration for a Payment Service Provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PspConfig {
    /// Unique PSP identifier (e.g., "psp_br_1").
    pub id: String,
    /// Human-readable PSP name (e.g., "PagSeguro").
    pub name: String,
    /// Country this PSP serves.
    pub country: super::transaction::Country,
    /// Base approval rate (0.0–1.0).
    pub base_success_rate: f64,
    /// Minimum response latency in milliseconds.
    pub latency_min_ms: u64,
    /// Maximum response latency in milliseconds.
    pub latency_max_ms: u64,
    /// Processing fee as a percentage (e.g., 2.9 for 2.9%).
    pub fee_percentage: f64,
    /// Fixed processing fee in USD cents.
    pub fee_fixed_cents: u64,
}

/// Response from a PSP after attempting to process a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PspResponse {
    /// PSP that processed this attempt.
    pub psp_id: String,
    /// PSP name.
    pub psp_name: String,
    /// Whether the transaction was approved.
    pub approved: bool,
    /// Decline reason (None if approved).
    pub decline_reason: Option<DeclineReason>,
    /// Response latency in milliseconds.
    pub latency_ms: u64,
}
