use yew::prelude::*;
use yew::props;
use chrono::Duration;
use crate::{format_duration, parse_duration_from_str, DurationFormat};
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use yew::services::ConsoleService;
use crate::event_bus::{EventBus, EventBusOutput, EventBusInput};
use serde::{Serialize, Deserialize};
use crate::overview::overall_fuel_stint_config::OverallFuelStintConfigData;

#[derive(Serialize, Deserialize, Debug)]
pub struct StandardLapTime {
    #[serde(with = "crate::duration_serde")]
    pub lap_time: Duration,
}

impl Clone for StandardLapTime {
    fn clone(&self) -> Self {
        Self {
            lap_time: self.lap_time
        }
    }
}

struct StintData {
    lap_time: Duration,
    fuel_per_lap: f32,
    lap_count: i32,
    lap_time_with_pit: Duration,
    track_time: Duration,
    track_time_with_pit: Duration,
    fuel_per_stint: f32,
    fuel_stint_config: OverallFuelStintConfigData
}

impl StintData {
    fn new() -> StintData {
        StintData {
            lap_time: Duration::zero(),
            fuel_per_lap: 0.0,
            lap_count: 0,
            lap_time_with_pit: Duration::zero(),
            track_time: Duration::zero(),
            track_time_with_pit: Duration::zero(),
            fuel_per_stint: 0.0,
            fuel_stint_config: OverallFuelStintConfigData::new()
        }
    }
    
    fn update(&mut self) {
        self.update_lap_time(self.lap_time);
        self.update_fuel_per_lap(self.fuel_per_lap);
    }
    
    fn update_lap_time(&mut self, lap_time: Duration) {
        self.lap_time = lap_time;
        self.update_lap_time_with_pit();
        self.update_track_time();
    }
    
    fn update_fuel_per_lap(&mut self, fuel_per_lap: f32) {
        self.fuel_per_lap = fuel_per_lap;
        self.lap_count = if fuel_per_lap == 0.0 { 
             0
        } else {
            (self.fuel_stint_config.fuel_tank_size as f32 / fuel_per_lap).floor() as i32
        };
        self.fuel_per_stint = fuel_per_lap * (self.lap_count as f32);
        self.update_lap_time_with_pit();
        self.update_track_time();
    }
    
    fn update_track_time(&mut self) {
        self.track_time = self.lap_time * self.lap_count;
        self.track_time_with_pit = self.track_time + self.fuel_stint_config.pit_duration;        
    }
    
    fn update_lap_time_with_pit(&mut self) {
        self.lap_time_with_pit = if self.lap_count == 0 {
            Duration::zero()
        } else {
            self.lap_time + self.fuel_stint_config.pit_duration / self.lap_count
        };
    }
}

#[derive(Debug)]
pub enum StintType {
    Standard,
    FuelSaving
}

pub enum FuelStintTimesMsg {
    UpdateLapTime(String, StintType),
    UpdateFuelPerLap(String, StintType),
    Update(OverallFuelStintConfigData)
}

pub struct FuelStintTimes {
    link: ComponentLink<Self>,
    standard_fuel_stint: StintData,
    fuel_saving_stint: StintData,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for FuelStintTimes {
    type Message = FuelStintTimesMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            _producer: EventBus::bridge(link.batch_callback(|message| {
                match message {
                    EventBusOutput::OverallFuelStintConfig(data) => {
                        Some(FuelStintTimesMsg::Update(data))
                    },
                    _ => None
                }
            })),
            link,
            standard_fuel_stint: StintData::new(),
            fuel_saving_stint: StintData::new(),            
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg { 
            FuelStintTimesMsg::UpdateLapTime(value, stint_type) => {
                let parsed_lap_time = parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match parsed_lap_time.map(|parsed_lap_time| {
                    match stint_type {
                        StintType::Standard => {
                            self.standard_fuel_stint.update_lap_time(parsed_lap_time);
                            let fuel_saving_lap_time = (parsed_lap_time.num_milliseconds() as f64) * 1.01;
                            self.fuel_saving_stint.update_lap_time(Duration::milliseconds(fuel_saving_lap_time.floor() as i64));
                            self._producer.send(EventBusInput::StandardLapTime(StandardLapTime {
                                lap_time: parsed_lap_time
                            }));
                        }
                        StintType::FuelSaving => self.fuel_saving_stint.update_lap_time(parsed_lap_time)
                    }
                }) {
                    Ok(_) => true,
                    Err(message) => {
                        ConsoleService::error(format!("{:?} stint lap time parse failed: {}", stint_type, message).as_str());
                        false
                    }
                }
            },
            FuelStintTimesMsg::UpdateFuelPerLap(value, stint_type) => {
                let parsed_fuel_per_lap = value.parse::<f32>();
                match parsed_fuel_per_lap.map(|parsed_fuel_per_lap| {
                    match stint_type {
                        StintType::Standard => self.standard_fuel_stint.update_fuel_per_lap(parsed_fuel_per_lap),
                        StintType::FuelSaving => self.fuel_saving_stint.update_fuel_per_lap(parsed_fuel_per_lap)
                    }
                }) {
                    Ok(_) => true,
                    Err(message) => {
                        ConsoleService::error(format!("{:?} stint fuel per lap parse failed: {}", stint_type, message).as_str());
                        false
                    }
                }
            },
            FuelStintTimesMsg::Update(data) => {
                self.fuel_saving_stint.fuel_stint_config = data.clone();
                self.standard_fuel_stint.fuel_stint_config = data.clone();
                self.fuel_saving_stint.update();
                self.standard_fuel_stint.update();
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let standard_fuel_lap_time_props = props!(MaterialTextFieldProps {
            value: format_duration(self.standard_fuel_stint.lap_time, DurationFormat::MinSecMilli),
            on_change: self.link.callback(|value| FuelStintTimesMsg::UpdateLapTime(value, StintType::Standard))
        });
        let standard_fuel_per_lap_props = props!(MaterialTextFieldProps {
            value: format_fuel_as_string(self.standard_fuel_stint.fuel_per_lap),
            on_change: self.link.callback(|value| FuelStintTimesMsg::UpdateFuelPerLap(value, StintType::Standard)),
            end_aligned: true
        });
        let fuel_saving_lap_time_props = props!(MaterialTextFieldProps {
            value: format_duration(self.fuel_saving_stint.lap_time, DurationFormat::MinSecMilli),
            on_change: self.link.callback(|value| FuelStintTimesMsg::UpdateLapTime(value, StintType::FuelSaving))
        });
        let fuel_saving_fuel_per_lap_props = props!(MaterialTextFieldProps {
            value: format_fuel_as_string(self.fuel_saving_stint.fuel_per_lap),
            on_change: self.link.callback(|value| FuelStintTimesMsg::UpdateFuelPerLap(value, StintType::FuelSaving)),
            end_aligned: true
        });
        
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
                            <MaterialTextField with standard_fuel_lap_time_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                            <MaterialTextField with standard_fuel_per_lap_props />
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
                            <MaterialTextField with fuel_saving_lap_time_props />
                          </td>
                          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                            <MaterialTextField with fuel_saving_fuel_per_lap_props />
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
}

fn format_fuel_as_string(fuel: f32) -> String {
    format!("{:.2}", fuel)
}