use yew::prelude::*;
use yew::props;
use chrono::Duration;
use crate::md_text_field::{MaterialTextFieldProps, MaterialTextField};
use crate::{format_duration, parse_duration_from_str, DurationFormat};
use yew::services::ConsoleService;
use yew::{ web_sys::HtmlInputElement, NodeRef};
use crate::event_bus::{EventBus, EventBusInput};
use yew::agent::{Dispatcher, Dispatched};
use serde::{Serialize, Deserialize};

pub enum OverallFuelStintMessage {
    UpdatePitDuration(String),
    UpdateFuelTankSize(String),
    UpdateTireChangeTime(String),
    UpdateAddTireTire(bool)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverallFuelStintConfigData {
    #[serde(with = "crate::duration_serde")]
    pub pit_duration: Duration,
    pub fuel_tank_size: i32,
    #[serde(with = "crate::duration_serde")]
    pub tire_change_time: Duration,
    pub add_tire_time: bool
}

impl OverallFuelStintConfigData {
    pub fn new() -> Self {
        Self {
            pit_duration: Duration::zero(),
            fuel_tank_size: 0,
            tire_change_time: Duration::zero(),
            add_tire_time: false
        }
    }
}

impl Clone for OverallFuelStintConfigData {
    fn clone(&self) -> Self {
        OverallFuelStintConfigData {
            pit_duration: self.pit_duration,
            fuel_tank_size: self.fuel_tank_size,
            tire_change_time: self.tire_change_time,
            add_tire_time: self.add_tire_time
        }
    }
}

pub struct OverallFuelStintConfig {
    link: ComponentLink<Self>,
    data: OverallFuelStintConfigData,
    add_tire_time_input_ref: NodeRef,
    event_bus: Dispatcher<EventBus>
}

impl Component for OverallFuelStintConfig {
    type Message = OverallFuelStintMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            data: OverallFuelStintConfigData::new(),
            add_tire_time_input_ref: NodeRef::default(),
            event_bus: EventBus::dispatcher()
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let mut should_render = false;
        match msg {
            OverallFuelStintMessage::UpdatePitDuration(value) => {
                let pit_duration = parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match pit_duration {
                    Ok(duration) => {
                        self.data.pit_duration = duration;
                        should_render = true;
                    }
                    Err(message) => {
                        ConsoleService::error(format!("pit duration parse failed: {}", message).as_str());
                    }
                }
            }
            OverallFuelStintMessage::UpdateFuelTankSize(value) => {                
                match value.parse::<i32>() {
                    Ok(tank_size) => {
                        self.data.fuel_tank_size = tank_size;
                        should_render = true;
                    }
                    Err(e) => {
                        ConsoleService::error(format!("fuel tank size parse failed: {:?}", e).as_str());
                    }
                }
            }
            OverallFuelStintMessage::UpdateTireChangeTime(value) => {
                let tire_change_time = parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match tire_change_time {
                    Ok(duration) => {
                        self.data.tire_change_time = duration;
                        should_render = true;
                    }
                    Err(message) => {
                        ConsoleService::error(format!("tire change time parse failed: {}", message).as_str());
                    }
                }
            }
            OverallFuelStintMessage::UpdateAddTireTire(value) => {
                self.data.add_tire_time = value;
            }
        };
        self.event_bus.send(EventBusInput::OverallFuelStintConfig(self.data.clone()));
        should_render
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {        
        let pit_duration_props = props!(MaterialTextFieldProps {
            value: format_duration(self.data.pit_duration, DurationFormat::MinSecMilli),
            label: Some("Pit Duration (MM:SS.mmm)".to_string()),
            id: "pit-duration".to_string(),
            on_change: self.link.callback(|value| OverallFuelStintMessage::UpdatePitDuration(value))
        });
        let fuel_tank_size_props = props!(MaterialTextFieldProps {
            value: self.data.fuel_tank_size.to_string(),
            label: Some("Fuel Tank Size".to_string()),
            id: "fuel-tank-size".to_string(),
            on_change: self.link.callback(|value| OverallFuelStintMessage::UpdateFuelTankSize(value))
        });
        let tire_change_time_props = props!(MaterialTextFieldProps {
            value: format_duration(self.data.tire_change_time, DurationFormat::MinSecMilli),
            label: Some("Tire Change Time (MM:SS.mmm)".to_string()),
            id: "tire-change-time".to_string(),
            on_change: self.link.callback(|value| OverallFuelStintMessage::UpdateTireChangeTime(value))
        });
        
        let add_tire_time_input_ref = self.add_tire_time_input_ref.clone();
        let add_tire_time_callback = self.link.batch_callback(move |_| {
            let input = add_tire_time_input_ref.cast::<HtmlInputElement>();            
            input.map(|input| OverallFuelStintMessage::UpdateAddTireTire(input.checked()))
        });
        html! {
            <div class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Overall Fuel Stint Config" }</div>
                </div>
                <MaterialTextField with pit_duration_props />
                <MaterialTextField with fuel_tank_size_props />
                <MaterialTextField with tire_change_time_props />
                <div class="mdc-form-field">
                  <div class="mdc-checkbox">
                    <input ref=self.add_tire_time_input_ref.clone()
                            type="checkbox"
                           class="mdc-checkbox__native-control"
                           id="add-tire-time"
                            checked=self.data.add_tire_time
                            oninput=add_tire_time_callback/>
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