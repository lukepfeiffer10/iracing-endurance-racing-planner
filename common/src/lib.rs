mod duration_serde;

use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RacePlanner {
    pub id: Option<Uuid>,
    pub title: String,
    pub overall_event_config: Option<EventConfigData>,
    pub overall_fuel_stint_config: OverallFuelStintConfigData,
    pub fuel_stint_average_times: Option<FuelStintAverageTimes>,
    pub time_of_day_lap_factors: Vec<TimeOfDayLapFactor>,
    pub per_driver_lap_factors: Vec<DriverLapFactor>,
    pub driver_roster: Vec<Driver>,
    pub schedule_rows: Option<Vec<ScheduleDataRow>>,
}

impl RacePlanner {
    pub fn new(plan: RacePlanner) -> RacePlanner {
        RacePlanner {
            id: Some(Uuid::new_v4()),
            ..plan
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventConfigData {
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
    pub tod_offset: Duration
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OverallFuelStintConfigData {
    #[serde(with = "crate::duration_serde")]
    pub pit_duration: Duration,
    pub fuel_tank_size: i32,
    #[serde(with = "crate::duration_serde")]
    pub tire_change_time: Duration,
    pub add_tire_time: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FuelStintAverageTimes {
    pub standard_fuel_stint: StintData,
    pub fuel_saving_stint: StintData
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StintData {
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOfDayLapFactor {
    pub time_of_day: String,
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
    pub tod_start: NaiveTime,
    #[serde(with = "crate::duration_serde")]
    pub delta: Duration,
    pub factor: f64,
    pub has_edited_lap_time: bool
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverLapFactor {
    pub driver_name: String,
    pub driver_color: String,
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
    pub factor: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Driver {
    pub name: String,
    pub total_stints: i32,
    pub fair_share: bool,
    pub color: String,
    pub utc_offset: i32,
    pub irating: i32,
    pub stint_preference: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleDataRow {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum StintType {
    FuelSavingNoTires,
    FuelSavingWithTires,
    StandardNoTires,
    StandardWithTires,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleOpenIdClaims {
    pub iss: String,
    pub azp: String,
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub at_hash: String,
    pub nonce: String,
    pub name: String,
    pub picture: String,
    pub given_name: String,
    pub family_name: String,
    pub locale: String,
    pub jti: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub oauth_id: String
}