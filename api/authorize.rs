/// POST /api/authorize — Route a payment transaction through PSPs.
///
/// Accepts a JSON `AuthorizationRequest`, validates the input, builds a
/// `Transaction`, runs it through the `RoutingEngine`, and returns the
/// serialized `RoutingResult`.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

use yuno_internal_challenge::engine::RoutingEngine;
use yuno_internal_challenge::models::routing::AuthorizationRequest;
use yuno_internal_challenge::models::transaction::{Country, Currency, Transaction};
use yuno_internal_challenge::simulator::PspSimulator;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

/// Vercel handler for the `/api/authorize` endpoint.
///
/// Only accepts `POST` requests. All other methods receive a 405
/// Method Not Allowed response.
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // ------------------------------------------------------------------
    // 1. Method guard — only POST is accepted
    // ------------------------------------------------------------------
    if *req.method() != http::Method::POST {
        return json_response(
            StatusCode::METHOD_NOT_ALLOWED,
            &json!({
                "error": "Method not allowed",
                "details": "Use POST to submit a transaction for authorization"
            }),
        );
    }

    // ------------------------------------------------------------------
    // 2. Parse the request body into an AuthorizationRequest
    // ------------------------------------------------------------------
    let body_bytes = req.body().as_ref();
    let auth_request: AuthorizationRequest = match serde_json::from_slice(body_bytes) {
        Ok(parsed) => parsed,
        Err(e) => {
            return json_response(
                StatusCode::BAD_REQUEST,
                &json!({
                    "error": "Invalid request body",
                    "details": format!("Failed to parse JSON: {e}")
                }),
            );
        }
    };

    // ------------------------------------------------------------------
    // 3. Validate the request fields
    // ------------------------------------------------------------------
    if let Err(msg) = validate_request(&auth_request) {
        return json_response(
            StatusCode::BAD_REQUEST,
            &json!({
                "error": "Validation failed",
                "details": msg
            }),
        );
    }

    // ------------------------------------------------------------------
    // 4. Map string fields to domain enums
    // ------------------------------------------------------------------
    let currency = match parse_currency(&auth_request.currency) {
        Some(c) => c,
        None => {
            return json_response(
                StatusCode::BAD_REQUEST,
                &json!({
                    "error": "Validation failed",
                    "details": format!("Unsupported currency: {}", auth_request.currency)
                }),
            );
        }
    };

    let country = match parse_country(&auth_request.country) {
        Some(c) => c,
        None => {
            return json_response(
                StatusCode::BAD_REQUEST,
                &json!({
                    "error": "Validation failed",
                    "details": format!("Unsupported country: {}", auth_request.country)
                }),
            );
        }
    };

    // ------------------------------------------------------------------
    // 5. Build the Transaction
    // ------------------------------------------------------------------
    let transaction_id = generate_transaction_id(&auth_request);
    let timestamp = current_timestamp();

    let transaction = Transaction {
        id: transaction_id,
        amount: auth_request.amount,
        currency,
        country,
        card_bin: auth_request.card_bin.clone(),
        card_last4: auth_request.card_last4.clone(),
        customer_id: auth_request.customer_id.clone(),
        timestamp,
    };

    // ------------------------------------------------------------------
    // 6. Route the transaction
    // ------------------------------------------------------------------
    let strategy = auth_request.routing_strategy.unwrap_or_default();

    let simulator = PspSimulator::new();
    let engine = RoutingEngine::new(simulator);
    let result = engine.route(&transaction, &strategy);

    // ------------------------------------------------------------------
    // 7. Return the routing result
    // ------------------------------------------------------------------
    json_response(StatusCode::OK, &result)
}

// ======================================================================
// Helper functions
// ======================================================================

/// Build a JSON `Response` with the given status code and serializable body.
fn json_response<T: serde::Serialize>(
    status: StatusCode,
    body: &T,
) -> Result<Response<Body>, Error> {
    let json_string = serde_json::to_string(body)?;
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::Text(json_string))?)
}

/// Validate all fields of an `AuthorizationRequest`.
///
/// Returns `Ok(())` when valid, or `Err(message)` describing the first
/// validation failure encountered.
fn validate_request(req: &AuthorizationRequest) -> Result<(), String> {
    if req.amount <= 0.0 {
        return Err("amount must be greater than 0".into());
    }

    if parse_currency(&req.currency).is_none() {
        return Err(format!(
            "Invalid currency '{}'. Supported: BRL, MXN, COP",
            req.currency
        ));
    }

    if parse_country(&req.country).is_none() {
        return Err(format!(
            "Invalid country '{}'. Supported: Brazil, Mexico, Colombia",
            req.country
        ));
    }

    if req.card_bin.is_empty() {
        return Err("card_bin must not be empty".into());
    }

    if req.card_last4.is_empty() {
        return Err("card_last4 must not be empty".into());
    }

    if req.customer_id.is_empty() {
        return Err("customer_id must not be empty".into());
    }

    Ok(())
}

/// Map a currency code string to the `Currency` enum.
fn parse_currency(s: &str) -> Option<Currency> {
    match s {
        "BRL" => Some(Currency::BRL),
        "MXN" => Some(Currency::MXN),
        "COP" => Some(Currency::COP),
        _ => None,
    }
}

/// Map a country name string to the `Country` enum.
fn parse_country(s: &str) -> Option<Country> {
    match s {
        "Brazil" => Some(Country::Brazil),
        "Mexico" => Some(Country::Mexico),
        "Colombia" => Some(Country::Colombia),
        _ => None,
    }
}

/// Generate a deterministic transaction ID from the request fields.
///
/// Uses `DefaultHasher` to produce a 16-hex-digit hash, prefixed with `txn_`.
fn generate_transaction_id(req: &AuthorizationRequest) -> String {
    let mut hasher = DefaultHasher::new();
    req.card_bin.hash(&mut hasher);
    req.card_last4.hash(&mut hasher);
    req.customer_id.hash(&mut hasher);
    req.amount.to_bits().hash(&mut hasher);
    let hash = hasher.finish();
    format!("txn_{:016x}", hash)
}

/// Return the current UTC time as an ISO 8601 string.
///
/// Uses a manual calculation from `SystemTime` to avoid pulling in the
/// `chrono` crate. Falls back to a fixed epoch string if the system clock
/// is unavailable.
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Convert epoch seconds to a basic ISO 8601 UTC timestamp.
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Gregorian calendar conversion from days since epoch (1970-01-01).
    let (year, month, day) = epoch_days_to_date(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since the Unix epoch (1970-01-01) to (year, month, day).
fn epoch_days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm adapted from Howard Hinnant's `civil_from_days`.
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
