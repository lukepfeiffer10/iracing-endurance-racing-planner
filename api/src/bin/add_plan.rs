use api::{initialize_lambda, AuthenticatedUser};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{routing::post, Extension, Json};
use data_access::entities::Plan;
use data_access::plans::create_plan;
use endurance_racing_planner_common::RacePlannerDto;
use lambda_http::Error;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/plans", post(add_plan)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn add_plan(
    Json(plan): Json<RacePlannerDto>,
    AuthenticatedUser(user): AuthenticatedUser,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let mut new_plan: Plan = plan.into();
    new_plan.created_by = user.id;

    let new_plan_result = create_plan(&pool, new_plan).await;
    if let Ok(new_plan) = new_plan_result {
        (
            StatusCode::CREATED,
            [(header::CONTENT_LOCATION, format!("/plans/{}", new_plan.id))],
            Json::<RacePlannerDto>(new_plan.into()),
        )
            .into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save the plan").into_response()
    }
}
