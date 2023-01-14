mod duration_serde;
mod option_duration_serde;
pub mod schedule;
pub mod uuid_gen;

use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, Utc};
use schedule::ScheduleStintDto;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RacePlannerDto {
    pub id: Uuid,
    pub title: String,
    pub overall_event_config: Option<EventConfigDto>,
    pub overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
    pub fuel_stint_average_times: Option<FuelStintAverageTimes>,
    pub time_of_day_lap_factors: Vec<TimeOfDayLapFactor>,
    pub per_driver_lap_factors: Vec<DriverLapFactor>,
    pub driver_roster: Vec<Driver>,
    pub schedule_rows: Option<Vec<ScheduleStintDto>>,
}

impl RacePlannerDto {
    pub fn new() -> RacePlannerDto {
        RacePlannerDto {
            id: Uuid::new_v4(),
            title: "New Plan".into(),
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

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatchRacePlannerDto {
    pub id: Uuid,
    pub title: Option<String>,
    pub overall_event_config: Option<EventConfigDto>,
    pub overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
    pub fuel_stint_average_times: Option<PatchFuelStintAverageTimes>,
    pub time_of_day_lap_factors: Option<Vec<TimeOfDayLapFactor>>,
    pub per_driver_lap_factors: Option<Vec<DriverLapFactor>>,
    pub driver_roster: Option<Vec<Driver>>,
    pub schedule_rows: Option<Vec<ScheduleStintDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EventConfigDto {
    #[serde(with = "crate::duration_serde")]
    pub race_duration: Duration,
    pub session_start_utc: DateTime<Utc>,
    pub race_start_utc: DateTime<Utc>,
    pub race_end_utc: DateTime<Utc>,
    pub race_start_tod: NaiveDateTime,
    pub race_end_tod: NaiveDateTime,
    #[serde(with = "crate::duration_serde")]
    pub green_flag_offset: Duration,
    #[serde(with = "crate::duration_serde")]
    pub tod_offset: Duration,
}

impl EventConfigDto {
    pub fn new() -> Self {
        let utc_now = Utc::now();
        Self {
            race_duration: Duration::zero(),
            session_start_utc: utc_now,
            race_start_utc: utc_now,
            race_end_utc: utc_now,
            race_start_tod: utc_now.naive_local(),
            race_end_tod: utc_now.naive_local(),
            green_flag_offset: Duration::zero(),
            tod_offset: Duration::zero(),
        }
    }

    pub fn update_race_times(&mut self) {
        self.race_start_utc = self.session_start_utc + self.green_flag_offset;
        self.race_end_utc = self.race_start_utc + self.race_duration;
        self.race_end_tod = self.race_start_tod + self.race_duration;
        self.tod_offset = self.race_start_tod - self.race_start_utc.naive_utc();
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverallFuelStintConfigData {
    #[serde(with = "crate::duration_serde")]
    pub pit_duration: Duration,
    pub fuel_tank_size: i32,
    #[serde(with = "crate::duration_serde")]
    pub tire_change_time: Duration,
    pub add_tire_time: bool,
}

impl OverallFuelStintConfigData {
    pub fn new() -> Self {
        Self {
            pit_duration: Duration::zero(),
            fuel_tank_size: 0,
            tire_change_time: Duration::zero(),
            add_tire_time: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuelStintAverageTimes {
    pub standard_fuel_stint: StintDataDto,
    pub fuel_saving_stint: StintDataDto,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatchFuelStintAverageTimes {
    pub standard_fuel_stint: Option<StintDataDto>,
    pub fuel_saving_stint: Option<StintDataDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StintDataDto {
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
    pub fuel_per_lap: f32,
    pub lap_count: i32,
    #[serde(with = "crate::duration_serde")]
    pub lap_time_with_pit: Duration,
    #[serde(with = "crate::duration_serde")]
    pub track_time: Duration,
    #[serde(with = "crate::duration_serde")]
    pub track_time_with_pit: Duration,
    pub fuel_per_stint: f32,
}

impl Eq for StintDataDto {}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeOfDayLapFactor {
    pub time_of_day: String,
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
    pub tod_start: NaiveTime,
    #[serde(with = "crate::duration_serde")]
    pub delta: Duration,
    pub factor: f64,
    pub has_edited_lap_time: bool,
}

impl Eq for TimeOfDayLapFactor {}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DriverLapFactor {
    pub driver_name: String,
    pub driver_color: String,
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
    pub factor: f64,
}

impl Eq for DriverLapFactor {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Driver {
    pub id: i32,
    pub name: String,
    pub total_stints: i32,
    pub fair_share: bool,
    pub color: String,
    pub utc_offset: i16,
    pub irating: i16,
    pub stint_preference: i16,
    #[serde(default)]
    #[serde(with = "crate::option_duration_serde")]
    pub lap_time: Option<Duration>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchDriver {
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleOpenIdClaims {
    pub iss: String,
    pub azp: Option<String>,
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub at_hash: Option<String>,
    pub nonce: String,
    pub name: String,
    pub picture: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub locale: Option<String>,
    pub jti: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub oauth_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanListDto {
    pub id: Uuid,
    pub title: String,
    pub owner: String,
    pub last_modified: DateTime<Utc>,
}
