use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use yew::{Component, Context, Html, html, props};
use yew_agent::{Bridge, Bridged};
use gloo_console::error;
use serde::{Serialize, Deserialize};
use crate::{bindings, planner::{DATE_FORMAT, DurationFormat, parse_duration_from_str, format_duration, format_date_time}, md_text_field::{MaterialTextFieldProps, MaterialTextField}, event_bus::EventBus};
use crate::event_bus::{EventBusInput, EventBusOutput};

pub enum EventConfigMsg {
    ChangeGreenFlagOffset(String),
    ChangeSessionStart(String),
    ChangeRaceStartToD(String),
    ChangeRaceDuration(String),
    OnCreate(EventConfigData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl EventConfigData {
    pub fn new() -> Self {
        //let utc_now = Utc::now();
        Self {
            race_duration: Duration::hours(10),
            session_start_utc: DateTime::from_utc(NaiveDate::from_ymd(2021, 10, 2).and_hms(12, 0, 0), Utc),
            race_start_utc: DateTime::from_utc(NaiveDate::from_ymd(2021, 10, 2).and_hms(12, 43, 0), Utc),
            race_end_utc: DateTime::from_utc(NaiveDate::from_ymd(2021, 10, 2).and_hms(22, 43, 0), Utc),
            race_start_tod: NaiveDate::from_ymd(2021, 11, 13).and_hms(11, 30, 0),
            race_end_tod: NaiveDate::from_ymd(2021, 11, 13).and_hms(21, 30, 0),
            green_flag_offset: Duration::minutes(43),
            tod_offset: Duration::zero()
        }
    }

    fn update_race_times(&mut self) {
        self.race_start_utc = self.session_start_utc + self.green_flag_offset;
        self.race_end_utc = self.race_start_utc + self.race_duration;
        self.race_end_tod = self.race_start_tod + self.race_duration;
        self.tod_offset = self.race_start_tod - self.race_start_utc.naive_utc();
    }
}

pub struct EventConfig {
    event_bus: Box<dyn Bridge<EventBus>>,
    data: EventConfigData,
}

impl Component for EventConfig {
    type Message = EventConfigMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut event_bus = EventBus::bridge(ctx.link().batch_callback(|message| {
            match message {
                EventBusOutput::SendOverallEventConfig(config) => {
                    config.map(|d| EventConfigMsg::OnCreate(d))
                }
                _ => None
            }
        }));
        event_bus.send(EventBusInput::GetOverallEventConfig);
        Self {
            event_bus, 
            data: EventConfigData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EventConfigMsg::ChangeGreenFlagOffset(offset) => {
                match parse_duration_from_str(offset.as_str(), DurationFormat::HourMinSec) {
                    Ok(duration) => {
                        self.data.green_flag_offset = duration;
                        self.data.update_race_times();
                        true
                    },
                    Err(e) => {
                        error!(format!("green flag offset parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeSessionStart(session_start) => {
                let parsed_session_start = NaiveDateTime::parse_from_str(session_start.as_str(), DATE_FORMAT);
                match parsed_session_start {
                    Ok(date) => {
                        self.data.session_start_utc = TimeZone::from_utc_datetime(&Utc,&date);
                        self.data.update_race_times();
                        true
                    }
                    Err(e) => {
                        error!(format!("session start parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeRaceStartToD(race_start_tod) => {
                let parsed_race_start_tod = NaiveDateTime::parse_from_str(race_start_tod.as_str(), DATE_FORMAT);
                match parsed_race_start_tod {
                    Ok(date) => {
                        self.data.race_start_tod = date;
                        self.data.update_race_times();
                        true
                    }
                    Err(e) => {
                        error!(format!("race start tod parse failure: {:?}", e).as_str());
                        false
                    }
                }
            },
            EventConfigMsg::ChangeRaceDuration(duration) => {
                match parse_duration_from_str(duration.as_str(), DurationFormat::HourMinSec) {
                    Ok(duration) => {
                        self.data.race_duration = duration;
                        self.data.update_race_times();
                        true
                    },
                    Err(e) => {
                        error!(format!("race duration parse failure: {:?}", e).as_str());
                        false
                    }
                }
            }
            EventConfigMsg::OnCreate(config) => {
                self.data = config;
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let race_duration_text_field_props = props!{MaterialTextFieldProps {
            value: format_duration(self.data.race_duration, DurationFormat::HourMinSec),
            label: Some("Race Duration (HH:MM:SS)".to_string()),
            id: "race_duration".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeRaceDuration(value))
        }};
        let session_start_utc_text_field_props = props!{MaterialTextFieldProps {
            value: format_date_time(self.data.session_start_utc.naive_utc()),
            label: Some("Session Start (UTC)".to_string()),
            id: "session-start-utc".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeSessionStart(value))
        }};
        let race_start_utc_text_field_props = props!{MaterialTextFieldProps {
            value: format_date_time(self.data.race_start_utc.naive_utc()),
            label: Some("Race Start (UTC)".to_string()),
            id: "race-start-utc".to_string(),
            disabled: true
        }};
        let race_end_utc_text_field_props = props!{MaterialTextFieldProps {
            value: format_date_time(self.data.race_end_utc.naive_utc()),
            label: Some("Race End (UTC)".to_string()),
            id: "race-end-utc".to_string(),
            disabled: true
        }};
        let race_start_tod_text_field_props = props!{MaterialTextFieldProps {
            value: format_date_time(self.data.race_start_tod),
            label: Some("Race Start (ToD)".to_string()),
            id: "race-start-tod".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeRaceStartToD(value))
        }};
        let race_end_tod_text_field_props = props!{MaterialTextFieldProps {
            value: format_date_time(self.data.race_end_tod),
            label: Some("Race End (ToD)".to_string()),
            id: "race-end-tod".to_string(),
            disabled: true
        }};
        let green_flag_offset_text_field_props = props!{MaterialTextFieldProps {
            value: format_duration(self.data.green_flag_offset, DurationFormat::HourMinSec),
            label: Some("Green Flag Offset (HH:MM:SS)".to_string()),
            id: "green-flag-offset".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeGreenFlagOffset(value))
        }};
        let tod_offset_text_field_props = props!{MaterialTextFieldProps {
            value: format_duration(self.data.tod_offset, DurationFormat::HourMinSec),
            label: Some("ToD Offset (HH:MM:SS)".to_string()),
            id: "tod-offset".to_string(),
            disabled: true
        }};
        html! {
            <div id="overall-event-config" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Overall Event Config" }</div>
                </div>
                <MaterialTextField ..race_duration_text_field_props />
                <MaterialTextField ..session_start_utc_text_field_props />
                <MaterialTextField ..race_start_utc_text_field_props />
                <MaterialTextField ..race_end_utc_text_field_props />
                <MaterialTextField ..race_start_tod_text_field_props />
                <MaterialTextField ..race_end_tod_text_field_props />
                <MaterialTextField ..green_flag_offset_text_field_props />
                <MaterialTextField ..tod_offset_text_field_props />
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            bindings::enable_ripple(".mdc-text-field");
            bindings::enable_text_field(".mdc-text-field");
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.event_bus.send(EventBusInput::PutOverallEventConfig(self.data.clone()))
    }
}
