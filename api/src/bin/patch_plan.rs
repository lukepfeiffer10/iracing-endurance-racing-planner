use std::str::FromStr;

use api::{bad_request_response, ok_response, ApiResponse};
use data_access::entities::plan::PatchPlan;
use endurance_racing_planner_common::RacePlannerDto;
use lambda_http::{service_fn, Body, Error, IntoResponse, Request, RequestExt};
use sqlx::{types::Uuid, Pool, Postgres};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| patch_plan(event, db_context_ref);

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn patch_plan(
    event: Request,
    db_context: &Pool<Postgres>,
) -> Result<impl IntoResponse, Error> {
    Ok(match event.path_parameters().first("id") {
        Some(id) => match Uuid::from_str(id) {
            Ok(id) => match event.body() {
                Body::Text(json) => match serde_json::from_str::<RacePlannerDto>(json) {
                    Ok(plan) => {
                        let patch = if plan.overall_event_config.is_some() {
                            PatchPlan::EventConfig(plan.overall_event_config.unwrap())
                        } else {
                            PatchPlan::Title(plan.title)
                        };

                        data_access::plans::patch_plan(db_context, id, patch).await;
                        ok_response(ApiResponse { body: id })
                    }
                    Err(_) => bad_request_response("couldn't parse plan".into()),
                },
                _ => bad_request_response("id must be a uuid".into()),
            },
            _ => bad_request_response("no plan id".into()),
        },
        None => bad_request_response("no plan id".into()),
    })
}
