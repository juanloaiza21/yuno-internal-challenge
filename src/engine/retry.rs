/// Decline classification for retry logic.
///
/// Hard declines are permanent and should NOT be retried.
/// Soft declines are temporary and may succeed with a different PSP.

use crate::models::psp::DeclineReason;

/// Returns true if the decline is permanent (do not retry).
///
/// # Stub Implementation
/// Delegates to DeclineReason::is_hard_decline(). Will be enhanced
/// by Instance 2 (feature/routing-engine branch).
pub fn is_hard_decline(reason: &DeclineReason) -> bool {
    reason.is_hard_decline()
}

/// Returns true if the decline is temporary (retry with next PSP).
pub fn is_soft_decline(reason: &DeclineReason) -> bool {
    reason.is_soft_decline()
}

/// Returns true if the PSP itself is unavailable (cascade immediately).
pub fn is_psp_unavailable(reason: &DeclineReason) -> bool {
    reason.is_psp_unavailable()
}
