use crate::http::plans::patch_plan;
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps};
use crate::planner::{
    format_duration, parse_duration_from_str, DurationFormat, RacePlannerAction, RacePlannerContext,
};
use chrono::Duration;
use endurance_racing_planner_common::{
    FuelStintAverageTimes, PatchFuelStintAverageTimes, PatchRacePlannerDto, StintDataDto,
};
use gloo_console::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::context::ContextHandle;
use yew::prelude::*;
use yew::props;
use yew::Properties;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

impl Eq for StintData {}

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

impl Default for StintData {
    fn default() -> Self {
        Self::new()
    }
}

impl From<StintDataDto> for StintData {
    fn from(dto: StintDataDto) -> Self {
        Self {
            lap_time: dto.lap_time,
            fuel_per_lap: dto.fuel_per_lap,
            lap_count: dto.lap_count,
            lap_time_with_pit: dto.lap_time_with_pit,
            track_time: dto.track_time,
            track_time_with_pit: dto.track_time_with_pit,
            fuel_per_stint: dto.fuel_per_stint,
        }
    }
}

impl From<StintData> for StintDataDto {
    fn from(val: StintData) -> Self {
        StintDataDto {
            lap_time: val.lap_time,
            fuel_per_lap: val.fuel_per_lap,
            lap_count: val.lap_count,
            lap_time_with_pit: val.lap_time_with_pit,
            track_time: val.track_time,
            track_time_with_pit: val.track_time_with_pit,
            fuel_per_stint: val.fuel_per_stint,
        }
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
    OnCreate(FuelStintAverageTimes),
}

#[derive(Properties, PartialEq)]
pub struct FuelStintTimesProps {
    pub fuel_tank_size: i32,
    pub pit_duration: Duration,
}

pub struct FuelStintTimes {
    standard_fuel_stint: StintData,
    fuel_saving_stint: StintData,
    _context_listener: ContextHandle<RacePlannerContext>,
}

impl Component for FuelStintTimes {
    type Message = FuelStintTimesMsg;
    type Properties = FuelStintTimesProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, planner_context_handle) = ctx
            .link()
            .context::<RacePlannerContext>(ctx.link().batch_callback(
                |context: RacePlannerContext| {
                    context
                        .data
                        .fuel_stint_average_times
                        .as_ref()
                        .map(|data| FuelStintTimesMsg::OnCreate(data.clone()))
                },
            ))
            .expect("No Planner Context Provided");

        let stint_data = planner_context.data.fuel_stint_average_times.as_ref();
        Self {
            standard_fuel_stint: stint_data
                .map(|fs| fs.standard_fuel_stint.clone().into())
                .unwrap_or_else(StintData::new),
            fuel_saving_stint: stint_data
                .map(|fs| fs.fuel_saving_stint.clone().into())
                .unwrap_or_else(StintData::new),
            _context_listener: planner_context_handle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context to be populated");
        let plan_id = planner_context.data.id;

        let Self::Properties {
            pit_duration,
            fuel_tank_size,
        } = ctx.props();

        match msg {
            FuelStintTimesMsg::UpdateLapTime(value, stint_type) => {
                let parsed_lap_time =
                    parse_duration_from_str(value.as_str(), DurationFormat::MinSecMilli);
                match parsed_lap_time.map(|parsed_lap_time| match stint_type {
                    StintType::Standard => {
                        self.standard_fuel_stint
                            .update_lap_time(parsed_lap_time, *pit_duration);
                        let fuel_saving_lap_time =
                            (parsed_lap_time.num_milliseconds() as f64) * 1.01;
                        self.fuel_saving_stint.update_lap_time(
                            Duration::milliseconds(fuel_saving_lap_time.floor() as i64),
                            *pit_duration,
                        );
                        send_patch_request(plan_id, self.standard_fuel_stint.clone(), &stint_type);
                    }
                    StintType::FuelSaving => {
                        self.fuel_saving_stint
                            .update_lap_time(parsed_lap_time, *pit_duration);
                        send_patch_request(plan_id, self.fuel_saving_stint.clone(), &stint_type);
                    }
                }) {
                    Ok(_) => {
                        planner_context.dispatch(RacePlannerAction::SetFuelStintTimes(
                            FuelStintAverageTimes {
                                standard_fuel_stint: self.standard_fuel_stint.clone().into(),
                                fuel_saving_stint: self.fuel_saving_stint.clone().into(),
                            },
                        ));
                        true
                    }
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
                    StintType::Standard => {
                        self.standard_fuel_stint.update_fuel_per_lap(
                            parsed_fuel_per_lap,
                            *fuel_tank_size,
                            *pit_duration,
                        );
                        send_patch_request(plan_id, self.standard_fuel_stint.clone(), &stint_type);
                    }
                    StintType::FuelSaving => {
                        self.fuel_saving_stint.update_fuel_per_lap(
                            parsed_fuel_per_lap,
                            *fuel_tank_size,
                            *pit_duration,
                        );
                        send_patch_request(plan_id, self.fuel_saving_stint.clone(), &stint_type);
                    }
                }) {
                    Ok(_) => {
                        planner_context.dispatch(RacePlannerAction::SetFuelStintTimes(
                            FuelStintAverageTimes {
                                standard_fuel_stint: self.standard_fuel_stint.clone().into(),
                                fuel_saving_stint: self.fuel_saving_stint.clone().into(),
                            },
                        ));
                        true
                    }
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
            FuelStintTimesMsg::OnCreate(data) => {
                self.fuel_saving_stint = data.fuel_saving_stint.into();
                self.standard_fuel_stint = data.standard_fuel_stint.into();
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let Self::Properties {
            pit_duration,
            fuel_tank_size,
        } = ctx.props();

        self.fuel_saving_stint
            .update(*fuel_tank_size, *pit_duration);
        self.standard_fuel_stint
            .update(*fuel_tank_size, *pit_duration);

        true
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
}

fn format_fuel_as_string(fuel: f32) -> String {
    format!("{:.2}", fuel)
}

fn send_patch_request(plan_id: Uuid, data: StintData, stint_type: &StintType) {
    if data.lap_time > Duration::zero() && data.fuel_per_lap > 0.0 {
        patch_plan(
            plan_id,
            PatchRacePlannerDto {
                id: plan_id,
                title: None,
                overall_event_config: None,
                overall_fuel_stint_config: None,
                fuel_stint_average_times: Some(PatchFuelStintAverageTimes {
                    standard_fuel_stint: match stint_type {
                        StintType::Standard => Some(data.clone().into()),
                        StintType::FuelSaving => None,
                    },
                    fuel_saving_stint: match stint_type {
                        StintType::Standard => None,
                        StintType::FuelSaving => Some(data.into()),
                    },
                }),
                time_of_day_lap_factors: None,
                per_driver_lap_factors: None,
                driver_roster: None,
                schedule_rows: None,
            },
        )
    }
}
