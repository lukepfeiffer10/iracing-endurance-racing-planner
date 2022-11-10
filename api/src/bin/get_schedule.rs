use api::initialize_lambda;
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json,
};
use data_access::schedules::get_schedule_by_plan_id;
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans/:id/schedule", get(get_schedule)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn get_schedule(
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
