use api::{initialize_lambda, AuthenticatedUser};
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json,
};
use data_access::plans::get_plan_by_id;
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans/:id", get(get_plan)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn get_plan(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> impl IntoResponse {
    get_plan_by_id(&pool, id, user.id)
        .await
        .map(|plan| match plan {
            Some(plan) => Json(plan).into_response(),
            None => (StatusCode::NOT_FOUND).into_response(),
        })
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to get the plan").into_response()
        })
}
