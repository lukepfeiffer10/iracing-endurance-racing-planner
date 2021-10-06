use yew::prelude::*;
use yew::{ ChangeData };
use chrono::{DateTime, Utc, Duration, NaiveDateTime, TimeZone};
use yew::services::ConsoleService;
use std::ops::{Add, Sub};

mod bindings;

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

trait UpdateRaceTimes {
    fn update_race_times(&mut self);
}

impl UpdateRaceTimes for EventConfig {
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
                let duration_split: Vec<i64> = offset.split(":")
                    .into_iter()
                    .map(|part| part.parse::<i64>().unwrap())
                    .collect();
                
                ConsoleService::info(format!("Offset: {:?}", offset).as_ref());
                ConsoleService::info(format!("Offset Split: {:?}", duration_split).as_ref());
                let duration_seconds = (duration_split[0] * 60 + duration_split[1]) * 60 + duration_split[2];
                ConsoleService::info(format!("Offset Seconds: {:?}", duration_seconds).as_ref());
                self.green_flag_offset = Duration::seconds(duration_seconds);
                self.update_race_times();
                true
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
        let offset_onchange = self.link.batch_callback(|data: ChangeData| {
            match data { 
                ChangeData::Value(value) => Some(EventConfigMsg::ChangeGreenFlagOffset(value)),
                _ => None
            }
        });
        let session_start_onchange = self.link.batch_callback(|data: ChangeData| {
            match data {
                ChangeData::Value(value) => Some(EventConfigMsg::ChangeSessionStart(value)),
                _ => None
            }
        });
        let race_start_onchange = self.link.batch_callback(|data: ChangeData| {
           match data {
               ChangeData::Value(value) => Some(EventConfigMsg::ChangeRaceStartToD(value)),
               _ => None
           } 
        });
        let race_duration_onchange = self.link.batch_callback(|data: ChangeData| {
            match data {
                ChangeData::Value(value) => Some(EventConfigMsg::ChangeRaceDuration(value)),
                _ => None
            }
        });
        html! {
            <div id="global-event-config" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Global Event Config" }</div>
                </div>
                <label class="mdc-text-field mdc-text-field--filled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="race-duration">{ "Race Duration (HH:MM:SS)" }</span>
                    <input class="mdc-text-field__input" type="text" value=format_duration(self.race_duration) onchange=race_duration_onchange aria-labelledby="race-duration"/>
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="session-start-utc">{ "Session Start (UTC)" }</span>
                    <input class="mdc-text-field__input" type="text" value=format_date_time(self.session_start_utc.naive_utc()) onchange=session_start_onchange aria-labelledby="session-start-utc" />
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled mdc-text-field--disabled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="race-start-utc">{ "Race Start (UTC)" }</span>
                    <input class="mdc-text-field__input" type="text" disabled=true value=format_date_time(self.race_start_utc.naive_utc()) aria-labelledby="race-start-utc" />
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled mdc-text-field--disabled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="race-end-utc">{ "Race End (UTC)" }</span>
                    <input class="mdc-text-field__input" type="text" disabled=true value=format_date_time(self.race_end_utc.naive_utc()) aria-labelledby="race-end-utc" />
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="race-start-tod">{ "Race Start (ToD)" }</span>                    
                    <input class="mdc-text-field__input" type="text" value=format_date_time(self.race_start_tod) onchange=race_start_onchange aria-labelledby="race-start-tod"/>
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled mdc-text-field--disabled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="race-end-tod">{ "Race End (ToD)" }</span>
                    <input class="mdc-text-field__input" type="text" disabled=true value=format_date_time(self.race_end_tod) aria-labelledby="race-end-tod" />
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="green-flag-offset">{ "Green Flag Offset" }</span>
                    <input class="mdc-text-field__input" type="text" value=format_duration(self.green_flag_offset) onchange=offset_onchange aria-labelledby="green-flag-offset"/>
                    <span class="mdc-line-ripple"></span>
                </label>
                <label class="mdc-text-field mdc-text-field--filled mdc-text-field--disabled">
                    <span class="mdc-text-field__ripple"></span>
                    <span class="mdc-floating-label" id="tod-offset">{ "ToD Offset" }</span>
                    <input class="mdc-text-field__input" type="text" disabled=true value=format_duration(self.tod_offset) aria-labelledby="tod-offset" />
                    <span class="mdc-line-ripple"></span>
                </label>
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
    let duration_split: Vec<i64> = duration_string.split(":")
        .into_iter()
        .map(|part| part.parse::<i64>().unwrap())
        .collect();
    
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