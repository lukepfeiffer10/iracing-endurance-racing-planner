use crate::event_bus::{EventBus, EventBusInput, EventBusOutput};
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use crate::planner::{
    format_duration, parse_duration_from_str, DurationFormat, FuelStintAverageTimes, PlannerContext,
};
use chrono::Duration;
use endurance_racing_planner_common::OverallFuelStintConfigData;
use gloo_console::error;
use serde::{Deserialize, Serialize};
use yew::context::ContextHandle;
use yew::prelude::*;
use yew::props;
use yew_agent::{Bridge, Bridged};

#[derive(Serialize, Deserialize, Debug)]
pub struct StandardLapTime {
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
}

impl Clone for StandardLapTime {
    fn clone(&self) -> Self {
        Self {
            lap_time: self.lap_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl StintData {
    pub fn new() -> StintData {
        StintData {
            lap_time: Duration::zero(),
            fuel_per_lap: 0.0,
            lap_count: 0,
            lap_time_with_pit: Duration::zero(),
            track_time: Duration::zero(),
            track_time_with_pit: Duration::zero(),
            fuel_per_stint: 0.0,
        }
    }

    fn update(&mut self, fuel_tank_size: i32, pit_duration: Duration) {
        self.update_lap_time(self.lap_time, pit_duration);
        self.update_fuel_per_lap(self.fuel_per_lap, fuel_tank_size, pit_duration);
    }

    fn update_lap_time(&mut self, lap_time: Duration, pit_duration: Duration) {
        self.lap_time = lap_time;
        self.update_lap_time_with_pit(pit_duration);
        self.update_track_time(pit_duration);
    }

    fn update_fuel_per_lap(
        &mut self,
        fuel_per_lap: f32,
        fuel_tank_size: i32,
        pit_duration: Duration,
    ) {
        self.fuel_per_lap = fuel_per_lap;
        self.lap_count = if fuel_per_lap == 0.0 {
            0
        } else {
            (fuel_tank_size as f32 / fuel_per_lap).floor() as i32
        };
        self.fuel_per_stint = fuel_per_lap * (self.lap_count as f32);
        self.update_lap_time_with_pit(pit_duration);
        self.update_track_time(pit_duration);
    }

    fn update_track_time(&mut self, pit_duration: Duration) {
        self.track_time = self.lap_time * self.lap_count;
        self.track_time_with_pit = self.track_time + pit_duration;
    }

    fn update_lap_time_with_pit(&mut self, pit_duration: Duration) {
        self.lap_time_with_pit = if self.lap_count == 0 {
            Duration::zero()
        } else {
            self.lap_time + pit_duration / self.lap_count
        };
    }
}

#[derive(Debug)]
pub enum StintType {
    Standard,
    FuelSaving,
}

pub enum FuelStintTimesMsg {
    UpdateLapTime(String, StintType),
    UpdateFuelPerLap(String, StintType),
    UpdateFuelConfig(OverallFuelStintConfigData),
    OnCreate(FuelStintAverageTimes),
}

pub struct FuelStintTimes {
    standard_fuel_stint: StintData,
    fuel_saving_stint: StintData,
    fuel_tank_size: i32,
    pit_duration: Duration,
    _producer: Box<dyn Bridge<EventBus>>,
    _context_listener: ContextHandle<PlannerContext>,
}

impl Component for FuelStintTimes {
    type Message = FuelStintTimesMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut event_bus_bridge =
            EventBus::bridge(ctx.link().batch_callback(|message| match message {
                EventBusOutput::SendFuelStintAverageTimes(data) => {
                    data.map(|f| FuelStintTimesMsg::OnCreate(f))
                }
                _ => None,
            }));
        event_bus_bridge.send(EventBusInput::GetFuelStintAverageTimes);

        let (_, planner_context_handle) = ctx
            .link()
            .context::<PlannerContext>(ctx.link().batch_callback(|context: PlannerContext| {
                match &context.data.overall_fuel_stint_config {
                    Some(config) => Some(FuelStintTimesMsg::UpdateFuelConfig(config.clone())),
                    None => None,
                }
            }))
            .expect("No Planner Context Provided");
        Self {
            _producer: event_bus_bridge,
            standard_fuel_stint: StintData::new(),
            fuel_saving_stint: StintData::new(),
            fuel_tank_size: 0,
            pit_duration: Duration::zero(),
            _context_listener: planner_context_handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FuelStintTimesMsg::UpdateLapTime(value, stint_type) => {
                let parsed_lap_time =
                    parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match parsed_lap_time.map(|parsed_lap_time| match stint_type {
                    StintType::Standard => {
                        self.standard_fuel_stint
                            .update_lap_time(parsed_lap_time, self.pit_duration);
                        let fuel_saving_lap_time =
                            (parsed_lap_time.num_milliseconds() as f64) * 1.01;
                        self.fuel_saving_stint.update_lap_time(
                            Duration::milliseconds(fuel_saving_lap_time.floor() as i64),
                            self.pit_duration,
                        );
                        self._producer
                            .send(EventBusInput::StandardLapTime(StandardLapTime {
                                lap_time: parsed_lap_time,
                            }));
                    }
                    StintType::FuelSaving => self
                        .fuel_saving_stint
                        .update_lap_time(parsed_lap_time, self.pit_duration),
                }) {
                    Ok(_) => true,
                    Err(message) => {
                        error!(format!(
                            "{:?} stint lap time parse failed: {}",
                            stint_type, message
                        )
                        .as_str());
                        false
                    }
                }
            }
            FuelStintTimesMsg::UpdateFuelPerLap(value, stint_type) => {
                let parsed_fuel_per_lap = value.parse::<f32>();
                match parsed_fuel_per_lap.map(|parsed_fuel_per_lap| match stint_type {
                    StintType::Standard => self.standard_fuel_stint.update_fuel_per_lap(
                        parsed_fuel_per_lap,
                        self.fuel_tank_size,
                        self.pit_duration,
                    ),
                    StintType::FuelSaving => self.fuel_saving_stint.update_fuel_per_lap(
                        parsed_fuel_per_lap,
                        self.fuel_tank_size,
                        self.pit_duration,
                    ),
                }) {
                    Ok(_) => true,
                    Err(message) => {
                        error!(format!(
                            "{:?} stint fuel per lap parse failed: {}",
                            stint_type, message
                        )
                        .as_str());
                        false
                    }
                }
            }
            FuelStintTimesMsg::UpdateFuelConfig(data) => {
                self.fuel_tank_size = data.fuel_tank_size;
                self.pit_duration = data.pit_duration;
                self.fuel_saving_stint
                    .update(self.fuel_tank_size, self.pit_duration);
                self.standard_fuel_stint
                    .update(self.fuel_tank_size, self.pit_duration);
                true
            }
            FuelStintTimesMsg::OnCreate(data) => {
                self.fuel_saving_stint = data.fuel_saving_stint;
                self.standard_fuel_stint = data.standard_fuel_stint;
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let standard_fuel_lap_time_props = props! {MaterialTextFieldProps {
            value: format_duration(self.standard_fuel_stint.lap_time, DurationFormat::MinSecMilli),
            on_change: link.callback(|value| FuelStintTimesMsg::UpdateLapTime(value, StintType::Standard))
        }};
        let standard_fuel_per_lap_props = props! {MaterialTextFieldProps {
            value: format_fuel_as_string(self.standard_fuel_stint.fuel_per_lap),
            on_change: link.callback(|value| FuelStintTimesMsg::UpdateFuelPerLap(value, StintType::Standard)),
            end_aligned: true
        }};
        let fuel_saving_lap_time_props = props! {MaterialTextFieldProps {
            value: format_duration(self.fuel_saving_stint.lap_time, DurationFormat::MinSecMilli),
            on_change: link.callback(|value| FuelStintTimesMsg::UpdateLapTime(value, StintType::FuelSaving))
        }};
        let fuel_saving_fuel_per_lap_props = props! {MaterialTextFieldProps {
            value: format_fuel_as_string(self.fuel_saving_stint.fuel_per_lap),
            on_change: link.callback(|value| FuelStintTimesMsg::UpdateFuelPerLap(value, StintType::FuelSaving)),
            end_aligned: true
        }};

        html! {
            <div id="fuel-stint-times" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Fuel Stint Average Times" }</div>
                </div>
                <div class="mdc-data-table">
                  <div class="mdc-data-table__table-container">
                    <table class="mdc-data-table__table">
                      <thead>
                        <tr class="mdc-data-table__header-row">
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Stint Type" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Laptime  (MM:SS.mmm)" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Fuel Per Lap" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Lap Count" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Lap W/Pit" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Track Time" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Time W/Pit" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Fuel Per Stint" }</th>
                        </tr>
                      </thead>
                      <tbody class="mdc-data-table__content">
                        <tr class="mdc-data-table__row">
                          <th class="mdc-data-table__cell" scope="row">{ "Standard Fuel Stint" }</th>
                          <td class="mdc-data-table__cell">
                            <MaterialTextField ..standard_fuel_lap_time_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                            <MaterialTextField ..standard_fuel_per_lap_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.standard_fuel_stint.lap_count }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.standard_fuel_stint.lap_time_with_pit, DurationFormat::MinSecMilli) }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.standard_fuel_stint.track_time, DurationFormat::HourMinSec) }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.standard_fuel_stint.track_time_with_pit, DurationFormat::HourMinSec) }</td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_fuel_as_string(self.standard_fuel_stint.fuel_per_stint) }</td>
                        </tr>
                        <tr class="mdc-data-table__row">
                          <th class="mdc-data-table__cell" scope="row">{ "Fuel Saving Stint" }</th>
                          <td class="mdc-data-table__cell">
                            <MaterialTextField ..fuel_saving_lap_time_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                            <MaterialTextField ..fuel_saving_fuel_per_lap_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.fuel_saving_stint.lap_count }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.fuel_saving_stint.lap_time_with_pit, DurationFormat::MinSecMilli) }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.fuel_saving_stint.track_time, DurationFormat::HourMinSec) }</td>
                          <td class="mdc-data-table__cell">{ format_duration(self.fuel_saving_stint.track_time_with_pit, DurationFormat::HourMinSec) }</td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_fuel_as_string(self.fuel_saving_stint.fuel_per_stint) }</td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
            </div>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self._producer.send(EventBusInput::PutFuelStintAverageTimes(
            FuelStintAverageTimes {
                fuel_saving_stint: self.fuel_saving_stint.clone(),
                standard_fuel_stint: self.standard_fuel_stint.clone(),
            },
        ))
    }
}

fn format_fuel_as_string(fuel: f32) -> String {
    format!("{:.2}", fuel)
}
