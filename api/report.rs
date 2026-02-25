use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

/// POST /api/report — Generate a performance report comparing routing scenarios.
///
/// # Stub Implementation
/// Returns 501 Not Implemented. Will be replaced by Instance 3 (feature/api-reports branch).
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
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

    let response = json!({
        "error": "Not implemented",
        "message": "Report endpoint is under development"
    });

    Ok(Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .header("Content-Type", "application/json")
        .body(Body::Text(response.to_string()))?)
}
