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
    } else if plan.overall_fuel_stint_config.is_some() {
        PatchPlanType::FuelStintConfig(plan.overall_fuel_stint_config.unwrap())
    } else if plan.fuel_stint_average_times.is_some() {
        let fuel_stint_average_times = plan.fuel_stint_average_times.unwrap();
        let stint_type = if fuel_stint_average_times.standard_fuel_stint.is_some() {
            data_access::entities::plan::StintType::Standard
        } else {
            data_access::entities::plan::StintType::FuelSaving
        };
        PatchPlanType::FuelStintAverageTime(
            fuel_stint_average_times
                .standard_fuel_stint
                .or(fuel_stint_average_times.fuel_saving_stint)
                .unwrap(),
            stint_type,
        )
    } else if plan.title.is_some() {
        PatchPlanType::Title(plan.title.unwrap())
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json("failed to supply any values to patch".to_string()),
        );
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
