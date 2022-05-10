use chrono::{DateTime, Duration, Utc};
use endurance_racing_planner_common::{OverallFuelStintConfigData, RacePlannerDto};
use uuid::Uuid;

pub struct Plan {
    pub id: Uuid,
    pub title: String,
    pub created_by: i32,
    pub created_date: DateTime<Utc>,
}

impl From<RacePlannerDto> for Plan {
    fn from(plan: RacePlannerDto) -> Self {
        Plan {
            id: plan.id,
            title: plan.title,
            created_by: 0,
            created_date: Utc::now(),
        }
    }
}

impl From<Plan> for RacePlannerDto {
    fn from(plan: Plan) -> Self {
        (&plan).into()
    }
}

impl From<&Plan> for RacePlannerDto {
    fn from(plan: &Plan) -> Self {
        RacePlannerDto {
            id: plan.id,
            title: plan.title.clone(),
            overall_event_config: None,
            overall_fuel_stint_config: OverallFuelStintConfigData {
                pit_duration: Duration::zero(),
                fuel_tank_size: 0,
                tire_change_time: Duration::zero(),
                add_tire_time: false,
            },
            fuel_stint_average_times: None,
            time_of_day_lap_factors: vec![],
            per_driver_lap_factors: vec![],
            driver_roster: vec![],
            schedule_rows: None,
        }
    }
}
