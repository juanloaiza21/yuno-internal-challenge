use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use yuno_internal_challenge::data;
use yuno_internal_challenge::engine::RoutingEngine;
use yuno_internal_challenge::models::report::ReportRequest;
use yuno_internal_challenge::models::routing::RoutingStrategy;
use yuno_internal_challenge::report;
use yuno_internal_challenge::simulator::PspSimulator;

/// Default number of transactions when none is specified.
const DEFAULT_TRANSACTION_COUNT: usize = 200;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

/// POST /api/report — Generate a performance report comparing routing scenarios.
///
/// Accepts an optional JSON body with `transaction_count` and `routing_strategy`
/// fields. When the body is empty or fields are omitted, defaults to 200
/// transactions with `OptimizeForApprovals` strategy.
///
/// # Request Body (optional)
///
/// ```json
/// {
///   "transaction_count": 500,
///   "routing_strategy": "Balanced"
/// }
/// ```
///
/// # Responses
///
/// - **200** — JSON `PerformanceReport` with no-retry vs smart-retry comparison.
/// - **400** — Malformed JSON in request body.
/// - **405** — Non-POST method used.
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Reject non-POST methods.
    if *req.method() != http::Method::POST {
        let error = json!({
            "error": "Method not allowed",
            "message": "Use POST to generate a performance report"
        });
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Content-Type", "application/json")
            .body(Body::Text(error.to_string()))?);
    }

    // Extract body bytes, handling all Body variants.
    let body = req.into_body();
    let bytes = match &body {
        Body::Empty => vec![],
        Body::Text(t) => t.as_bytes().to_vec(),
        Body::Binary(b) => b.to_vec(),
    };

    // Parse request parameters or use defaults for empty body.
    let (count, strategy) = if bytes.is_empty() {
        (
            DEFAULT_TRANSACTION_COUNT,
            RoutingStrategy::OptimizeForApprovals,
        )
    } else {
        match serde_json::from_slice::<ReportRequest>(&bytes) {
            Ok(req) => (
                req.transaction_count.unwrap_or(DEFAULT_TRANSACTION_COUNT),
                req.routing_strategy
                    .unwrap_or(RoutingStrategy::OptimizeForApprovals),
            ),
            Err(e) => {
                let error = json!({
                    "error": "Bad request",
                    "message": format!("Invalid JSON body: {e}")
                });
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(Body::Text(error.to_string()))?);
            }
        }
    };

    // Generate test transactions.
    let transactions = data::generate_test_data(count);

    // Build the routing engine with a fresh PSP simulator.
    let simulator = PspSimulator::new();
    let engine = RoutingEngine::new(simulator);

    // Run the report comparing no-retry vs smart-retry scenarios.
    let performance_report = report::generate_report(&transactions, &engine, &strategy);

    // Serialize the report to JSON.
    let body = serde_json::to_string(&performance_report)
        .map_err(|e| Error::from(format!("Failed to serialize report: {e}")))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::Text(body))?)
}
