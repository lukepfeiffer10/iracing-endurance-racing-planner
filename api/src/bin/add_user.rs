use api::{bad_request_response, created_response};
use data_access::user::Users;
use endurance_racing_planner_common::User;
use lambda_http::{service_fn, Body, Error, IntoResponse, Request};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| add_user(event, db_context_ref);

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn add_user(event: Request, db_context_ref: &PgPool) -> Result<impl IntoResponse, Error> {
    Ok(match event.body() {
        Body::Text(json) => {
            let user = serde_json::from_str::<User>(json);

            match user {
                Ok(u) => {
                    let new_user = Users::create_user(db_context_ref, u).await?;
                    created_response(&new_user, format!("/users/{}", new_user.id))
                }
                Err(e) => bad_request_response(e.to_string()),
            }
        }
        _ => bad_request_response("Invalid body type".into()),
    })
}
