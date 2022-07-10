use chrono::{DateTime, NaiveDateTime, Utc};
use endurance_racing_planner_common::{EventConfigDto, RacePlannerDto};
use sqlx::postgres::types::PgInterval;
use uuid::Uuid;

pub struct Plan {
    pub id: Uuid,
    pub title: String,
    pub created_by: i32,
    pub created_date: DateTime<Utc>,
    pub modified_by: Option<i32>,
    pub modified_date: Option<DateTime<Utc>>,
}

pub struct PlanWithOwner {
    pub id: Uuid,
    pub title: String,
    pub created_by: i32,
    pub created_date: DateTime<Utc>,
    pub modified_by: Option<i32>,
    pub modified_date: Option<DateTime<Utc>>,
    pub owner: String,
}

pub struct PlanWithOverview {
    pub id: Uuid,
    pub title: String,
    pub race_duration: Option<PgInterval>,
    pub session_start_utc: Option<DateTime<Utc>>,
    pub race_start_tod: Option<NaiveDateTime>,
    pub green_flag_offset: Option<PgInterval>,
}

pub enum PatchPlan {
    Title(String),
    EventConfig(EventConfigDto),
}

impl From<RacePlannerDto> for Plan {
    fn from(plan: RacePlannerDto) -> Self {
        Plan {
            id: plan.id,
            title: plan.title,
            created_by: 0,
            created_date: Utc::now(),
            modified_by: None,
            modified_date: None,
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
            overall_fuel_stint_config: None,
            fuel_stint_average_times: None,
            time_of_day_lap_factors: vec![],
            per_driver_lap_factors: vec![],
            driver_roster: vec![],
            schedule_rows: None,
        }
    }
}
