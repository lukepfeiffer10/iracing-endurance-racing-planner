use api::{initialize_lambda, AuthenticatedUser};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json};
use data_access::plans::get_plans_by_user_id;
use endurance_racing_planner_common::PlanListDto;
use lambda_http::Error;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans", get(get_plans)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn get_plans(
    AuthenticatedUser(user): AuthenticatedUser,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let plans = get_plans_by_user_id(&pool, user.id).await.map(|plans| {
        plans
            .iter()
            .map(|p| PlanListDto {
                id: p.id,
                title: p.title.clone(),
                owner: p.owner.clone(),
                last_modified: p.modified_date.or(Some(p.created_date)).unwrap(),
            })
            .collect::<Vec<PlanListDto>>()
    });

    match plans {
        Ok(plans) => Json(plans).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load the plans",
        )
            .into_response(),
    }
}
