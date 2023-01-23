use api::initialize_lambda;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{routing::post, Extension, Json};
use data_access::entities::schedule::Stint;
use data_access::schedules::create_schedule;
use endurance_racing_planner_common::schedule::ScheduleStintDto;
use lambda_http::Error;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/plans/:id/schedule", post(add_schedule)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn add_schedule(
    Path(plan_id): Path<Uuid>,
    Json(schedule): Json<Vec<ScheduleStintDto>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let new_schedule: Vec<Stint> = schedule.iter().map(|stint| stint.into()).collect();

    let new_schedule_result = create_schedule(&pool, plan_id, new_schedule).await;
    match new_schedule_result {
        Ok(success) => {
            if success {
                (
                    StatusCode::CREATED,
                    [(
                        header::CONTENT_LOCATION,
                        format!("/plans/{}/schedule", &plan_id),
                    )],
                )
                    .into_response()
            } else {
                (StatusCode::BAD_REQUEST, "failed to save the schedule").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
