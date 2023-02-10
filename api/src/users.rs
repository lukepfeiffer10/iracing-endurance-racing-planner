use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use endurance_racing_planner_common::User;
use sqlx::PgPool;

use crate::{data_access::user::Users, AuthenticatedUser};

pub(crate) async fn me(user: AuthenticatedUser) -> impl IntoResponse {
    (StatusCode::OK, Json(user.0))
}

pub(crate) async fn add_user(
    State(pool): State<PgPool>,
    Json(user): Json<User>,
) -> impl IntoResponse {
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
