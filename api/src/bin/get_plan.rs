use std::str::FromStr;

use api::{bad_request_response, not_found_response, ok_response, ApiResponse};
use chrono::Duration;
use data_access::plans::get_plan_by_id;
use endurance_racing_planner_common::{EventConfigDto, RacePlannerDto};
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt};
use sqlx::{types::Uuid, Pool, Postgres};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| get_plan(event, db_context_ref);

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn get_plan(event: Request, db_context: &Pool<Postgres>) -> Result<impl IntoResponse, Error> {
    Ok(match event.path_parameters().first("id") {
        Some(id) => match Uuid::from_str(id) {
            Ok(id) => {
                let plan_with_overview = get_plan_by_id(db_context, id).await?;

                match plan_with_overview {
                    Some(plan) => {
                        let event_config = match plan.race_duration {
                            Some(race_duration) => Some(EventConfigDto {
                                race_duration: Duration::microseconds(race_duration.microseconds),
                                session_start_utc: plan.session_start_utc.unwrap(),
                                race_start_tod: plan.race_start_tod.unwrap(),
                                green_flag_offset: Duration::microseconds(
                                    plan.green_flag_offset.unwrap().microseconds,
                                ),
                            }),
                            None => None,
                        };
                        ok_response(ApiResponse {
                            body: RacePlannerDto {
                                id: plan.id,
                                title: plan.title,
                                overall_event_config: event_config,
                                overall_fuel_stint_config: None,
                                fuel_stint_average_times: None,
                                time_of_day_lap_factors: vec![],
                                per_driver_lap_factors: vec![],
                                driver_roster: vec![],
                                schedule_rows: None,
                            },
                        })
                    }
                    None => not_found_response(),
                }
            }
            Err(_) => bad_request_response("id must be a uuid".into()),
        },
        _ => bad_request_response("no plan id".into()),
    })
}
