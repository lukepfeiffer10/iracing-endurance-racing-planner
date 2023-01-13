use api::initialize_lambda;
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::post, Extension, Json,
};
use data_access::drivers::create_driver;
use endurance_racing_planner_common::Driver;
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans/:plan_id/drivers", post(add_driver)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn add_driver(
    Path(plan_id): Path<Uuid>,
    Json(driver): Json<Driver>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let driver = data_access::entities::driver::Driver::create(driver, plan_id);
    let new_driver_result = create_driver(&pool, driver).await;
    match new_driver_result {
        Ok(new_driver) => (StatusCode::CREATED, Json::<Driver>(new_driver.into())).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "there was a problem creating the user",
        )
            .into_response(),
    }
}
