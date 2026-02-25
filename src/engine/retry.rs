/// Decline classification for the routing engine's retry logic.
///
/// Classifies PSP decline reasons into three categories:
/// - **Hard declines**: Permanent failures — retrying will not help.
/// - **Soft declines**: Temporary failures — a different PSP may succeed.
/// - **PSP unavailable**: The PSP itself is down — cascade immediately.
use crate::models::psp::DeclineReason;

/// Returns true if the decline is permanent and should NOT be retried.
///
/// Hard declines indicate a fundamental issue with the payment instrument
/// that no PSP can resolve: insufficient funds, expired card, invalid card,
/// or a stolen card flagged by the network.
pub fn is_hard_decline(reason: &DeclineReason) -> bool {
    matches!(
        reason,
        DeclineReason::InsufficientFunds
            | DeclineReason::CardExpired
            | DeclineReason::InvalidCard
            | DeclineReason::StolenCard
    )
}

/// Returns true if the decline is temporary and should be retried with the next PSP.
///
/// Soft declines are often PSP-specific or transient — the issuing bank may be
/// temporarily unreachable, the PSP's fraud model may be overly aggressive, or
/// the processor may have a momentary issue. Trying a different PSP frequently
/// resolves these.
pub fn is_soft_decline(reason: &DeclineReason) -> bool {
    matches!(
        reason,
        DeclineReason::IssuerUnavailable
            | DeclineReason::SuspectedFraud
            | DeclineReason::DoNotHonor
            | DeclineReason::ProcessorDeclined
    )
}

/// Returns true if the PSP itself is unavailable (timeout, downtime, etc.).
///
/// When a PSP is unavailable, the attempt should not count as a decline.
/// The engine cascades immediately to the next PSP in the priority list.
pub fn is_psp_unavailable(reason: &DeclineReason) -> bool {
    matches!(reason, DeclineReason::PspUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_declines_are_classified_correctly() {
        let hard_reasons = vec![
            DeclineReason::InsufficientFunds,
            DeclineReason::CardExpired,
            DeclineReason::InvalidCard,
            DeclineReason::StolenCard,
        ];

        for reason in &hard_reasons {
            assert!(is_hard_decline(reason), "{reason} should be a hard decline");
            assert!(
                !is_soft_decline(reason),
                "{reason} should NOT be a soft decline"
            );
            assert!(
                !is_psp_unavailable(reason),
                "{reason} should NOT be psp_unavailable"
            );
        }
    }

    #[test]
    fn test_soft_declines_are_classified_correctly() {
        let soft_reasons = vec![
            DeclineReason::IssuerUnavailable,
            DeclineReason::SuspectedFraud,
            DeclineReason::DoNotHonor,
            DeclineReason::ProcessorDeclined,
        ];

        for reason in &soft_reasons {
            assert!(is_soft_decline(reason), "{reason} should be a soft decline");
            assert!(
                !is_hard_decline(reason),
                "{reason} should NOT be a hard decline"
            );
            assert!(
                !is_psp_unavailable(reason),
                "{reason} should NOT be psp_unavailable"
            );
        }
    }

    #[test]
    fn test_psp_unavailable_is_classified_correctly() {
        let reason = DeclineReason::PspUnavailable;
        assert!(is_psp_unavailable(&reason));
        assert!(!is_hard_decline(&reason));
        assert!(!is_soft_decline(&reason));
    }

    #[test]
    fn test_all_reasons_are_classified_into_exactly_one_category() {
        let all_reasons = vec![
            DeclineReason::InsufficientFunds,
            DeclineReason::CardExpired,
            DeclineReason::InvalidCard,
            DeclineReason::StolenCard,
            DeclineReason::IssuerUnavailable,
            DeclineReason::SuspectedFraud,
            DeclineReason::DoNotHonor,
            DeclineReason::ProcessorDeclined,
            DeclineReason::PspUnavailable,
        ];

        for reason in &all_reasons {
            let categories = [
                is_hard_decline(reason),
                is_soft_decline(reason),
                is_psp_unavailable(reason),
            ];
            let count = categories.iter().filter(|&&v| v).count();
            assert_eq!(
                count, 1,
                "{reason} must belong to exactly one category, found {count}"
            );
        }
    }
}
