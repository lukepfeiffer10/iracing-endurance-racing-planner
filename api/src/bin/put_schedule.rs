use api::initialize_lambda;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::put, Extension, Json};
use data_access::entities::schedule::Stint;
use data_access::schedules::update_schedule;
use endurance_racing_planner_common::schedule::ScheduleStintDto;
use lambda_http::Error;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/plans/:id/schedule", put(put_schedule)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn put_schedule(
    Path(_plan_id): Path<Uuid>,
    Json(schedule): Json<Vec<ScheduleStintDto>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let schedule: Vec<Stint> = schedule.iter().map(|stint| stint.into()).collect();

    let result = update_schedule(&pool, schedule).await;
    match result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
