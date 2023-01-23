use api::initialize_lambda;
use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Extension, Json,
};
use data_access::user::Users;
use endurance_racing_planner_common::User;
use lambda_http::Error;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/users", post(add_user)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn add_user(Json(user): Json<User>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let new_user_result = Users::create_user(&pool, user).await;
    match new_user_result {
        Ok(new_user) => (
            StatusCode::CREATED,
            [(header::CONTENT_LOCATION, format!("/users/{}", new_user.id))],
            Json(new_user),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "there was a problem creating the user",
        )
            .into_response(),
    }
}
