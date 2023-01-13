use api::initialize_lambda;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::put, Extension, Json};
use data_access::drivers::update_driver;
use lambda_http::Error;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/drivers/:id", put(put_driver)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn put_driver(
    Path(driver_id): Path<i32>,
    Json(driver): Json<endurance_racing_planner_common::Driver>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let driver = data_access::entities::driver::Driver::create(driver, Uuid::nil());
    let result = update_driver(&pool, driver_id, driver).await;
    match result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
