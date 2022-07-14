use api::{initialize_lambda, AuthenticatedUser};
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::patch, Extension, Json,
};
use data_access::entities::plan::{PatchPlan, PatchPlanType};
use endurance_racing_planner_common::PatchRacePlannerDto;
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = initialize_lambda("/plans/:id", patch(patch_plan)).await?;

    lambda_http::run(app).await?;
    Ok(())
}

async fn patch_plan(
    Path(id): Path<Uuid>,
    Json(plan): Json<PatchRacePlannerDto>,
    AuthenticatedUser(user): AuthenticatedUser,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let patch = if plan.overall_event_config.is_some() {
        PatchPlanType::EventConfig(plan.overall_event_config.unwrap())
    } else {
        PatchPlanType::Title(plan.title.unwrap())
    };

    let result = data_access::plans::patch_plan(&pool, PatchPlan::new(id, user.id, patch)).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(id.to_string())),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("something went wrong saving the plan".to_string()),
        ),
    }
}
