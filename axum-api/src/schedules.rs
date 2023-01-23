use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use data_access::entities::schedule::Stint;
use data_access::schedules::{create_schedule, get_schedule_by_plan_id, update_schedule};
use endurance_racing_planner_common::schedule::ScheduleStintDto;
use sqlx::types::Uuid;
use sqlx::PgPool;

pub(crate) async fn add_schedule(
    Path(plan_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(schedule): Json<Vec<ScheduleStintDto>>,
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

pub(crate) async fn get_schedule(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    get_schedule_by_plan_id(&pool, id)
        .await
        .map(|schedule| Json(schedule).into_response())
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get the schedule",
            )
                .into_response()
        })
}

pub(crate) async fn put_schedule(
    Path(_plan_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(schedule): Json<Vec<ScheduleStintDto>>,
) -> impl IntoResponse {
    let schedule: Vec<Stint> = schedule.iter().map(|stint| stint.into()).collect();

    let result = update_schedule(&pool, schedule).await;
    match result {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
