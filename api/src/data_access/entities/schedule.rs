use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use endurance_racing_planner_common::schedule::ScheduleStintDto;
use sqlx::{
    postgres::{types::PgInterval, PgValueRef},
    Decode, Postgres,
};
use uuid::Uuid;

#[repr(i16)]
#[derive(Clone)]
pub enum StintType {
    FuelSavingNoTires,
    FuelSavingWithTires,
    StandardNoTires,
    StandardWithTires,
}

impl TryFrom<i16> for StintType {
    type Error = Box<dyn std::error::Error + 'static + Sync + Send>;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StintType::FuelSavingNoTires),
            1 => Ok(StintType::FuelSavingWithTires),
            2 => Ok(StintType::StandardNoTires),
            3 => Ok(StintType::StandardWithTires),
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

impl From<endurance_racing_planner_common::schedule::StintType> for StintType {
    fn from(dto_stint_type: endurance_racing_planner_common::schedule::StintType) -> Self {
        match dto_stint_type {
            endurance_racing_planner_common::schedule::StintType::FuelSavingNoTires => {
                Self::FuelSavingNoTires
            }
            endurance_racing_planner_common::schedule::StintType::FuelSavingWithTires => {
                Self::FuelSavingWithTires
            }
            endurance_racing_planner_common::schedule::StintType::StandardNoTires => {
                Self::StandardNoTires
            }
            endurance_racing_planner_common::schedule::StintType::StandardWithTires => {
                Self::StandardWithTires
            }
        }
    }
}

impl From<StintType> for endurance_racing_planner_common::schedule::StintType {
    fn from(val: StintType) -> Self {
        match val {
            StintType::FuelSavingNoTires => {
                endurance_racing_planner_common::schedule::StintType::FuelSavingNoTires
            }
            StintType::FuelSavingWithTires => {
                endurance_racing_planner_common::schedule::StintType::FuelSavingWithTires
            }
            StintType::StandardNoTires => {
                endurance_racing_planner_common::schedule::StintType::StandardNoTires
            }
            StintType::StandardWithTires => {
                endurance_racing_planner_common::schedule::StintType::StandardWithTires
            }
        }
    }
}

pub struct Stint {
    pub id: Uuid,
    pub stint_type: StintType,
    pub number: i32,
    pub utc_start: DateTime<Utc>,
    pub utc_end: DateTime<Utc>,
    pub tod_start: NaiveDateTime,
    pub tod_end: NaiveDateTime,
    pub actual_end: DateTime<Utc>,
    pub duration_delta: PgInterval,
    pub damage_modifier: PgInterval,
    pub calculated_laps: i32,
    pub actual_laps: i32,
    pub driver_stint_count: i32,
    pub driver_id: Option<i32>,
}

impl From<&ScheduleStintDto> for Stint {
    fn from(dto: &ScheduleStintDto) -> Self {
        Self {
            id: dto.id,
            stint_type: dto.stint_type.clone().into(),
            number: dto.fuel_stint_number,
            utc_start: dto.utc_start,
            utc_end: dto.utc_end,
            tod_start: dto.tod_start,
            tod_end: dto.tod_end,
            actual_end: dto.actual_end,
            duration_delta: dto.duration_delta.try_into().unwrap(),
            damage_modifier: dto.damage_modifier.try_into().unwrap(),
            calculated_laps: dto.calculated_laps,
            actual_laps: dto.actual_laps,
            driver_stint_count: dto.stint_number,
            driver_id: if dto.driver_id == 0 {
                None
            } else {
                Some(dto.driver_id)
            },
        }
    }
}

impl From<&Stint> for ScheduleStintDto {
    fn from(val: &Stint) -> Self {
        ScheduleStintDto {
            id: val.id,
            stint_type: val.stint_type.clone().into(),
            fuel_stint_number: val.number,
            utc_start: val.utc_start,
            utc_end: val.utc_end,
            tod_start: val.tod_start,
            tod_end: val.tod_end,
            actual_end: val.actual_end,
            duration_delta: Duration::microseconds(val.duration_delta.microseconds),
            damage_modifier: Duration::microseconds(val.damage_modifier.microseconds),
            calculated_laps: val.calculated_laps,
            actual_laps: val.actual_laps,
            driver_id: val.driver_id.unwrap_or_default(),
            availability: "".into(),
            stint_number: val.driver_stint_count,
            factor: 1_f32,
        }
    }
}
