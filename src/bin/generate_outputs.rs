//! CLI tool to generate test data and performance report files.
//!
//! Produces:
//! - `output/test_transactions.json` — 210 test transactions
//! - `output/performance_report.json` — Full performance report (no-retry vs smart-retry)

use yuno_internal_challenge::data::get_test_dataset;
use yuno_internal_challenge::engine::RoutingEngine;
use yuno_internal_challenge::models::routing::RoutingStrategy;
use yuno_internal_challenge::report::generate_report;
use yuno_internal_challenge::simulator::PspSimulator;

fn main() {
    // Generate test transactions
    let transactions = get_test_dataset();
    let tx_json = serde_json::to_string_pretty(&transactions).expect("Failed to serialize transactions");
    std::fs::create_dir_all("output").expect("Failed to create output directory");
    std::fs::write("output/test_transactions.json", &tx_json).expect("Failed to write transactions");
    println!("Wrote output/test_transactions.json ({} transactions)", transactions.len());

    // Generate performance report
    let simulator = PspSimulator::new();
    let engine = RoutingEngine::new(simulator);
    let report = generate_report(&transactions, &engine, &RoutingStrategy::OptimizeForApprovals);
    let report_json = serde_json::to_string_pretty(&report).expect("Failed to serialize report");
    std::fs::write("output/performance_report.json", &report_json).expect("Failed to write report");

    // Print summary
    println!("Wrote output/performance_report.json");
    println!();
    println!("=== PERFORMANCE REPORT SUMMARY ===");
    println!("Total Transactions: {}", report.total_transactions);
    println!();
    println!("--- No Retry (Current FashionForward) ---");
    println!("  Approved:           {}", report.no_retry.approved);
    println!("  Declined:           {}", report.no_retry.declined);
    println!("  Authorization Rate: {:.1}%", report.no_retry.authorization_rate);
    println!("  Avg Attempts:       {:.2}", report.no_retry.avg_attempts);
    println!("  Avg Latency:        {:.1}ms", report.no_retry.avg_latency_ms);
    println!();
    println!("--- Smart Retry (Routing Engine) ---");
    println!("  Approved:           {}", report.smart_retry.approved);
    println!("  Declined:           {}", report.smart_retry.declined);
    println!("  Authorization Rate: {:.1}%", report.smart_retry.authorization_rate);
    println!("  Avg Attempts:       {:.2}", report.smart_retry.avg_attempts);
    println!("  Avg Latency:        {:.1}ms", report.smart_retry.avg_latency_ms);
    println!();
    println!("--- Improvement ---");
    println!("  Rate Lift:          +{:.1} percentage points", report.improvement.rate_lift_percentage);
    println!("  Extra Approvals:    {} transactions", report.improvement.additional_approvals);
    println!("  Revenue Recovered:  ${:.2}", report.improvement.estimated_revenue_recovered_usd);
    println!();
    println!("--- By Country ---");
    for (country, metrics) in &report.by_country {
        println!("  {}: {:.1}% -> {:.1}% (+{:.1}pp, {} txns)",
            country, metrics.no_retry_rate, metrics.smart_retry_rate,
            metrics.improvement, metrics.total_transactions);
    }
    println!();
    println!("--- By PSP ---");
    for (psp, metrics) in &report.by_psp {
        println!("  {}: {} attempts, {} approved, {:.1}% rate, {:.1}ms avg latency",
            psp, metrics.total_attempts, metrics.approvals,
            metrics.approval_rate, metrics.avg_latency_ms);
    }
}
