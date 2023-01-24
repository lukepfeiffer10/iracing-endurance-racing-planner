use crate::http::plans::patch_plan;
use crate::planner::{RacePlannerAction, RacePlannerContext};
use crate::{
    bindings,
    md_text_field::{MaterialTextField, MaterialTextFieldProps},
    planner::{
        format_date_time, format_duration, parse_duration_from_str, DurationFormat, DATE_FORMAT,
    },
};
use chrono::{NaiveDateTime, TimeZone, Utc};
use endurance_racing_planner_common::{EventConfigDto, PatchRacePlannerDto};
use gloo_console::error;
use uuid::Uuid;
use yew::context::ContextHandle;
use yew::{html, props, Callback, Component, Context, Html};

pub enum EventConfigMsg {
    ChangeGreenFlagOffset(String),
    ChangeSessionStart(String),
    ChangeRaceStartToD(String),
    ChangeRaceDuration(String),
    OnCreate(Uuid, EventConfigDto),
}

pub struct EventConfig {
    data: EventConfigDto,
    plan_id: Uuid,
    _planner_context_listener: ContextHandle<RacePlannerContext>,
}

impl Component for EventConfig {
    type Message = EventConfigMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, planner_context_listener) = ctx
            .link()
            .context::<RacePlannerContext>(ctx.link().batch_callback(
                |context: RacePlannerContext| -> Option<EventConfigMsg> {
                    match &context.data.overall_event_config {
                        Some(event_config) => Some(EventConfigMsg::OnCreate(
                            context.data.id,
                            event_config.clone(),
                        )),
                        None => None,
                    }
                },
            ))
            .expect("No Planner Context Provided");

        Self {
            data: planner_context
                .data
                .overall_event_config
                .clone()
                .unwrap_or_else(EventConfigDto::new),
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
                                overall_event_config: Some(self.data.clone()),
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
            EventConfigMsg::OnCreate(plan_id, config) => {
                self.plan_id = plan_id;
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
            on_change: link.callback(EventConfigMsg::ChangeRaceDuration)
        }};
        let session_start_utc_text_field_props = props! {MaterialTextFieldProps {
            value: format_date_time(self.data.session_start_utc.naive_utc()),
            label: Some("Session Start (UTC)".to_string()),
            id: "session-start-utc".to_string(),
            disabled: false,
            on_change: link.callback(EventConfigMsg::ChangeSessionStart)
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
            on_change: link.callback(EventConfigMsg::ChangeRaceStartToD)
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
            on_change: link.callback(EventConfigMsg::ChangeGreenFlagOffset)
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
    fn update_planner_context(&self, ctx: &Context<Self>) {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context to exist");
        planner_context.dispatch(RacePlannerAction::SetOverallEventConfig(self.data.clone()));
    }
}
