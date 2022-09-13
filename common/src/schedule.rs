use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{uuid_gen::uuid_time_nextval, EventConfigDto, FuelStintAverageTimes};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum StintType {
    FuelSavingNoTires,
    FuelSavingWithTires,
    StandardNoTires,
    StandardWithTires,
}

impl Display for StintType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StintType::FuelSavingNoTires => write!(f, "fs no tires"),
            StintType::FuelSavingWithTires => write!(f, "fs w/ tires"),
            StintType::StandardNoTires => write!(f, "std no tires"),
            StintType::StandardWithTires => write!(f, "std w/ tires"),
        }
    }
}

impl FromStr for StintType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fs no tires" => Ok(StintType::FuelSavingNoTires),
            "fs w/ tires" => Ok(StintType::FuelSavingWithTires),
            "std no tires" => Ok(StintType::StandardNoTires),
            "std w/ tires" => Ok(StintType::StandardWithTires),
            _ => Err(format!("{} cannot be mapped to a valid StintType", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleStintDto {
    pub id: Uuid,
    pub stint_type: StintType,
    pub fuel_stint_number: i32,
    pub utc_start: DateTime<Utc>,
    pub utc_end: DateTime<Utc>,
    pub tod_start: NaiveDateTime,
    pub tod_end: NaiveDateTime,
    pub actual_end: DateTime<Utc>,
    #[serde(with = "crate::duration_serde")]
    pub duration_delta: Duration,
    #[serde(with = "crate::duration_serde")]
    pub damage_modifier: Duration,
    pub calculated_laps: i32,
    pub actual_laps: i32,
    pub driver_name: String,
    pub availability: String,
    pub stint_number: i32,
    pub stint_preference: i32,
    pub factor: f32,
    pub local_start: NaiveDateTime,
    pub local_end: NaiveDateTime,
}

impl Eq for ScheduleStintDto {}

impl ScheduleStintDto {
    pub fn new(config: &EventConfigDto, fuel_stint_times: &FuelStintAverageTimes) -> Self {
        let stint_duration = fuel_stint_times.fuel_saving_stint.track_time_with_pit;
        Self {
            id: uuid_time_nextval(),
            stint_type: StintType::FuelSavingNoTires,
            fuel_stint_number: 1,
            utc_start: config.race_start_utc,
            utc_end: config.race_start_utc + stint_duration,
            tod_start: config.race_start_tod,
            tod_end: config.race_start_tod + stint_duration,
            actual_end: config.race_start_utc + stint_duration,
            duration_delta: Duration::zero(),
            damage_modifier: Duration::zero(),
            calculated_laps: fuel_stint_times.fuel_saving_stint.lap_count,
            actual_laps: fuel_stint_times.fuel_saving_stint.lap_count,
            driver_name: "".to_string(),
            availability: "".to_string(),
            stint_number: 1,
            stint_preference: 0,
            factor: 0.0,
            local_start: config.race_start_utc.naive_local(),
            local_end: config.race_start_utc.naive_local() + stint_duration,
        }
    }

    pub fn from_previous(
        previous_row: &ScheduleStintDto,
        stint_type: StintType,
        fuel_stint_times: &FuelStintAverageTimes,
        race_end_utc: DateTime<Utc>,
        tire_change_time: Duration,
        damage_modifier: Duration,
    ) -> Self {
        let utc_start = previous_row.utc_end.clone();
        let tod_start = previous_row.tod_end.clone();

        let (stint_duration, calculated_laps) = calculate_stint_duration_and_laps(
            utc_start,
            &stint_type,
            fuel_stint_times,
            race_end_utc,
            tire_change_time,
            damage_modifier,
        );

        Self {
            id: uuid_time_nextval(),
            stint_type,
            fuel_stint_number: previous_row.fuel_stint_number + 1,
            utc_start,
            utc_end: utc_start + stint_duration,
            tod_start,
            tod_end: tod_start + stint_duration,
            actual_end: utc_start + stint_duration,
            duration_delta: Duration::zero(),
            damage_modifier: previous_row.damage_modifier.clone(),
            calculated_laps,
            actual_laps: calculated_laps,
            driver_name: "".to_string(),
            availability: "".to_string(),
            stint_number: 1,
            stint_preference: 0,
            factor: 0.0,
            local_start: utc_start.naive_local(),
            local_end: utc_start.naive_local() + stint_duration,
        }
    }

    pub fn update(
        &mut self,
        utc_start: DateTime<Utc>,
        tod_start: NaiveDateTime,
        previous_row_driver_name: String,
        previous_row_stint_number: i32,
        fuel_stint_times: &FuelStintAverageTimes,
        race_end_utc: DateTime<Utc>,
        tire_change_time: Duration,
        damage_modifier: Duration,
    ) {
        let (stint_duration, calculated_laps) = calculate_stint_duration_and_laps(
            utc_start,
            &self.stint_type,
            fuel_stint_times,
            race_end_utc,
            tire_change_time,
            damage_modifier + self.damage_modifier,
        );
        self.utc_start = utc_start;
        self.utc_end = self.utc_start + stint_duration;
        self.actual_end = self.utc_end;
        self.tod_start = tod_start;
        self.tod_end = self.tod_start + stint_duration;
        self.calculated_laps = calculated_laps;
        self.actual_laps = calculated_laps;
        self.duration_delta = self.actual_end - self.utc_end;
        self.local_start = self.utc_start.naive_local();
        self.local_end = self.local_start + stint_duration;

        if (self.driver_name != "" && previous_row_driver_name != "")
            && previous_row_driver_name == self.driver_name
        {
            self.stint_number = previous_row_stint_number + 1;
        } else {
            self.stint_number = 1;
        }
    }
}

fn calculate_stint_duration_and_laps(
    stint_utc_start: DateTime<Utc>,
    stint_type: &StintType,
    fuel_stint_times: &FuelStintAverageTimes,
    race_end_utc: DateTime<Utc>,
    tire_change_time: Duration,
    damage_modifier: Duration,
) -> (Duration, i32) {
    let fuel_stint_data = match stint_type {
        StintType::FuelSavingNoTires | StintType::FuelSavingWithTires => {
            &fuel_stint_times.fuel_saving_stint
        }
        StintType::StandardNoTires | StintType::StandardWithTires => {
            &fuel_stint_times.standard_fuel_stint
        }
    };
    let track_time_with_pit = match stint_type {
        StintType::FuelSavingWithTires | StintType::StandardWithTires => {
            fuel_stint_data.track_time_with_pit + tire_change_time
        }
        StintType::FuelSavingNoTires | StintType::StandardNoTires => {
            fuel_stint_data.track_time_with_pit
        }
    };

    let total_stint_time =
        Duration::seconds((fuel_stint_data.lap_count as i64) * damage_modifier.num_seconds())
            + track_time_with_pit;

    if stint_utc_start + total_stint_time > race_end_utc {
        let stint_duration = race_end_utc - stint_utc_start;
        let lap_time = fuel_stint_data.lap_time + damage_modifier;
        let calculated_laps = (stint_duration.num_milliseconds() as f64
            / lap_time.num_milliseconds() as f64)
            .ceil() as i32;
        (stint_duration, calculated_laps)
    } else {
        (total_stint_time, fuel_stint_data.lap_count)
    }
}
