use crate::http;
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use crate::planner::{
    format_duration, parse_duration_from_str, DurationFormat, PlannerRoutes, RacePlannerAction,
    RacePlannerContext,
};
use chrono::{Duration, NaiveTime, Timelike};
use endurance_racing_planner_common::schedule::{ScheduleStintDto, StintType};
use endurance_racing_planner_common::{
    Driver, EventConfigDto, FuelStintAverageTimes, OverallFuelStintConfigData,
};
use gloo_console::error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
use web_sys::Element;
use yew::prelude::*;
use yew::{html::Scope, props, Component, Context, Html};
use yew_mdc::{
    components::{select::SelectChangeEventData, Select, SelectItem},
    mdc_sys::MDCDataTable,
};
use yew_router::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ScheduleRow {
    stint_data: ScheduleStintDto,
}

impl ScheduleRow {
    fn get_view(
        &self,
        link: &Scope<FuelStintSchedule>,
        index: usize,
        drivers: Option<&Vec<Driver>>,
    ) -> Html {
        let time_format = "%l:%M %p"; // (H)H:MM AM|PM
        let actual_end_on_change = link.batch_callback(move |value: String| {
            let value_as_str = value.as_str();
            let actual_end_time = NaiveTime::parse_from_str(value_as_str, time_format)
                .or_else(|_| NaiveTime::parse_from_str(value_as_str, "%R")) //HH:MM
                .or_else(|_| NaiveTime::parse_from_str(value_as_str, "%T")) //HH:MM:SS
                .or_else(|_| NaiveTime::parse_from_str(value_as_str, "%l:%M:%S %p")); // (H)H:MM:SS AM|PM

            match actual_end_time {
                Ok(value) => Some(FuelStintScheduleMsg::UpdateActualEndTime(value, index)),
                Err(e) => {
                    error!(format!("The actual end time could not be parsed: {}", e).as_str());
                    None
                }
            }
        });
        let actual_end_props = props!(MaterialTextFieldProps {
            value: self.stint_data.actual_end.format(time_format).to_string(),
            end_aligned: true,
            on_change: actual_end_on_change
        });

        let damage_modifier_on_change = link.batch_callback(move |value: String| {
            let damage_modifier = parse_duration_from_str(&value, DurationFormat::MinSecMilli);

            match damage_modifier {
                Ok(value) => Some(FuelStintScheduleMsg::UpdateDamageModifier(value, index)),
                Err(e) => {
                    error!(format!("Damage modifier could not be parsed: {}", e).as_str());
                    None
                }
            }
        });
        let damage_modifier_props = props!(MaterialTextFieldProps {
            value: format_duration(self.stint_data.damage_modifier, DurationFormat::MinSecMilli),
            end_aligned: true,
            on_change: damage_modifier_on_change,
        });
        let actual_laps_props = props!(MaterialTextFieldProps {
            value: self.stint_data.actual_laps.to_string(),
            end_aligned: true,
        });
        let stint_type_onchange = link.batch_callback(move |data: SelectChangeEventData| {
            let stint_type = StintType::from_str(data.value.as_str());
            match stint_type {
                Ok(stint_type) => {
                    Some(FuelStintScheduleMsg::UpdateFuelStintType(stint_type, index))
                }
                Err(s) => {
                    error!(s.as_str());
                    None
                }
            }
        });
        let driver_name_on_change = link.callback(move |data: SelectChangeEventData| {
            FuelStintScheduleMsg::UpdateDriver(data, index)
        });

        let driver =
            drivers.and_then(|drivers| drivers.iter().find(|d| d.id == self.stint_data.driver_id));

        let (local_start, local_end, driver_color) = if let Some(driver) = driver {
            (
                self.stint_data.utc_start + Duration::hours(driver.utc_offset as i64),
                self.stint_data.utc_end + Duration::hours(driver.utc_offset as i64),
                driver.color.as_str(),
            )
        } else {
            (self.stint_data.utc_start, self.stint_data.utc_end, "")
        };

        let row_id = format!("row-{}", index);
        html! {
            <tr data-row-id={row_id.clone()} class="mdc-data-table__row">
                <td class="mdc-data-table__cell mdc-data-table__cell--checkbox">
                    <div class="mdc-touch-target-wrapper">
                        <div class="mdc-checkbox mdc-checkbox--touch mdc-data-table__row-checkbox">
                            <input type="checkbox" class="mdc-checkbox__native-control" aria-labelledby={row_id.clone()} />
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
                    <Select id={format!("stint-type-{}", index)}
                        select_width_class="select-width"
                        fixed_position={true}
                        selected_value={Some(self.stint_data.stint_type.to_string())}
                        onchange={stint_type_onchange}>
                        <SelectItem text={StintType::FuelSavingNoTires.to_string()}
                            value={StintType::FuelSavingNoTires.to_string()} />
                        <SelectItem text={StintType::FuelSavingWithTires.to_string()}
                            value={StintType::FuelSavingWithTires.to_string()} />
                        <SelectItem text={StintType::StandardNoTires.to_string()}
                            value={StintType::StandardNoTires.to_string()} />
                        <SelectItem text={StintType::StandardWithTires.to_string()}
                            value={StintType::StandardWithTires.to_string()} />
                    </Select>
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.fuel_stint_number }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.utc_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.utc_end.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.tod_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.tod_end.format(time_format) }</td>
                <td class="mdc-data-table__cell">
                    <MaterialTextField ..actual_end_props />
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format_duration(self.stint_data.duration_delta, DurationFormat::HourMinSec) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                    <MaterialTextField ..damage_modifier_props />
                </td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.calculated_laps }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                    <MaterialTextField ..actual_laps_props />
                </td>
                <td class="mdc-data-table__cell">
                    <Select id={format!("driver-name-{}", index)}
                        select_width_class="select-width"
                        fixed_position={true}
                        selected_value={Some(self.stint_data.driver_id.to_string())}
                        onchange={driver_name_on_change}>
                        //style={format!("background-color: {}", driver_color)}>
                        <SelectItem text="" value="0" />
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
                <td class="mdc-data-table__cell">{ self.stint_data.availability.clone() }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.stint_number }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ driver.map_or(0, |d| d.stint_preference) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ self.stint_data.factor }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric" style={format!("background-color: {}", driver_color)}>{ local_start.format(time_format) }</td>
                <td class="mdc-data-table__cell mdc-data-table__cell--numeric" style={format!("background-color: {}", driver_color)}>{ local_end.format(time_format) }</td>
            </tr>
        }
    }
}

fn get_driver_select_view(driver: &Driver) -> Html {
    html! {
        <SelectItem text={driver.name.clone()} value={driver.id.to_string()} />
    }
}

pub enum FuelStintScheduleMsg {
    UpdateFuelStintType(StintType, usize),
    UpdateActualEndTime(NaiveTime, usize),
    UpdateDriver(SelectChangeEventData, usize),
    UpdateDamageModifier(Duration, usize),
}

pub struct FuelStintSchedule {
    plan_id: Uuid,
    schedule_rows: Vec<ScheduleRow>,
    overall_event_config: Option<EventConfigDto>,
    fuel_stint_times: Option<FuelStintAverageTimes>,
    overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
    drivers: Option<Vec<Driver>>,
    mdc_data_table_node_ref: NodeRef,
    data_table: Option<MDCDataTable>,
    context: RacePlannerContext,
}

impl FuelStintSchedule {
    fn update_schedule(&mut self, update_row_index: usize) {
        let event_config = self.overall_event_config.as_ref().unwrap();
        let fuel_stint_times = self.fuel_stint_times.as_ref().unwrap();
        let fuel_stint_config = self.overall_fuel_stint_config.as_ref().unwrap();

        let initial_schedule_length = self.schedule_rows.len();
        update_schedule(
            &mut self.schedule_rows,
            update_row_index,
            event_config,
            fuel_stint_times,
            fuel_stint_config,
        );

        if initial_schedule_length != self.schedule_rows.len() {
            if let Some(data_table) = self.data_table.take() {
                data_table.destroy();
            }
        }
    }
}

fn create_schedule(
    plan_id: Uuid,
    overall_event_config: Option<EventConfigDto>,
    fuel_stint_times: Option<FuelStintAverageTimes>,
    overall_fuel_stint_config: Option<OverallFuelStintConfigData>,
) -> Vec<ScheduleRow> {
    if overall_event_config.is_some()
        && fuel_stint_times.is_some()
        && overall_fuel_stint_config.is_some()
    {
        let event_config = overall_event_config.as_ref().unwrap();
        let fuel_stint_times = fuel_stint_times.as_ref().unwrap();
        let fuel_stint_config = overall_fuel_stint_config.as_ref().unwrap();

        if event_config.race_start_utc == event_config.race_end_utc
            || fuel_stint_times.standard_fuel_stint.track_time == Duration::zero()
        {
            return vec![];
        }

        let mut schedule_rows = vec![];
        let stint_data = ScheduleStintDto::new(event_config, fuel_stint_times);
        let mut is_schedule_complete = stint_data.utc_end >= event_config.race_end_utc;
        schedule_rows.push(ScheduleRow { stint_data });

        while !is_schedule_complete {
            let previous_row = schedule_rows.last().unwrap();
            let stint_data = ScheduleStintDto::from_previous(
                &previous_row.stint_data,
                StintType::FuelSavingWithTires,
                fuel_stint_times,
                event_config.race_end_utc,
                fuel_stint_config.tire_change_time,
                Duration::zero(),
            );
            is_schedule_complete = stint_data.utc_end >= event_config.race_end_utc;
            schedule_rows.push(ScheduleRow { stint_data });
        }

        http::schedules::create_schedule(
            plan_id,
            schedule_rows
                .iter()
                .map(|row| row.stint_data.clone())
                .collect(),
        );
        schedule_rows
    } else {
        vec![]
    }
}

fn update_schedule(
    schedule_rows: &mut Vec<ScheduleRow>,
    update_row_index: usize,
    event_config: &EventConfigDto,
    fuel_stint_times: &FuelStintAverageTimes,
    fuel_stint_config: &OverallFuelStintConfigData,
) {
    let updated_row = &schedule_rows[update_row_index];
    let mut is_schedule_complete = updated_row.stint_data.utc_end >= event_config.race_end_utc;
    let mut next_row_index = update_row_index;
    let mut damage_modifier = updated_row.stint_data.damage_modifier;
    while !is_schedule_complete {
        next_row_index += 1;
        if next_row_index == schedule_rows.len() {
            let previous_row = schedule_rows.last().unwrap();
            let stint_data = ScheduleStintDto::from_previous(
                &previous_row.stint_data,
                StintType::FuelSavingWithTires,
                fuel_stint_times,
                event_config.race_end_utc,
                fuel_stint_config.tire_change_time,
                damage_modifier,
            );

            is_schedule_complete = stint_data.utc_end >= event_config.race_end_utc;
            schedule_rows.push(ScheduleRow { stint_data });
        } else {
            let previous_row = &schedule_rows[next_row_index - 1];
            let previous_row_actual_end = previous_row.stint_data.actual_end;
            let previous_row_driver_id = previous_row.stint_data.driver_id;
            let previous_row_stint_number = previous_row.stint_data.stint_number;
            let previous_row_tod_end =
                previous_row_actual_end.naive_utc() + event_config.tod_offset;

            let next_row = &mut schedule_rows[next_row_index];
            next_row.stint_data.update(
                previous_row_actual_end,
                previous_row_tod_end,
                previous_row_driver_id,
                previous_row_stint_number,
                fuel_stint_times,
                event_config.race_end_utc,
                fuel_stint_config.tire_change_time,
                damage_modifier,
            );

            is_schedule_complete = next_row.stint_data.utc_end >= event_config.race_end_utc;
            damage_modifier = damage_modifier + next_row.stint_data.damage_modifier;
        }
    }

    while schedule_rows.len() > next_row_index + 1 {
        schedule_rows.pop();
    }
}

impl Component for FuelStintSchedule {
    type Message = FuelStintScheduleMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let plan_id = ctx
            .link()
            .route()
            .and_then(|route: PlannerRoutes| match route {
                PlannerRoutes::Schedule { id } => Some(id),
                _ => None,
            })
            .expect("plan id to be set from the route");
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("No Planner Context Provided");

        let overall_event_config = planner_context.data.overall_event_config.clone();
        let fuel_stint_times = planner_context.data.fuel_stint_average_times.clone();
        let overall_fuel_stint_config = planner_context.data.overall_fuel_stint_config.clone();
        let schedule_rows = planner_context.data.schedule_rows.as_ref().map(|stints| {
            stints
                .iter()
                .map(|stint_data| ScheduleRow {
                    stint_data: stint_data.clone(),
                })
                .collect::<Vec<_>>()
        });

        let schedule_rows = match schedule_rows {
            Some(mut rows) => {
                if overall_event_config.is_some()
                    && fuel_stint_times.is_some()
                    && overall_fuel_stint_config.is_some()
                {
                    let event_config = overall_event_config.as_ref().unwrap();
                    let fuel_stint_times = fuel_stint_times.as_ref().unwrap();
                    let fuel_stint_config = overall_fuel_stint_config.as_ref().unwrap();

                    if event_config.race_start_utc == event_config.race_end_utc
                        || fuel_stint_times.standard_fuel_stint.track_time == Duration::zero()
                    {
                        vec![]
                    } else {
                        rows[0].stint_data.update(
                            event_config.race_start_utc,
                            event_config.race_start_tod,
                            0,
                            1,
                            fuel_stint_times,
                            event_config.race_end_utc,
                            fuel_stint_config.tire_change_time,
                            Duration::zero(),
                        );
                        update_schedule(
                            &mut rows,
                            0,
                            event_config,
                            fuel_stint_times,
                            fuel_stint_config,
                        );
                        rows
                    }
                } else {
                    vec![]
                }
            }
            None => create_schedule(
                plan_id,
                overall_event_config,
                fuel_stint_times,
                overall_fuel_stint_config,
            ),
        };

        let drivers = if planner_context.data.driver_roster.is_empty() {
            None
        } else {
            Some(planner_context.data.driver_roster.clone())
        };

        Self {
            plan_id,
            schedule_rows,
            overall_event_config: planner_context.data.overall_event_config.clone(),
            fuel_stint_times: planner_context.data.fuel_stint_average_times.clone(),
            overall_fuel_stint_config: planner_context.data.overall_fuel_stint_config.clone(),
            drivers,
            mdc_data_table_node_ref: NodeRef::default(),
            data_table: None,
            context: planner_context,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FuelStintScheduleMsg::UpdateFuelStintType(stint_type, index) => {
                let previous_row_stint_data = if index > 0 {
                    Some(self.schedule_rows[index - 1].stint_data.clone())
                } else {
                    None
                };

                let stint_data = &mut self.schedule_rows[index].stint_data;
                stint_data.stint_type = stint_type;
                stint_data.update(
                    stint_data.utc_start,
                    stint_data.tod_start,
                    previous_row_stint_data
                        .as_ref()
                        .map(|row| row.driver_id)
                        .unwrap_or(0),
                    previous_row_stint_data
                        .as_ref()
                        .map(|row| row.stint_number)
                        .unwrap_or(0),
                    self.fuel_stint_times.as_ref().unwrap(),
                    self.overall_event_config.as_ref().unwrap().race_end_utc,
                    self.overall_fuel_stint_config
                        .as_ref()
                        .unwrap()
                        .tire_change_time,
                    previous_row_stint_data
                        .map(|row| row.damage_modifier)
                        .unwrap_or_else(Duration::zero),
                );
                self.update_schedule(index);
            }
            FuelStintScheduleMsg::UpdateActualEndTime(end_time, index) => {
                let stint_data = &mut self.schedule_rows[index].stint_data;
                let mut actual_end_time = stint_data.actual_end.with_hour(end_time.hour()).unwrap();
                actual_end_time = actual_end_time.with_minute(end_time.minute()).unwrap();
                actual_end_time = actual_end_time.with_second(end_time.second()).unwrap();

                stint_data.actual_end = actual_end_time;
                stint_data.duration_delta = actual_end_time - stint_data.utc_end;
                self.update_schedule(index);
            }
            FuelStintScheduleMsg::UpdateDriver(data, row_index) => {
                //subtract one from the selected index to account for the blank option
                let driver_index = data.index - 1;
                let selected_driver_id = data
                    .value
                    .parse::<i32>()
                    .expect("selected driver value to be an integer");

                if driver_index < 0 {
                    let stint_data = &mut self.schedule_rows[row_index].stint_data;
                    stint_data.driver_id = 0;
                    stint_data.stint_number = 1;
                } else {
                    let mut stint_number = 1;

                    if row_index > 0 {
                        let previous_stint_data = &self.schedule_rows[row_index - 1].stint_data;
                        let previous_row_driver_id = previous_stint_data.driver_id;
                        let previous_row_stint_number = previous_stint_data.stint_number;

                        if previous_row_driver_id == selected_driver_id {
                            stint_number = previous_row_stint_number + 1;
                        }
                    }

                    let stint_data = &mut self.schedule_rows[row_index].stint_data;
                    stint_data.driver_id = selected_driver_id;
                    stint_data.stint_number = stint_number;
                }
                self.update_schedule(row_index);
            }
            FuelStintScheduleMsg::UpdateDamageModifier(value, index) => {
                let previous_row_stint_data = if index > 0 {
                    Some(self.schedule_rows[index - 1].stint_data.clone())
                } else {
                    None
                };

                let stint_data = &mut self.schedule_rows[index].stint_data;
                stint_data.damage_modifier = value;
                stint_data.update(
                    stint_data.utc_start,
                    stint_data.tod_start,
                    previous_row_stint_data
                        .as_ref()
                        .map(|row| row.driver_id)
                        .unwrap_or(0),
                    previous_row_stint_data
                        .as_ref()
                        .map(|row| row.stint_number)
                        .unwrap_or(0),
                    self.fuel_stint_times.as_ref().unwrap(),
                    self.overall_event_config.as_ref().unwrap().race_end_utc,
                    self.overall_fuel_stint_config
                        .as_ref()
                        .unwrap()
                        .tire_change_time,
                    previous_row_stint_data
                        .map(|row| row.damage_modifier)
                        .unwrap_or_else(Duration::zero),
                );
                self.update_schedule(index);
            }
        }
        http::schedules::update_schedule(
            self.plan_id,
            self.schedule_rows
                .iter()
                .map(|s| s.stint_data.clone())
                .collect(),
        );
        true
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context must be set");
        html! {
            <div id="fuel-stint-schedule" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Fuel Stint Schedule" }</div>
                </div>
                {
                    if self.schedule_rows.is_empty() {
                        html!{
                            <p>
                                { "Complete the event config, fuel stint average times, and fuel stint config on the "}
                                <Link<PlannerRoutes> to={PlannerRoutes::Overview { id: planner_context.data.id }}>{"Overview"}</Link<PlannerRoutes>>
                                {" page"}
                            </p>
                        }
                    } else {
                        html! {
                            <div class="mdc-data-table"
                                    ref={self.mdc_data_table_node_ref.clone()}>
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
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Actual Laps" }</th>
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
                                            .map(|(index, row)| row.get_view(ctx.link(), index, self.drivers.as_ref()))
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

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if !self.schedule_rows.is_empty() && self.data_table.is_none() {
            self.data_table = self
                .mdc_data_table_node_ref
                .cast::<Element>()
                .map(MDCDataTable::new);
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if !self.schedule_rows.is_empty() {
            self.context.dispatch(RacePlannerAction::SetStints(
                self.schedule_rows
                    .iter()
                    .map(|row| row.stint_data.clone())
                    .collect(),
            ));
        }

        if let Some(data_table) = &self.data_table {
            data_table.destroy();
        }
    }
}
