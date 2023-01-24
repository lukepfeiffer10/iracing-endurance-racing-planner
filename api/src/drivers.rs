use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use endurance_racing_planner_common::Driver;
use sqlx::{types::Uuid, PgPool};

use crate::data_access::{
    self,
    drivers::{create_driver, get_drivers_by_plan_id, update_driver},
};

pub(crate) async fn add_driver(
    State(pool): State<PgPool>,
    Path(plan_id): Path<Uuid>,
    Json(driver): Json<Driver>,
) -> impl IntoResponse {
    let driver = data_access::entities::driver::Driver::create(driver, plan_id);
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

pub(crate) async fn get_plan_drivers(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    get_drivers_by_plan_id(&pool, id)
        .await
        .map(|drivers| {
            Json(
                drivers
                    .iter()
                    .map(|d| -> Driver { d.into() })
                    .collect::<Vec<_>>(),
            )
            .into_response()
        })
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get the plan drivers",
            )
                .into_response()
        })
}

pub(crate) async fn put_driver(
    Path(driver_id): Path<i32>,
    State(pool): State<PgPool>,
    Json(driver): Json<endurance_racing_planner_common::Driver>,
) -> impl IntoResponse {
    let driver = data_access::entities::driver::Driver::create(driver, Uuid::nil());
    let result = update_driver(&pool, driver_id, driver).await;
    match result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
