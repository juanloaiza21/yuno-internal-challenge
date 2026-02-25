use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete performance report comparing no-retry vs smart-retry routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Total number of transactions processed.
    pub total_transactions: usize,
    /// Results without retry (current FashionForward behavior).
    pub no_retry: ScenarioResult,
    /// Results with smart retry routing.
    pub smart_retry: ScenarioResult,
    /// Improvement metrics from smart retry.
    pub improvement: ImprovementMetrics,
    /// Authorization rate breakdown by country.
    pub by_country: HashMap<String, CountryMetrics>,
    /// Performance breakdown by PSP.
    pub by_psp: HashMap<String, PspMetrics>,
}

/// Results for a single routing scenario (no-retry or smart-retry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Number of approved transactions.
    pub approved: usize,
    /// Number of declined transactions.
    pub declined: usize,
    /// Authorization rate as a percentage (0.0–100.0).
    pub authorization_rate: f64,
    /// Average number of PSP attempts per transaction.
    pub avg_attempts: f64,
    /// Average latency per transaction in milliseconds.
    pub avg_latency_ms: f64,
}

/// Metrics showing the business impact of smart retry over no-retry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    /// Authorization rate improvement in percentage points.
    pub rate_lift_percentage: f64,
    /// Additional transactions approved by smart retry.
    pub additional_approvals: usize,
    /// Estimated additional revenue in USD.
    pub estimated_revenue_recovered_usd: f64,
}

/// Authorization rate metrics for a specific country.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryMetrics {
    /// Auth rate without retry.
    pub no_retry_rate: f64,
    /// Auth rate with smart retry.
    pub smart_retry_rate: f64,
    /// Improvement in percentage points.
    pub improvement: f64,
    /// Total transactions for this country.
    pub total_transactions: usize,
}

/// Performance metrics for a specific PSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PspMetrics {
    /// Total times this PSP was attempted.
    pub total_attempts: usize,
    /// Number of approvals.
    pub approvals: usize,
    /// Number of declines.
    pub declines: usize,
    /// Approval rate as a percentage.
    pub approval_rate: f64,
    /// Average response latency in milliseconds.
    pub avg_latency_ms: f64,
}

/// API request body for the /api/report endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    /// Number of transactions to process (default: 200).
    pub transaction_count: Option<usize>,
    /// Routing strategy to use for smart retry scenario.
    pub routing_strategy: Option<super::routing::RoutingStrategy>,
}
