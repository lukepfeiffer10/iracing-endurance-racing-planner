use api::{bad_request_response, created_response, get_current_user, unauthorized_response};
use data_access::entities::Plan;
use data_access::plans::create_plan;
use endurance_racing_planner_common::RacePlannerDto;
use lambda_http::{service_fn, Body, Error, IntoResponse, Request};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| add_plan(event, db_context_ref);

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn add_plan(event: Request, db_context_ref: &PgPool) -> Result<impl IntoResponse, Error> {
    let current_user = get_current_user(event.headers(), db_context_ref).await;
    match current_user {
        Some(user) => Ok(match event.body() {
            Body::Text(json) => {
                let plan = serde_json::from_str::<RacePlannerDto>(json);
                match plan {
                    Ok(p) => {
                        let mut new_plan: Plan = p.into();
                        new_plan.created_by = user.id;

                        let new_plan: RacePlannerDto =
                            create_plan(db_context_ref, new_plan).await?.into();
                        created_response(&new_plan, format!("/plans/{}", new_plan.id))
                    }
                    Err(e) => bad_request_response(e.to_string()),
                }
            }
            _ => bad_request_response("Invalid body type".into()),
        }),
        None => Ok(unauthorized_response()),
    }
}
