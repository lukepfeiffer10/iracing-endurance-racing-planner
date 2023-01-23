use api::initialize_lambda;
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json,
};
use data_access::drivers::get_drivers_by_plan_id;
use endurance_racing_planner_common::Driver;
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans/:id/drivers", get(get_plan_drivers)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn get_plan_drivers(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
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
