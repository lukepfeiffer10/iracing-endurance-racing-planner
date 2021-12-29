use std::fmt::{Display, Formatter};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Serialize, Deserialize};
use yew::prelude::*;
use yew::{Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::*;
use crate::{Duration, DurationFormat, EventBus, EventConfigData, format_duration, FuelStintAverageTimes, AppRoutes, OverallFuelStintConfigData, Driver};
use crate::bindings::{enable_selects};
use crate::event_bus::{EventBusInput, EventBusOutput};

#[derive(Debug)]
enum StintType {
    FuelSavingNoTires,
    FuelSavingWithTires,
    StandardNoTires,
    StandardWithTires,
}

impl Display for StintType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StintType::FuelSavingNoTires => write!(f, "fs no tires"),
            StintType::FuelSavingWithTires => write!(f, "fs w/ tires"),
            StintType::StandardNoTires => write!(f, "std no tires"),
            StintType::StandardWithTires => write!(f, "std w/ tires")
        }
    }
}

#[derive(Debug)]
struct ScheduleDataRow {
    stint_type: StintType,
    fuel_stint_number: i32,
    utc_start: DateTime<Utc>,
    utc_end: DateTime<Utc>,
    tod_start: NaiveDateTime,
    tod_end: NaiveDateTime,
    actual_end: DateTime<Utc>,
    duration_delta: Duration,
    damage_modifier: Duration,
    calculated_laps: i32,
    actual_laps: i32,
    driver_name: String,
    availability: String,
    stint_number: i32,
    stint_preference: i32,
    factor: f32,
    local_start: NaiveDateTime,
    local_end: NaiveDateTime,
}

impl ScheduleDataRow {
    fn new(config: &EventConfigData, fuel_stint_times: &FuelStintAverageTimes) -> Self {
        let stint_duration = fuel_stint_times.fuel_saving_stint.track_time_with_pit;
        Self {
            stint_type: StintType::FuelSavingNoTires,
            fuel_stint_number: 1,
            utc_start: config.race_start_utc,
            utc_end: config.race_start_utc + stint_duration,
            tod_start: config.race_start_tod,
            tod_end: config.race_start_tod + stint_duration,
            actual_end: config.race_start_utc + stint_duration,
            duration_delta: Duration::zero(),
            damage_modifier: Duration::zero(),
            calculated_laps: fuel_stint_times.fuel_saving_stint.lap_count,
            actual_laps: fuel_stint_times.fuel_saving_stint.lap_count,
            driver_name: "".to_string(),
            availability: "".to_string(),
            stint_number: 1,
            stint_preference: 0,
            factor: 0.0,
            local_start: config.race_start_utc.naive_local(),
            local_end: config.race_start_utc.naive_local() + stint_duration
        }
    }
    
    fn from_previous(previous_row: &ScheduleDataRow, stint_type: StintType, fuel_stint_times: &FuelStintAverageTimes, race_end_utc: DateTime<Utc>, tire_change_time: Duration) -> Self {
        let utc_start = previous_row.utc_end.clone();
        let tod_start = previous_row.tod_end.clone();
        let fuel_stint_data = match stint_type {
            StintType::FuelSavingNoTires | StintType::FuelSavingWithTires => &fuel_stint_times.fuel_saving_stint,
            StintType::StandardNoTires | StintType::StandardWithTires => &fuel_stint_times.standard_fuel_stint
        };
        let track_time_with_pit = match stint_type { 
            StintType::FuelSavingWithTires | StintType::StandardWithTires => fuel_stint_data.track_time_with_pit + tire_change_time,
            StintType::FuelSavingNoTires | StintType::StandardNoTires => fuel_stint_data.track_time_with_pit
        };
        let (stint_duration, calculated_laps) = if utc_start + track_time_with_pit > race_end_utc {
            let stint_duration = race_end_utc - utc_start;
            let calculated_laps = (stint_duration.num_milliseconds() as f64 / fuel_stint_data.lap_time.num_milliseconds() as f64).ceil();
            (stint_duration, calculated_laps as i32)
        } else {
            (track_time_with_pit, fuel_stint_data.lap_count)
        };
        Self {
            stint_type,
            fuel_stint_number: previous_row.fuel_stint_number + 1,
            utc_start,
            utc_end: utc_start + stint_duration,
            tod_start,
            tod_end: tod_start + stint_duration,
            actual_end: utc_start + stint_duration,
            duration_delta: Duration::zero(),
            damage_modifier: previous_row.damage_modifier.clone(),
            calculated_laps,
            actual_laps: calculated_laps,
            driver_name: "".to_string(),
            availability: "".to_string(),
            stint_number: if previous_row.driver_name == "" {
                previous_row.stint_number + 1
            } else {
                1
            },
            stint_preference: 0,
            factor: 0.0,
            local_start: utc_start.naive_local(),
            local_end: utc_start.naive_local() + stint_duration
        }
    }
    
    fn get_view(&self, _link: &ComponentLink<FuelStintSchedule>, index: usize) -> Html {
        let time_format = "%l:%M %p";        
        html! {
            <tr class="mdc-data-table__row">
                <td class="mdc-data-table__cell mdc-data-table__cell--checkbox">
                    <div class="mdc-touch-target-wrapper">
                        <div class="mdc-checkbox mdc-checkbox--touch mdc-data-table__row-checkbox">
                            <input type="checkbox" class="mdc-checkbox__native-control" id="checkbox-1"/>
                            <div class="mdc-checkbox__background">
                                <svg class="mdc-checkbox__checkmark" viewBox="0 0 24 24">
                                    <path class="mdc-checkbox__checkmark-path" fill="none" d="M1.73,12.91 8.1,19.28 22.79,4.59"/>
                                </svg>
                                <div class="mdc-checkbox__mixedmark"></div>
                            </div>
                            <div class="mdc-checkbox__ripple"></div>
                        </div>
                    </div>
                </td>
                <td class="mdc-data-table__cell">
                    <div class="mdc-select mdc-select--filled mdc-select--no-label select-width">
                        <div class="mdc-select__anchor" 
                            role="button"
                            aria-haspopup="listbox"
                            aria-expanded="false"
                            aria-labelledby={ format!("stint-type-selected-text-{}", index) }>
            
                            <span class="mdc-select__ripple"></span>
                            <span class="mdc-select__selected-text-container">
                                <span id={ format!("stint-type-selected-text-{}", index) } class="mdc-select__selected-text"></span>
                            </span>
                            <span class="mdc-select__dropdown-icon">
                                <svg class="mdc-select__dropdown-icon-graphic" viewBox="7 10 10 5" focusable="false">
                                    <polygon
                                        class="mdc-select__dropdown-icon-inactive"
                                        stroke="none"
                                        fill-rule="evenodd"
                                        points="7 10 12 15 17 10">
                                    </polygon>
                                    <polygon
                                        class="mdc-select__dropdown-icon-active"
                                        stroke="none"
                                        fill-rule="evenodd"
                                        points="7 15 12 10 17 15">
                                    </polygon>
                                </svg>
                            </span>
                            <span class="mdc-line-ripple"></span>
                        </div>                    
                        <div class="mdc-select__menu mdc-menu mdc-menu-surface mdc-menu-surface--fixed select-width">
                            <ul class="mdc-deprecated-list" role="menu" aria-hidden="true" aria-orientation="vertical" tabindex="-1">
                                <li class="mdc-deprecated-list-item" aria-selected="" data-value=StintType::FuelSavingNoTires.to_string() role="option">
                                    <span class="mdc-deprecated-list-item__ripple"></span>
                                    <span class="mdc-deprecated-list-item__text">
                                        { StintType::FuelSavingNoTires }
                                    </span>
                                </li>
                                <li class="mdc-deprecated-list-item" aria-selected="false" data-value=StintType::FuelSavingWithTires.to_string() role="option">
                                    <span class="mdc-deprecated-list-item__ripple"></span>
                                    <span class="mdc-deprecated-list-item__text">
                                        { StintType::FuelSavingWithTires }
                                    </span>
                                </li>
                                <li class="mdc-deprecated-list-item" aria-selected="false" data-value=StintType::StandardNoTires.to_string() aria-disabled="true" role="option">
                                    <span class="mdc-deprecated-list-item__ripple"></span>
                                    <span class="mdc-deprecated-list-item__text">
                                        { StintType::StandardNoTires }
                                    </span>
                                </li>
                                <li class="mdc-deprecated-list-item" aria-selected="false" data-value=StintType::StandardWithTires.to_string() role="option">
                                    <span class="mdc-deprecated-list-item__ripple"></span>
                                    <span class="mdc-deprecated-list-item__text">
                                        { StintType::StandardWithTires }
                                    </span>
                                </li>
                            </ul>
                        </div>
                    </div>
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.fuel_stint_number }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.utc_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.utc_end.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.tod_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.tod_end.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.actual_end.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_duration(self.duration_delta, DurationFormat::HourMinSec) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_duration(self.damage_modifier, DurationFormat::HourMinSec) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.calculated_laps }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.actual_laps }</td>
                <td class="mdc-data-table__cell">{ self.driver_name.clone() }</td>
                <td class="mdc-data-table__cell">{ self.availability.clone() }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_number }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_preference }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.factor }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.local_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.local_end.format(time_format) }</td>
            </tr>
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleRelatedData {
    pub overall_event_config: Option<EventConfigData>,
    pub fuel_stint_times: Option<FuelStintAverageTimes>,
    pub overall_fuel_stint_config: OverallFuelStintConfigData,
    pub drivers: Vec<Driver>,
}

pub enum FuelStintScheduleMsg {
    OnCreate(ScheduleRelatedData),
}

pub struct FuelStintSchedule {
    link: ComponentLink<Self>,
    _producer: Box<dyn Bridge<EventBus>>,
    schedule_rows: Vec<ScheduleDataRow>,
    overall_event_config: Option<EventConfigData>,
    fuel_stint_times: Option<FuelStintAverageTimes>,
    overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
    drivers: Option<Vec<Driver>>,
}

impl FuelStintSchedule {
    fn create_schedule(&mut self) -> ShouldRender {
        if self.overall_event_config.is_some() && 
            self.fuel_stint_times.is_some() && 
            self.overall_fuel_stint_config.is_some() && 
            self.schedule_rows.is_empty() {
            
            let event_config = self.overall_event_config.as_ref().unwrap();
            let fuel_stint_times = self.fuel_stint_times.as_ref().unwrap();
            let fuel_stint_config = self.overall_fuel_stint_config.as_ref().unwrap();
            
            if event_config.race_start_utc == event_config.race_end_utc {
                return false;
            }
            
            let schedule_row = ScheduleDataRow::new(event_config, fuel_stint_times);
            let mut is_schedule_complete = schedule_row.utc_end >= event_config.race_end_utc;
            self.schedule_rows.push(schedule_row);

            while !is_schedule_complete {
                let previous_row = self.schedule_rows.last().unwrap();
                let schedule_row = ScheduleDataRow::from_previous(previous_row, 
                                                                  StintType::FuelSavingWithTires, 
                                                                  fuel_stint_times, 
                                                                  event_config.race_end_utc, 
                                                                  fuel_stint_config.tire_change_time);
                is_schedule_complete = schedule_row.utc_end >= event_config.race_end_utc;
                self.schedule_rows.push(schedule_row);
            }
            true
        } else {
            false
        }
    }
}

impl Component for FuelStintSchedule {
    type Message = FuelStintScheduleMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut event_bus_bridge = EventBus::bridge(link.batch_callback(|message| {
            match message {
                EventBusOutput::SendScheduleRelatedData(data) => {
                    Some(FuelStintScheduleMsg::OnCreate(data))
                }
                _ => None
            }
        }));
        event_bus_bridge.send(EventBusInput::GetScheduleRelatedData);
        Self {
            link,
            _producer: event_bus_bridge,
            schedule_rows: vec![],
            overall_event_config: None,
            fuel_stint_times: None,
            overall_fuel_stint_config: None,
            drivers: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            FuelStintScheduleMsg::OnCreate(data) => {
                self.overall_event_config = data.overall_event_config;
                self.fuel_stint_times = data.fuel_stint_times;
                self.overall_fuel_stint_config = Some(data.overall_fuel_stint_config);
                self.drivers = Some(data.drivers);
                self.create_schedule()
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html!{
            <div id="fuel-stint-schedule" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Fuel Stint Schedule" }</div>
                </div>
                {
                    if self.schedule_rows.len() == 0 {
                        html!{
                            <p>{ "Complete the event config, fuel stint average times, and fuel stint config on the "}<RouterAnchor<AppRoutes> route=AppRoutes::Overview>{"Overview"}</RouterAnchor<AppRoutes>>{" page"}</p>
                        }
                    } else {
                        html! {
                            <div class="mdc-data-table">
                              <div class="mdc-data-table__table-container">
                                <table class="mdc-data-table__table">
                                  <thead>
                                    <tr class="mdc-data-table__header-row">
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--checkbox" role="columnheader" scope="col">
                                        <div class="mdc-checkbox mdc-data-table__header-row-checkbox mdc-checkbox--selected">
                                          <input type="checkbox" class="mdc-checkbox__native-control" aria-label="Toggle all rows"/>
                                          <div class="mdc-checkbox__background">
                                            <svg class="mdc-checkbox__checkmark" viewBox="0 0 24 24">
                                              <path class="mdc-checkbox__checkmark-path" fill="none" d="M1.73,12.91 8.1,19.28 22.79,4.59" />
                                            </svg>
                                            <div class="mdc-checkbox__mixedmark"></div>
                                          </div>
                                          <div class="mdc-checkbox__ripple"></div>
                                        </div>
                                      </th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Stint Type" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Fuel Stint" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "UTC Start" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "UTC End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "ToD Start" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "ToD End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Actual End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Duration Delta" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Damage Modifier" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Calc Laps" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Actual Laps" }</th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Driver" }</th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Availability" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Stint Num" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Stint Pref" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Factor" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Local Start" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Local End" }</th>
                                    </tr>
                                  </thead>
                                  <tbody class="mdc-data-table__content">
                                    {
                                        self.schedule_rows
                                            .iter()
                                            .enumerate()
                                            .map(|(index, row)| row.get_view(&self.link, index))
                                            .collect::<Vec<_>>()
                                    }
                                  </tbody>
                                </table>
                              </div>
                            </div>
                        }
                    }
                }
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if !self.schedule_rows.is_empty() {
            enable_selects(".mdc-select");
        }
    }
}