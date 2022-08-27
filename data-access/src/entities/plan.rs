use chrono::{DateTime, NaiveDateTime, Utc};
use endurance_racing_planner_common::{
    EventConfigDto, OverallFuelStintConfigData, RacePlannerDto, StintDataDto,
};
use sqlx::{
    postgres::{types::PgInterval, PgValueRef},
    Decode, Postgres,
};
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
    pub pit_duration: Option<PgInterval>,
    pub fuel_tank_size: Option<i32>,
    pub tire_change_time: Option<PgInterval>,
    pub add_tire_time: Option<bool>,
}

pub struct PatchPlan {
    pub id: Uuid,
    pub modified_by: i32,
    pub modified_date: DateTime<Utc>,
    pub patch_type: PatchPlanType,
}

impl PatchPlan {
    pub fn new(id: Uuid, user_id: i32, patch_type: PatchPlanType) -> PatchPlan {
        PatchPlan {
            id,
            modified_by: user_id,
            modified_date: Utc::now(),
            patch_type,
        }
    }
}

pub enum PatchPlanType {
    Title(String),
    EventConfig(EventConfigDto),
    FuelStintConfig(OverallFuelStintConfigData),
    FuelStintAverageTime(StintDataDto, StintType),
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

pub struct FuelStintAverageTimes {
    pub plan_id: Uuid,
    pub lap_time: PgInterval,
    pub fuel_per_lap: f32,
    pub lap_count: i32,
    pub lap_time_with_pit: PgInterval,
    pub track_time: PgInterval,
    pub track_time_with_pit: PgInterval,
    pub fuel_per_stint: f32,
    pub stint_type: StintType,
}

#[repr(i16)]
pub enum StintType {
    Standard,
    FuelSaving,
}

impl TryFrom<i16> for StintType {
    type Error = Box<dyn std::error::Error + 'static + Sync + Send>;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StintType::Standard),
            1 => Ok(StintType::FuelSaving),
            _ => Err(format!("value: {} couldn't be converted to StintType", value).into()),
        }
    }
}

impl Decode<'_, Postgres> for StintType {
    fn decode(value: PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <i16 as Decode<Postgres>>::decode(value)?;

        value.try_into()
    }
}
