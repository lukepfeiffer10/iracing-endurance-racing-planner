use yew::prelude::*;
use yew::{ props };
use chrono::{DateTime, Utc, Duration, NaiveDateTime, TimeZone};
use yew::services::ConsoleService;
use std::ops::{Add, Sub};
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use crate::fuel_stint_times::{FuelStintTimes};
use crate::overall_fuel_stint_config::OverallFuelStintConfig;
use crate::time_of_day_lap_factors::TimeOfDayLapFactors;

mod bindings;
mod md_text_field;
mod fuel_stint_times;
mod overall_fuel_stint_config;
mod event_bus;
mod duration_serde;
mod time_of_day_lap_factors;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub enum DurationFormat {
    HourMinSec,
    MinSecMilli
}

enum EventConfigMsg {
    ChangeGreenFlagOffset(String),
    ChangeSessionStart(String),
    ChangeRaceStartToD(String),
    ChangeRaceDuration(String)
}

struct EventConfig {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    race_duration: Duration,
    session_start_utc: DateTime<Utc>,
    race_start_utc: DateTime<Utc>,
    race_end_utc: DateTime<Utc>,
    race_start_tod: NaiveDateTime,
    race_end_tod: NaiveDateTime,
    green_flag_offset: Duration,
    tod_offset: Duration
}

impl EventConfig {
    fn update_race_times(&mut self) {
        self.race_start_utc = self.session_start_utc.add(self.green_flag_offset);
        self.race_end_utc = self.race_start_utc.add(self.race_duration);
        self.race_end_tod = self.race_start_tod.add(self.race_duration);
        self.tod_offset = self.race_start_tod.sub(self.race_start_utc.naive_utc());
    }
}

impl Component for EventConfig {
    type Message = EventConfigMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let utc_now = Utc::now();
        Self {
            link,
            race_duration: Duration::zero(),
            session_start_utc: utc_now,
            race_start_utc: utc_now,
            race_end_utc: utc_now,
            race_start_tod: utc_now.naive_utc(),
            race_end_tod: utc_now.naive_utc(),
            green_flag_offset: Duration::zero(),
            tod_offset: Duration::zero()
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            EventConfigMsg::ChangeGreenFlagOffset(offset) => {
                match parse_duration_from_str(offset.as_str(), DurationFormat::HourMinSec) {
                    Ok(duration) => {
                        self.green_flag_offset = duration;
                        self.update_race_times();
                        true
                    },
                    Err(e) => {
                        ConsoleService::error(format!("green flag offset parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeSessionStart(session_start) => {
                let parsed_session_start = NaiveDateTime::parse_from_str(session_start.as_str(), DATE_FORMAT);
                match parsed_session_start {
                    Ok(date) => {
                        self.session_start_utc = TimeZone::from_utc_datetime(&Utc,&date);
                        self.update_race_times();
                        true 
                    }
                    Err(e) => { 
                        ConsoleService::error(format!("session start parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeRaceStartToD(race_start_tod) => {
                let parsed_race_start_tod = NaiveDateTime::parse_from_str(race_start_tod.as_str(), DATE_FORMAT);
                match parsed_race_start_tod {
                    Ok(date) => {
                        self.race_start_tod = date;
                        self.update_race_times();
                        true
                    }
                    Err(e) => {
                        ConsoleService::error(format!("race start tod parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeRaceDuration(duration) => {
                match parse_duration_from_str(duration.as_str(), DurationFormat::HourMinSec) { 
                    Ok(duration) => {
                        self.race_duration = duration;
                        self.update_race_times();
                        true
                    },
                    Err(e) => {
                        ConsoleService::error(format!("race duration parse failure: {:?}", e).as_str());
                        false
                    }
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let race_duration_text_field_props = props!(MaterialTextFieldProps {
            value: format_duration(self.race_duration, DurationFormat::HourMinSec),
            label: Some("Race Duration (HH:MM:SS)".to_string()),
            id: "race_duration".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeRaceDuration(value))
        });
        let session_start_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.session_start_utc.naive_utc()),
            label: Some("Session Start (UTC)".to_string()),
            id: "session-start-utc".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeSessionStart(value))
        });
        let race_start_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_start_utc.naive_utc()),
            label: Some("Race Start (UTC)".to_string()),
            id: "race-start-utc".to_string(),
            disabled: true
        });
        let race_end_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_end_utc.naive_utc()),
            label: Some("Race End (UTC)".to_string()),
            id: "race-end-utc".to_string(),
            disabled: true
        });
        let race_start_tod_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_start_tod),
            label: Some("Race Start (ToD)".to_string()),
            id: "race-start-tod".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeRaceStartToD(value))
        });
        let race_end_tod_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_end_tod),
            label: Some("Race End (ToD)".to_string()),
            id: "race-end-tod".to_string(),
            disabled: true
        });
        let green_flag_offset_text_field_props = props!(MaterialTextFieldProps {
            value: format_duration(self.green_flag_offset, DurationFormat::HourMinSec),
            label: Some("Green Flag Offset (HH:MM:SS)".to_string()),
            id: "green-flag-offset".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeGreenFlagOffset(value))
        });
        let tod_offset_text_field_props = props!(MaterialTextFieldProps {
            value: format_duration(self.tod_offset, DurationFormat::HourMinSec),
            label: Some("ToD Offset (HH:MM:SS)".to_string()),
            id: "tod-offset".to_string(),
            disabled: true
        });
        html! {
            <div class="mdc-typography flex-container flex-row">
                <div id="left-column" class="flex-container flex-column">
                    <div id="overall-event-config" class="mdc-card">
                        <div class="mdc-card-wrapper__text-section">
                            <div class="card-title">{ "Overall Event Config" }</div>
                        </div>
                        <MaterialTextField with race_duration_text_field_props />
                        <MaterialTextField with session_start_utc_text_field_props />
                        <MaterialTextField with race_start_utc_text_field_props />
                        <MaterialTextField with race_end_utc_text_field_props />
                        <MaterialTextField with race_start_tod_text_field_props />
                        <MaterialTextField with race_end_tod_text_field_props />
                        <MaterialTextField with green_flag_offset_text_field_props />
                        <MaterialTextField with tod_offset_text_field_props />
                    </div>
                    <OverallFuelStintConfig />
                </div>
                <div id="right-column" class="flex-container flex-column">
                    <div class="flex-container flex-row flex-justify-content-center">
                        <FuelStintTimes />
                    </div>
                    <div class="flex-container flex-row">
                        <TimeOfDayLapFactors />
                        <div style="flex-grow: 2">
                            <h3>{ "Per Driver Lap Factors"} </h3>
                        </div>
                    </div>
                    <div class="flex-container flex-row">
                        <div style="flex-grow: 1">
                            <h3>{ "Realtime Deltas" }</h3>
                        </div>
                        <div style="flex-grow: 1">
                            <h3>{ "Manual Fuel Stint Calculator"} </h3>
                        </div>
                    </div>
                    <div class="flex-container flex-row">
                        <div style="flex-grow: 1">
                            <h3>{ "Final Fuel Stint Calculator" }</h3>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render { 
            bindings::enable_ripple(".mdc-text-field");
            bindings::enable_text_field(".mdc-text-field");
        }
    }
}

pub fn format_duration(duration: Duration, format: DurationFormat) -> String {
    match format {
        DurationFormat::HourMinSec => format!("{:02}:{:02}:{:02}", duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60),
        DurationFormat::MinSecMilli => {
            let prefix = if duration.num_milliseconds().is_negative() {
                "-"
            } else {
                ""
            };
            format!("{}{:02}:{:02}.{:03}", prefix, duration.num_minutes().abs() % 60, duration.num_seconds().abs() % 60, duration.num_milliseconds().abs() % 1000)
        }
    }
}

fn format_date_time(date_time: NaiveDateTime) -> String {
    date_time.format(DATE_FORMAT).to_string()
}

fn parse_duration_from_str(str: &str, format: DurationFormat) -> Result<Duration, &str> {
    let duration = match format {
       DurationFormat::HourMinSec => {
           let duration_split = str
               .split(':')
               .into_iter()
               .map(|part| part.parse::<i64>().unwrap())
               .collect::<Vec<_>>();

           let duration_seconds = match duration_split.len() {
               3 => {
                   Some((duration_split[0] * 60 + duration_split[1]) * 60 + duration_split[2])
               }
               2 => {
                   Some(duration_split[0] * 60 + duration_split[1])
               }
               1 => {
                   Some(duration_split[0])
               }
               _ => None
           };
           
           duration_seconds
               .map(|value| Duration::seconds(value))
       }
       DurationFormat::MinSecMilli => {
           let has_milliseconds = str.contains('.');
           let duration_split = str
               .split('.')
               .flat_map(|value| value.split(':'))               
               .map(|value| value.parse::<i64>().unwrap())
               .collect::<Vec<_>>();
           
           let duration_milliseconds = match duration_split.len() {
               3 => Some((duration_split[0] * 60 + duration_split[1]) * 1000 + duration_split[2]),
               2 => {
                   if has_milliseconds {
                       Some(duration_split[0] * 1000 + duration_split[1])
                   } else {
                       Some((duration_split[0] * 60 + duration_split[1]) * 1000)
                   }
               },
               1 => Some(duration_split[0] * 1000),
               _ => None
           };
           
           duration_milliseconds
               .map(|value| Duration::milliseconds(value))
       }
    };
    
    duration.ok_or("the duration could not be parsed")
}

fn main() {
    yew::start_app::<EventConfig>();
}