use yew::prelude::*;
use yew::{ props };
use chrono::{DateTime, Utc, Duration, NaiveDateTime, TimeZone};
use yew::services::ConsoleService;
use std::ops::{Add, Sub};
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};

mod bindings;
mod md_text_field;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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
                match parse_duration_from_str(offset.as_str()) {
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
                match parse_duration_from_str(duration.as_str()) { 
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
            value: format_duration(self.race_duration),
            label: "Race Duration (HH:MM:SS)".to_string(),
            id: "race_duration".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeRaceDuration(value))
        });
        let session_start_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.session_start_utc.naive_utc()),
            label: "Session Start (UTC)".to_string(),
            id: "session-start-utc".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeSessionStart(value))
        });
        let race_start_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_start_utc.naive_utc()),
            label: "Race Start (UTC)".to_string(),
            id: "race-start-utc".to_string(),
            disabled: true
        });
        let race_end_utc_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_end_utc.naive_utc()),
            label: "Race End (UTC)".to_string(),
            id: "race-end-utc".to_string(),
            disabled: true
        });
        let race_start_tod_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_start_tod),
            label: "Race Start (ToD)".to_string(),
            id: "race-start-tod".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeRaceStartToD(value))
        });
        let race_end_tod_text_field_props = props!(MaterialTextFieldProps {
            value: format_date_time(self.race_end_tod),
            label: "Race End (ToD)".to_string(),
            id: "race-end-tod".to_string(),
            disabled: true
        });
        let green_flag_offset_text_field_props = props!(MaterialTextFieldProps {
            value: format_duration(self.green_flag_offset),
            label: "Green Flag Offset (HH:MM:SS)".to_string(),
            id: "green-flag-offset".to_string(),
            disabled: false,
            on_change: self.link.callback(|value| EventConfigMsg::ChangeGreenFlagOffset(value))
        });
        let tod_offset_text_field_props = props!(MaterialTextFieldProps {
            value: format_duration(self.tod_offset),
            label: "ToD Offset (HH:MM:SS)".to_string(),
            id: "tod-offset".to_string(),
            disabled: true
        });
        html! {
            <div id="global-event-config" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Global Event Config" }</div>
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
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render { 
            bindings::enable_ripple(".mdc-text-field");
            bindings::enable_text_field(".mdc-text-field");
        }
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:02}:{:02}:{:02}", duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60)
}

fn format_date_time(date_time: NaiveDateTime) -> String {
    date_time.format(DATE_FORMAT).to_string()
}

fn parse_duration_from_str(duration_string: &str) -> Result<Duration, &str> {
    let duration_split = duration_string
        .split(':')
        .into_iter()
        .map(|part| part.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    
    if duration_split.len() != 3 {
        Err("the duration string was not in the proper format of 00:00:00")
    } else {
        let duration_seconds = (duration_split[0] * 60 + duration_split[1]) * 60 + duration_split[2];
        Ok(Duration::seconds(duration_seconds))
    }
}

fn main() {
    yew::start_app::<EventConfig>();
}