use std::fmt::{Display, Formatter};
use std::str::FromStr;
use chrono::{DateTime, NaiveDateTime, NaiveTime, Timelike, Utc};
use serde::{Serialize, Deserialize};
use yew::prelude::*;
use yew::{Component, ComponentLink, Html, props, ShouldRender};
use yew::services::ConsoleService;
use yew::web_sys::{Element};
use yew_router::prelude::*;
use yew_mdc::components::{Select, SelectItem};
use yew_mdc::components::select::SelectChangeEventData;
use yew_mdc::mdc_sys::MDCDataTable;
use crate::{Duration, DurationFormat, EventBus, EventConfigData, format_duration, FuelStintAverageTimes, AppRoutes, OverallFuelStintConfigData, Driver};
use crate::event_bus::{EventBusInput, EventBusOutput};
use crate::md_text_field::{MaterialTextFieldProps, MaterialTextField};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum StintType {
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

impl FromStr for StintType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s { 
            "fs no tires" => Ok(StintType::FuelSavingNoTires),
            "fs w/ tires" => Ok(StintType::FuelSavingWithTires),
            "std no tires" => Ok(StintType::StandardNoTires),
            "std w/ tires" => Ok(StintType::StandardWithTires),
            _ => Err(format!("{} cannot be mapped to a valid StintType", s))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleDataRow {
    stint_type: StintType,
    fuel_stint_number: i32,
    utc_start: DateTime<Utc>,
    utc_end: DateTime<Utc>,
    tod_start: NaiveDateTime,
    tod_end: NaiveDateTime,
    actual_end: DateTime<Utc>,
    #[serde(with = "crate::duration_serde")]
    duration_delta: Duration,
    #[serde(with = "crate::duration_serde")]
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
    
    fn from_previous(previous_row: &ScheduleDataRow, 
                     stint_type: StintType, 
                     fuel_stint_times: &FuelStintAverageTimes, 
                     race_end_utc: DateTime<Utc>, 
                     tire_change_time: Duration) -> Self {
        
        let utc_start = previous_row.utc_end.clone();
        let tod_start = previous_row.tod_end.clone();
        
        let (stint_duration, calculated_laps) = calculate_stint_duration_and_laps(utc_start, &stint_type, fuel_stint_times, race_end_utc, tire_change_time);
        
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
            stint_number: 1,
            stint_preference: 0,
            factor: 0.0,
            local_start: utc_start.naive_local(),
            local_end: utc_start.naive_local() + stint_duration
        }
    }
    
    fn update(&mut self,
              utc_start: DateTime<Utc>,
              tod_start: NaiveDateTime,
              previous_row_driver_name: String,
              previous_row_stint_number: i32,
              fuel_stint_times: &FuelStintAverageTimes,
              race_end_utc: DateTime<Utc>,
              tire_change_time: Duration) {
        
        let (stint_duration, calculated_laps) = calculate_stint_duration_and_laps(utc_start,
                                                                                  &self.stint_type,
                                                                                  fuel_stint_times,
                                                                                  race_end_utc,
                                                                                  tire_change_time);
        self.utc_start = utc_start;
        self.utc_end = self.utc_start + stint_duration;
        self.actual_end = self.utc_end;
        self.tod_start = tod_start;
        self.tod_end = self.tod_start + stint_duration;
        self.calculated_laps = calculated_laps;
        self.actual_laps = calculated_laps;
        self.duration_delta = self.actual_end - self.utc_end;

        if (self.driver_name != "" && previous_row_driver_name != "") && previous_row_driver_name == self.driver_name {
            self.stint_number = previous_row_stint_number + 1;
        } else {
            self.stint_number = 1;
        }
    }
    
    fn get_view(&self, link: &ComponentLink<FuelStintSchedule>, index: usize, drivers: Option<&Vec<Driver>>) -> Html {
        let time_format = "%l:%M %p"; // (H)H:MM AM|PM
        let actual_end_on_change = link.batch_callback(move |value: String| {
            let value_as_str = value.as_str();
            let actual_end_time = NaiveTime::parse_from_str(value_as_str, time_format)
                .or_else(|_| NaiveTime::parse_from_str(value_as_str,"%R")) //HH:MM
                .or_else(|_| NaiveTime::parse_from_str(value_as_str, "%T")) //HH:MM:SS
                .or_else(|_| NaiveTime::parse_from_str(value_as_str, "%l:%M:%S %p")); // (H)H:MM:SS AM|PM
            
            match actual_end_time {
                Ok(value) => Some(FuelStintScheduleMsg::UpdateActualEndTime(value, index)),
                Err(e) => {
                    ConsoleService::error(format!("The actual end time could not be parsed: {}", e).as_str());
                    None
                }
            }
        });
        let actual_end_props = props!(MaterialTextFieldProps {
            value: self.actual_end.format(time_format).to_string(),
            end_aligned: true,
            on_change: actual_end_on_change
        });
        let damage_modifier_props = props!(MaterialTextFieldProps {
            value: format_duration(self.damage_modifier, DurationFormat::HourMinSec),
            end_aligned: true,
        });
        let actual_laps_props = props!(MaterialTextFieldProps {
            value: self.actual_laps.to_string(),
            end_aligned: true,
        });
        let stint_type_onchange = link.batch_callback(move |data: SelectChangeEventData| {
            let stint_type = StintType::from_str(data.value.as_str());
            match stint_type { 
                Ok(stint_type) => Some(FuelStintScheduleMsg::UpdateFuelStintType(stint_type, index)),
                Err(s) => {
                    ConsoleService::error(s.as_str());
                    None
                }
            }
        });
        let driver_name_on_change = link.callback(move |data:SelectChangeEventData| {
            FuelStintScheduleMsg::UpdateDriver(data, index)
        });
        
        let row_id = format!("row-{}", index);
        html! {
            <tr data-row-id=row_id.clone() class="mdc-data-table__row">
                <td class="mdc-data-table__cell mdc-data-table__cell--checkbox">
                    <div class="mdc-touch-target-wrapper">
                        <div class="mdc-checkbox mdc-checkbox--touch mdc-data-table__row-checkbox">
                            <input type="checkbox" class="mdc-checkbox__native-control" aria-labelledby=row_id.clone() />
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
                    <Select id=format!("stint-type-{}", index) 
                        select_width_class="select-width"
                        fixed_position=true
                        selected_value=Some(self.stint_type.to_string())
                        onchange=stint_type_onchange>
                        <SelectItem text=StintType::FuelSavingNoTires.to_string() 
                            value=StintType::FuelSavingNoTires.to_string() />
                        <SelectItem text=StintType::FuelSavingWithTires.to_string() 
                            value=StintType::FuelSavingWithTires.to_string() />
                        <SelectItem text=StintType::StandardNoTires.to_string() 
                            value=StintType::StandardNoTires.to_string() />
                        <SelectItem text=StintType::StandardWithTires.to_string() 
                            value=StintType::StandardWithTires.to_string() />
                    </Select>                        
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.fuel_stint_number }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.utc_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.utc_end.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.tod_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.tod_end.format(time_format) }</td>
                <td class="mdc-data-table__cell">
                    <MaterialTextField with actual_end_props />
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_duration(self.duration_delta, DurationFormat::HourMinSec) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                    <MaterialTextField with damage_modifier_props />
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.calculated_laps }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                    <MaterialTextField with actual_laps_props />
                </td>
                <td class="mdc-data-table__cell">
                    <Select id=format!("driver-name-{}", index)
                        select_width_class="select-width"
                        fixed_position=true
                        selected_value=Some(self.driver_name.to_string())
                        onchange=driver_name_on_change>
                        <SelectItem text="" value="" />
                        {
                            drivers
                                .map_or(html! {},
                                    |drivers| drivers
                                        .iter()
                                        .map(get_driver_select_view)
                                        .collect::<Html>()
                                )
                        }
                    </Select>                        
                </td>
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

fn calculate_stint_duration_and_laps(stint_utc_start: DateTime<Utc>, stint_type: &StintType, fuel_stint_times: &FuelStintAverageTimes, race_end_utc: DateTime<Utc>, tire_change_time: Duration) -> (Duration, i32) {
    let fuel_stint_data = match stint_type {
        StintType::FuelSavingNoTires | StintType::FuelSavingWithTires => &fuel_stint_times.fuel_saving_stint,
        StintType::StandardNoTires | StintType::StandardWithTires => &fuel_stint_times.standard_fuel_stint
    };
    let track_time_with_pit = match stint_type {
        StintType::FuelSavingWithTires | StintType::StandardWithTires => fuel_stint_data.track_time_with_pit + tire_change_time,
        StintType::FuelSavingNoTires | StintType::StandardNoTires => fuel_stint_data.track_time_with_pit
    };
    
    if stint_utc_start + track_time_with_pit > race_end_utc {
        let stint_duration = race_end_utc - stint_utc_start;
        let calculated_laps = (stint_duration.num_milliseconds() as f64 / fuel_stint_data.lap_time.num_milliseconds() as f64).ceil() as i32;
        (stint_duration, calculated_laps)
    } else {
        (track_time_with_pit, fuel_stint_data.lap_count)
    }
}

fn get_driver_select_view(driver: &Driver) -> Html {
    html!{
        <SelectItem text=driver.name.clone() value=driver.name.clone() />
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
    OnCreate(Option<Vec<ScheduleDataRow>>, ScheduleRelatedData),
    UpdateFuelStintType(StintType, usize),
    UpdateActualEndTime(NaiveTime, usize),
    UpdateDriver(SelectChangeEventData, usize)
}

pub struct FuelStintSchedule {
    link: ComponentLink<Self>,
    _producer: Box<dyn Bridge<EventBus>>,
    schedule_rows: Vec<ScheduleDataRow>,
    overall_event_config: Option<EventConfigData>,
    fuel_stint_times: Option<FuelStintAverageTimes>,
    overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
    drivers: Option<Vec<Driver>>,
    mdc_data_table_node_ref: NodeRef,
    data_table: Option<MDCDataTable>
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
            
            if event_config.race_start_utc == event_config.race_end_utc || 
                fuel_stint_times.standard_fuel_stint.track_time == Duration::zero() {
                
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
    
    fn update_schedule(&mut self, update_row_index: usize) {
        let event_config = self.overall_event_config.as_ref().unwrap();
        let fuel_stint_times = self.fuel_stint_times.as_ref().unwrap();
        let fuel_stint_config = self.overall_fuel_stint_config.as_ref().unwrap();
        
        let initial_schedule_length = self.schedule_rows.len();
        let updated_row = &self.schedule_rows[update_row_index];
        let mut is_schedule_complete = updated_row.utc_end >= event_config.race_end_utc;
        let mut next_row_index = update_row_index;
        while !is_schedule_complete {
            next_row_index += 1;
            if next_row_index == self.schedule_rows.len() {
                let previous_row = self.schedule_rows.last().unwrap();
                let schedule_row = ScheduleDataRow::from_previous(previous_row,
                                                                  StintType::FuelSavingWithTires,
                                                                  fuel_stint_times,
                                                                  event_config.race_end_utc,
                                                                  fuel_stint_config.tire_change_time);
                
                is_schedule_complete = schedule_row.utc_end >= event_config.race_end_utc;
                self.schedule_rows.push(schedule_row);
            } else {
                let previous_row = &self.schedule_rows[next_row_index - 1];
                let previous_row_actual_end = previous_row.actual_end;
                let previous_row_driver_name = previous_row.driver_name.clone();
                let previous_row_stint_number = previous_row.stint_number;
                let previous_row_tod_end = previous_row_actual_end.naive_utc() + event_config.tod_offset;
                
                let next_row = &mut self.schedule_rows[next_row_index];
                next_row.update(previous_row_actual_end, 
                                previous_row_tod_end, 
                                previous_row_driver_name, 
                                previous_row_stint_number,
                                fuel_stint_times,
                                event_config.race_end_utc,
                                fuel_stint_config.tire_change_time);                
                
                is_schedule_complete = next_row.utc_end >= event_config.race_end_utc;
            }
        }

        while self.schedule_rows.len() > next_row_index + 1 {
            self.schedule_rows.pop();
        }
        
        if initial_schedule_length != self.schedule_rows.len() {
            if let Some(data_table) = self.data_table.take() {
                data_table.destroy();
            }
        }
    }
}

impl Component for FuelStintSchedule {
    type Message = FuelStintScheduleMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut event_bus_bridge = EventBus::bridge(link.batch_callback(|message| {
            match message {
                EventBusOutput::SendScheduleAndRelatedData(schedule_rows,data) => {
                    Some(FuelStintScheduleMsg::OnCreate(schedule_rows, data))
                }
                _ => None
            }
        }));
        event_bus_bridge.send(EventBusInput::GetScheduleAndRelatedData);
        Self {
            link,
            _producer: event_bus_bridge,
            schedule_rows: vec![],
            overall_event_config: None,
            fuel_stint_times: None,
            overall_fuel_stint_config: None,
            drivers: None,
            mdc_data_table_node_ref: NodeRef::default(),
            data_table: None
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            FuelStintScheduleMsg::OnCreate(schedule_rows, data) => {
                self.overall_event_config = data.overall_event_config;
                self.fuel_stint_times = data.fuel_stint_times;
                self.overall_fuel_stint_config = Some(data.overall_fuel_stint_config);
                self.drivers = Some(data.drivers);
                
                match schedule_rows { 
                    Some(rows) => {
                        if self.overall_event_config.is_some() &&
                            self.fuel_stint_times.is_some() &&
                            self.overall_fuel_stint_config.is_some() {
                            let event_config = self.overall_event_config.as_ref().unwrap();
                            let fuel_stint_times = self.fuel_stint_times.as_ref().unwrap();
                            let fuel_stint_config = self.overall_fuel_stint_config.as_ref().unwrap();

                            if event_config.race_start_utc == event_config.race_end_utc ||
                                fuel_stint_times.standard_fuel_stint.track_time == Duration::zero() {                                
                                return false;
                            }

                            self.schedule_rows = rows;
                            self.schedule_rows[0].update(event_config.race_start_utc, 
                                                         event_config.race_start_tod,
                                                         "".to_string(),
                                                         1,
                                                         fuel_stint_times,
                                                         event_config.race_end_utc,
                                                         fuel_stint_config.tire_change_time);
                            self.update_schedule(0);
                            true   
                        } else {
                            false
                        }
                    }
                    None => self.create_schedule()
                }
            }
            FuelStintScheduleMsg::UpdateFuelStintType(stint_type, index) => {
                let row = &mut self.schedule_rows[index];
                let (stint_duration, laps) = calculate_stint_duration_and_laps(row.utc_start,
                                                                       &stint_type,
                                                                       self.fuel_stint_times.as_ref().unwrap(),
                                                                       self.overall_event_config.as_ref().unwrap().race_end_utc,
                                                                       self.overall_fuel_stint_config.as_ref().unwrap().tire_change_time);
                row.stint_type = stint_type;
                row.utc_end = row.utc_start + stint_duration;
                row.tod_end = row.tod_start + stint_duration;
                row.actual_end = row.utc_end;
                row.calculated_laps = laps;
                row.actual_laps = laps;
                self.update_schedule(index);
                true
            }
            FuelStintScheduleMsg::UpdateActualEndTime(end_time, index) => {                
                let row = &mut self.schedule_rows[index];
                let mut actual_end_time = row.actual_end.with_hour(end_time.hour()).unwrap();
                actual_end_time = actual_end_time.with_minute(end_time.minute()).unwrap();
                actual_end_time = actual_end_time.with_second(end_time.second()).unwrap();
                
                row.actual_end = actual_end_time;
                row.duration_delta = actual_end_time - row.utc_end;
                self.update_schedule(index);
                true
            }
            FuelStintScheduleMsg::UpdateDriver(data, row_index) => {
                //subtract one from the selected index to account for the blank option
                let driver_index = data.index - 1;

                if driver_index < 0 {
                    let row = &mut self.schedule_rows[row_index];
                    row.driver_name = "".to_string();
                    row.stint_preference = 0;
                    row.stint_number = 1;                    
                } else {
                    let driver = &self.drivers.as_ref().unwrap()[driver_index as usize];
                    let mut stint_number = 1;                    
                    
                    if row_index > 0 {
                        let previous_row = &self.schedule_rows[row_index - 1];
                        let previous_row_driver_name = previous_row.driver_name.clone();
                        let previous_row_stint_number = previous_row.stint_number;
                        
                        if previous_row_driver_name == data.value {
                            stint_number = previous_row_stint_number + 1;
                        }
                    }
                    
                    let row = &mut self.schedule_rows[row_index];
                    row.driver_name = data.value.clone();
                    row.stint_preference = driver.stint_preference;
                    row.stint_number = stint_number;
                }
                self.update_schedule(row_index);
                true
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
                            <div class="mdc-data-table"
                                    ref=self.mdc_data_table_node_ref.clone()>
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
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Fuel" }<br/>{ "Stint" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "UTC Start" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "UTC End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "ToD Start" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "ToD End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Actual End" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Duration Delta" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Damage Modifier" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Calc" }<br/>{ "Laps" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Actual" }<br/>{ "Laps" }</th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Driver" }</th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Availability" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Stint" }<br/>{ "Num" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Stint" }<br/>{ "Pref" }</th>
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
                                            .map(|(index, row)| row.get_view(&self.link, index, self.drivers.as_ref()))
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
        if !self.schedule_rows.is_empty() && self.data_table.is_none() {
            self.data_table = self.mdc_data_table_node_ref
                .cast::<Element>()
                .map(MDCDataTable::new);          
        }
    }

    fn destroy(&mut self) {
        if !self.schedule_rows.is_empty() {
            self._producer.send(EventBusInput::PutSchedule(self.schedule_rows.clone()));
        }
        if let Some(data_table) = &self.data_table {
            data_table.destroy();
        }
    }
}