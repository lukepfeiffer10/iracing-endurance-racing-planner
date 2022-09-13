use crate::http::plans::patch_plan;
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use crate::planner::{
    format_duration, parse_duration_from_str, DurationFormat, RacePlannerAction, RacePlannerContext,
};
use endurance_racing_planner_common::{OverallFuelStintConfigData, PatchRacePlannerDto};
use gloo_console::error;
use web_sys::HtmlInputElement;
use yew::context::ContextHandle;
use yew::prelude::*;
use yew::props;
use yew::NodeRef;

pub enum OverallFuelStintMessage {
    UpdatePitDuration(String),
    UpdateFuelTankSize(String),
    UpdateTireChangeTime(String),
    UpdateAddTireTire(bool),
    OnCreate(OverallFuelStintConfigData),
}

pub struct OverallFuelStintConfig {
    data: OverallFuelStintConfigData,
    add_tire_time_input_ref: NodeRef,
    _planner_context_listener: ContextHandle<RacePlannerContext>,
}

impl Component for OverallFuelStintConfig {
    type Message = OverallFuelStintMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, planner_context_listener) = ctx
            .link()
            .context::<RacePlannerContext>(ctx.link().batch_callback(
                |context: RacePlannerContext| -> Option<OverallFuelStintMessage> {
                    match &context.data.overall_fuel_stint_config {
                        Some(fuel_stint_config) => {
                            Some(OverallFuelStintMessage::OnCreate(fuel_stint_config.clone()))
                        }
                        None => None,
                    }
                },
            ))
            .expect("No Planner Context Provided");

        Self {
            data: planner_context
                .data
                .overall_fuel_stint_config
                .as_ref()
                .map(|data| data.clone())
                .unwrap_or_else(|| OverallFuelStintConfigData::new()),
            add_tire_time_input_ref: NodeRef::default(),
            _planner_context_listener: planner_context_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context");
        match msg {
            OverallFuelStintMessage::UpdatePitDuration(value) => {
                let pit_duration =
                    parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match pit_duration {
                    Ok(duration) => {
                        self.data.pit_duration = duration;
                        planner_context
                            .dispatch(RacePlannerAction::SetFuelStintConfig(self.data.clone()));
                        true
                    }
                    Err(message) => {
                        error!(format!("pit duration parse failed: {}", message).as_str());
                        false
                    }
                }
            }
            OverallFuelStintMessage::UpdateFuelTankSize(value) => match value.parse::<i32>() {
                Ok(tank_size) => {
                    self.data.fuel_tank_size = tank_size;
                    planner_context
                        .dispatch(RacePlannerAction::SetFuelStintConfig(self.data.clone()));
                    true
                }
                Err(e) => {
                    error!(format!("fuel tank size parse failed: {:?}", e).as_str());
                    false
                }
            },
            OverallFuelStintMessage::UpdateTireChangeTime(value) => {
                let tire_change_time =
                    parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match tire_change_time {
                    Ok(duration) => {
                        self.data.tire_change_time = duration;
                        planner_context
                            .dispatch(RacePlannerAction::SetFuelStintConfig(self.data.clone()));
                        true
                    }
                    Err(message) => {
                        error!(format!("tire change time parse failed: {}", message).as_str());
                        false
                    }
                }
            }
            OverallFuelStintMessage::UpdateAddTireTire(value) => {
                self.data.add_tire_time = value;
                planner_context.dispatch(RacePlannerAction::SetFuelStintConfig(self.data.clone()));
                patch_plan(
                    planner_context.data.id,
                    PatchRacePlannerDto {
                        id: planner_context.data.id,
                        title: None,
                        overall_event_config: None,
                        overall_fuel_stint_config: Some(self.data.clone()),
                        fuel_stint_average_times: None,
                        time_of_day_lap_factors: None,
                        per_driver_lap_factors: None,
                        driver_roster: None,
                        schedule_rows: None,
                    },
                );
                false
            }
            OverallFuelStintMessage::OnCreate(data) => {
                self.data = data;
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let pit_duration_props = props! {MaterialTextFieldProps {
            value: format_duration(self.data.pit_duration, DurationFormat::MinSecMilli),
            label: Some("Pit Duration (MM:SS.mmm)".to_string()),
            id: "pit-duration".to_string(),
            on_change: link.callback(|value| OverallFuelStintMessage::UpdatePitDuration(value))
        }};
        let fuel_tank_size_props = props! {MaterialTextFieldProps {
            value: self.data.fuel_tank_size.to_string(),
            label: Some("Fuel Tank Size".to_string()),
            id: "fuel-tank-size".to_string(),
            on_change: link.callback(|value| OverallFuelStintMessage::UpdateFuelTankSize(value))
        }};
        let tire_change_time_props = props! {MaterialTextFieldProps {
            value: format_duration(self.data.tire_change_time, DurationFormat::MinSecMilli),
            label: Some("Tire Change Time (MM:SS.mmm)".to_string()),
            id: "tire-change-time".to_string(),
            on_change: link.callback(|value| OverallFuelStintMessage::UpdateTireChangeTime(value))
        }};

        let add_tire_time_input_ref = self.add_tire_time_input_ref.clone();
        let add_tire_time_callback = link.batch_callback(move |_| {
            let input = add_tire_time_input_ref.cast::<HtmlInputElement>();
            input.map(|input| OverallFuelStintMessage::UpdateAddTireTire(input.checked()))
        });
        html! {
            <div class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Overall Fuel Stint Config" }</div>
                </div>
                <MaterialTextField ..pit_duration_props />
                <MaterialTextField ..fuel_tank_size_props />
                <MaterialTextField ..tire_change_time_props />
                <div class="mdc-form-field">
                  <div class="mdc-checkbox">
                    <input ref={self.add_tire_time_input_ref.clone()}
                            type="checkbox"
                           class="mdc-checkbox__native-control"
                           id="add-tire-time"
                            checked={self.data.add_tire_time}
                            oninput={add_tire_time_callback}/>
                    <div class="mdc-checkbox__background">
                      <svg class="mdc-checkbox__checkmark"
                           viewBox="0 0 24 24">
                        <path class="mdc-checkbox__checkmark-path"
                              fill="none"
                              d="M1.73,12.91 8.1,19.28 22.79,4.59"/>
                      </svg>
                      <div class="mdc-checkbox__mixedmark"></div>
                    </div>
                    <div class="mdc-checkbox__ripple"></div>
                  </div>
                  <label for="add-tire-time">{ "Add Tire Time?" }</label>
                </div>
            </div>
        }
    }
}
