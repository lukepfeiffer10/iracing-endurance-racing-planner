use api::initialize_lambda;
use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Json,
};
use chrono::Duration;
use data_access::plans::get_plan_by_id;
use endurance_racing_planner_common::{EventConfigDto, OverallFuelStintConfigData, RacePlannerDto};
use lambda_http::Error;
use sqlx::{types::Uuid, PgPool};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = initialize_lambda("/plans/:id", get(get_plan)).await?;

    lambda_http::run(handler).await?;
    Ok(())
}

async fn get_plan(Path(id): Path<Uuid>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    get_plan_by_id(&pool, id)
        .await
        .map(|plan_with_overview| match plan_with_overview {
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
                let fuel_stint_config = match plan.pit_duration {
                    Some(pit_duration) => Some(OverallFuelStintConfigData {
                        pit_duration: Duration::microseconds(pit_duration.microseconds),
                        fuel_tank_size: plan.fuel_tank_size.unwrap(),
                        tire_change_time: Duration::microseconds(
                            plan.tire_change_time.unwrap().microseconds,
                        ),
                        add_tire_time: plan.add_tire_time.unwrap(),
                    }),
                    None => None,
                };
                Json(RacePlannerDto {
                    id: plan.id,
                    title: plan.title,
                    overall_event_config: event_config,
                    overall_fuel_stint_config: fuel_stint_config,
                    fuel_stint_average_times: None,
                    time_of_day_lap_factors: vec![],
                    per_driver_lap_factors: vec![],
                    driver_roster: vec![],
                    schedule_rows: None,
                })
                .into_response()
            }
            None => (StatusCode::NOT_FOUND).into_response(),
        })
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to get the plan").into_response()
        })
}
