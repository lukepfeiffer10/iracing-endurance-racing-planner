use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use data_access::drivers::create_driver;
use endurance_racing_planner_common::Driver;
use sqlx::{types::Uuid, PgPool};

#[axum_macros::debug_handler]
pub(crate) async fn add_driver(
    Extension(pool): Extension<PgPool>,
    Path(plan_id): Path<Uuid>,
    Json(driver): Json<Driver>,
) -> impl IntoResponse {
    let driver = data_access::entities::driver::Driver::create(driver, plan_id);
    tracing::info!("{:?}", &driver);
    let new_driver_result = create_driver(&pool, driver).await;
    match new_driver_result {
        Ok(new_driver) => (StatusCode::CREATED, Json::<Driver>(new_driver.into())).into_response(),
        Err(e) => {
            tracing::error!("{:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "there was a problem creating the driver",
            )
                .into_response()
        }
    }
}
