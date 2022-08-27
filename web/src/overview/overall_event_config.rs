use crate::http::plans::patch_plan;
use crate::planner::{PlannerContext, PlannerContextAction};
use crate::{
    bindings,
    md_text_field::{MaterialTextField, MaterialTextFieldProps},
    planner::{
        format_date_time, format_duration, parse_duration_from_str, DurationFormat, DATE_FORMAT,
    },
};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use endurance_racing_planner_common::{EventConfigDto, PatchRacePlannerDto};
use gloo_console::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::context::ContextHandle;
use yew::{html, props, Callback, Component, Context, Html};

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
    pub tod_offset: Duration,
}

impl EventConfigData {
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

    fn update_race_times(&mut self) {
        self.race_start_utc = self.session_start_utc + self.green_flag_offset;
        self.race_end_utc = self.race_start_utc + self.race_duration;
        self.race_end_tod = self.race_start_tod + self.race_duration;
        self.tod_offset = self.race_start_tod - self.race_start_utc.naive_utc();
    }
}

impl From<&EventConfigDto> for EventConfigData {
    fn from(dto: &EventConfigDto) -> Self {
        let utc_now = Utc::now();
        Self {
            race_duration: dto.race_duration,
            session_start_utc: dto.session_start_utc,
            race_start_utc: utc_now,
            race_end_utc: utc_now,
            race_start_tod: dto.race_start_tod,
            race_end_tod: utc_now.naive_utc(),
            green_flag_offset: dto.green_flag_offset,
            tod_offset: Duration::zero(),
        }
    }
}

pub struct EventConfig {
    data: EventConfigData,
    plan_id: Uuid,
    _planner_context_listener: ContextHandle<PlannerContext>,
}

impl Component for EventConfig {
    type Message = EventConfigMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, planner_context_listener) = ctx
            .link()
            .context::<PlannerContext>(ctx.link().batch_callback(
                |context: PlannerContext| -> Option<EventConfigMsg> {
                    match &context.data.overall_event_config {
                        Some(event_config) => {
                            let mut config_data: EventConfigData = event_config.into();
                            config_data.update_race_times();
                            Some(EventConfigMsg::OnCreate(config_data))
                        }
                        None => None,
                    }
                },
            ))
            .expect("No Planner Context Provided");

        let mut data = planner_context
            .data
            .overall_event_config
            .as_ref()
            .map(|dto| dto.into())
            .unwrap_or_else(|| EventConfigData::new());
        data.update_race_times();

        Self {
            data,
            plan_id: planner_context.data.id,
            _planner_context_listener: planner_context_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut should_update = false;
        match msg {
            EventConfigMsg::ChangeGreenFlagOffset(offset) => {
                match parse_duration_from_str(offset.as_str(), DurationFormat::HourMinSec) {
                    Ok(duration) => {
                        self.data.green_flag_offset = duration;
                        self.data.update_race_times();

                        patch_plan(
                            self.plan_id,
                            PatchRacePlannerDto {
                                id: self.plan_id,
                                title: None,
                                overall_event_config: Some(EventConfigDto {
                                    race_duration: self.data.race_duration,
                                    session_start_utc: self.data.session_start_utc,
                                    race_start_tod: self.data.race_start_tod,
                                    green_flag_offset: self.data.green_flag_offset,
                                }),
                                overall_fuel_stint_config: None,
                                fuel_stint_average_times: None,
                                time_of_day_lap_factors: None,
                                per_driver_lap_factors: None,
                                schedule_rows: None,
                                driver_roster: None,
                            },
                        );
                        should_update = true;
                    }
                    Err(e) => {
                        error!(format!("green flag offset parse failure: {:?}", e).as_str());
                    }
                }
            }
            EventConfigMsg::ChangeSessionStart(session_start) => {
                let parsed_session_start =
                    NaiveDateTime::parse_from_str(session_start.as_str(), DATE_FORMAT);
                match parsed_session_start {
                    Ok(date) => {
                        self.data.session_start_utc = TimeZone::from_utc_datetime(&Utc, &date);
                        self.data.update_race_times();
                        should_update = true;
                    }
                    Err(e) => {
                        error!(format!("session start parse failure: {:?}", e).as_str());
                    }
                }
            }
            EventConfigMsg::ChangeRaceStartToD(race_start_tod) => {
                let parsed_race_start_tod =
                    NaiveDateTime::parse_from_str(race_start_tod.as_str(), DATE_FORMAT);
                match parsed_race_start_tod {
                    Ok(date) => {
                        self.data.race_start_tod = date;
                        self.data.update_race_times();
                        should_update = true;
                    }
                    Err(e) => {
                        error!(format!("race start tod parse failure: {:?}", e).as_str());
                    }
                }
            }
            EventConfigMsg::ChangeRaceDuration(duration) => {
                match parse_duration_from_str(duration.as_str(), DurationFormat::HourMinSec) {
                    Ok(duration) => {
                        self.data.race_duration = duration;
                        self.data.update_race_times();
                        should_update = true;
                    }
                    Err(e) => {
                        error!(format!("race duration parse failure: {:?}", e).as_str());
                    }
                }
            }
            EventConfigMsg::OnCreate(config) => {
                self.data = config;
                should_update = true;
            }
        }

        if should_update {
            self.update_planner_context(ctx);
        }
        should_update
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let race_duration_text_field_props = props! {MaterialTextFieldProps {
            value: format_duration(self.data.race_duration, DurationFormat::HourMinSec),
            label: Some("Race Duration (HH:MM:SS)".to_string()),
            id: "race_duration".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeRaceDuration(value))
        }};
        let session_start_utc_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.session_start_utc.naive_utc()),
            label: Some("Session Start (UTC)".to_string()),
            id: "session-start-utc".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeSessionStart(value))
        }};
        let race_start_utc_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.race_start_utc.naive_utc()),
            label: Some("Race Start (UTC)".to_string()),
            id: "race-start-utc".to_string(),
            disabled: true
        }};
        let race_end_utc_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.race_end_utc.naive_utc()),
            label: Some("Race End (UTC)".to_string()),
            id: "race-end-utc".to_string(),
            disabled: true
        }};
        let race_start_tod_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.race_start_tod),
            label: Some("Race Start (ToD)".to_string()),
            id: "race-start-tod".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeRaceStartToD(value))
        }};
        let race_end_tod_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.race_end_tod),
            label: Some("Race End (ToD)".to_string()),
            id: "race-end-tod".to_string(),
            disabled: true
        }};
        let green_flag_offset_text_field_props = props! {MaterialTextFieldProps {
            value: format_duration(self.data.green_flag_offset, DurationFormat::HourMinSec),
            label: Some("Green Flag Offset (HH:MM:SS)".to_string()),
            id: "green-flag-offset".to_string(),
            disabled: false,
            on_change: link.callback(|value| EventConfigMsg::ChangeGreenFlagOffset(value))
        }};
        let tod_offset_text_field_props = props! {MaterialTextFieldProps {
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
}

impl EventConfig {
    fn update_planner_context(&self, ctx: &Context<Self>) -> () {
        let (planner_context, _) = ctx
            .link()
            .context::<PlannerContext>(Callback::noop())
            .expect("planner context to exist");
        planner_context
            .dispatch
            .emit(PlannerContextAction::UpdateOverallEventConfig(
                self.data.clone(),
            ));
    }
}
