use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {    
    Ok(match event.path_parameters().first("id") {
        Some(id) => json!({"id": id}).into_response(),
        _ => Response::builder()
            .status(400)
            .body("Empty id".into())
            .expect("failed to render response"),
    })
}