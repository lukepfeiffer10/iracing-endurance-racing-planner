use api::{initialize_lambda, AuthenticatedUser};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json};
use lambda_http::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/users/me", get(me)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn me(user: AuthenticatedUser) -> impl IntoResponse {
    (StatusCode::OK, Json(user.0))
}
