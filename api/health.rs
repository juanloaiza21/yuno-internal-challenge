use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::json;
use yuno_internal_challenge::version;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let payload = json!({
        "status": "ok",
        "version": version(),
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::Text(payload.to_string()))?)
}
