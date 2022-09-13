use crate::bindings;
use crate::md_text_field::{
    MaterialTextField, MaterialTextFieldIcon, MaterialTextFieldIconStyle, MaterialTextFieldProps,
};
use crate::planner::{RacePlannerAction, RacePlannerContext};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew::{html::Scope, props, Component, Context, Html};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Driver {
    pub name: String,
    pub total_stints: i32,
    pub fair_share: bool,
    pub color: String,
    pub utc_offset: i32,
    pub irating: i32,
    pub stint_preference: i32,
}

impl From<&endurance_racing_planner_common::Driver> for Driver {
    fn from(driver: &endurance_racing_planner_common::Driver) -> Self {
        Self {
            name: driver.name.clone(),
            total_stints: driver.total_stints,
            fair_share: driver.fair_share,
            color: driver.color.clone(),
            utc_offset: driver.utc_offset,
            irating: driver.irating,
            stint_preference: driver.stint_preference,
        }
    }
}

impl Into<endurance_racing_planner_common::Driver> for Driver {
    fn into(self) -> endurance_racing_planner_common::Driver {
        endurance_racing_planner_common::Driver {
            name: self.name,
            total_stints: self.total_stints,
            fair_share: self.fair_share,
            color: self.color,
            utc_offset: self.utc_offset,
            irating: self.irating,
            stint_preference: self.stint_preference,
        }
    }
}

impl Driver {
    fn new() -> Self {
        Driver {
            name: "".to_string(),
            total_stints: 0,
            fair_share: false,
            color: "#FFFFFF".to_string(),
            utc_offset: 0,
            irating: 0,
            stint_preference: 0,
        }
    }

    fn get_view(&self, link: &Scope<DriverRoster>, index: usize) -> Html {
        let name_props = props! {MaterialTextFieldProps {
            value: self.name.clone(),
            on_change: link.callback(move |value| {
                DriverRosterMsg::UpdateDriverName(value, index)
            })
        }};
        let color_props = props! {MaterialTextFieldProps {
            value: self.color.clone(),
            on_change: link.callback(move |value| {
                DriverRosterMsg::UpdateDriverColor(value, index)
            }),
            icon: MaterialTextFieldIcon {
                style: MaterialTextFieldIconStyle::Leading,
                icon: "a".to_string(),
                on_click: None,
                background_color: Some(self.color.clone()),
            }
        }};
        let utc_offset_props = props! {MaterialTextFieldProps {
            value: self.utc_offset.to_string(),
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverUtcOffset(value, index)
            }),
        }};
        let irating_props = props! {MaterialTextFieldProps {
            value: self.irating.to_string(),
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverIrating(value, index)
            }),
        }};
        let stint_preference_props = props! {MaterialTextFieldProps {
            value: self.stint_preference.to_string(),
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverStintPreference(value, index)
            }),
        }};

        html! {
            <tr class="mdc-data-table__row">
              <td class="mdc-data-table__cell">
                <MaterialTextField ..name_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                { self.total_stints }
              </td>
              <td class="mdc-data-table__cell">
                { format!("{}", self.fair_share) }
              </td>
              <td class="mdc-data-table__cell">
                <MaterialTextField ..color_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField ..utc_offset_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField ..irating_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField ..stint_preference_props />
              </td>
            </tr>
        }
    }
}

impl Clone for Driver {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            total_stints: self.total_stints,
            fair_share: self.fair_share,
            color: self.color.clone(),
            utc_offset: self.utc_offset,
            irating: self.irating,
            stint_preference: self.stint_preference,
        }
    }
}

pub enum DriverRosterMsg {
    AddDriver,
    UpdateDriverName(String, usize),
    UpdateDriverColor(String, usize),
    UpdateDriverUtcOffset(i32, usize),
    UpdateDriverIrating(i32, usize),
    UpdateDriverStintPreference(i32, usize),
    LoadDrivers(Vec<Driver>),
}

pub struct DriverRoster {
    drivers: Vec<Driver>,
    planner_context: RacePlannerContext,
}

impl Component for DriverRoster {
    type Message = DriverRosterMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(ctx.link().callback(
                |race_planner_context: RacePlannerContext| {
                    DriverRosterMsg::LoadDrivers(
                        race_planner_context
                            .data
                            .driver_roster
                            .iter()
                            .map(|d| d.into())
                            .collect(),
                    )
                },
            ))
            .expect("planner context must be set");
        Self {
            drivers: Vec::new(),
            planner_context: planner_context,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DriverRosterMsg::AddDriver => {
                self.drivers.push(Driver::new());
                true
            }
            DriverRosterMsg::UpdateDriverName(name, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.name = name;
                false
            }
            DriverRosterMsg::UpdateDriverColor(color, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.color = color;
                true
            }
            DriverRosterMsg::UpdateDriverUtcOffset(offset, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.utc_offset = offset;
                false
            }
            DriverRosterMsg::UpdateDriverIrating(irating, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.irating = irating;
                false
            }
            DriverRosterMsg::UpdateDriverStintPreference(stint_preference, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.stint_preference = stint_preference;
                false
            }
            DriverRosterMsg::LoadDrivers(drivers) => {
                self.drivers = drivers;
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Driver Roster" }</div>
                </div>
                <div class="mdc-data-table">
                  <div class="mdc-data-table__table-container">
                    <table class="mdc-data-table__table">
                      <thead>
                        <tr class="mdc-data-table__header-row">
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Driver" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Total Stints" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Fair Share" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Color" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "UTC Offset" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "iRating" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Stint Preference" }</th>
                        </tr>
                      </thead>
                      <tbody class="mdc-data-table__content">
                        {
                            self.drivers
                                .iter()
                                .enumerate()
                                .map(|(index, driver)| driver.get_view(ctx.link(), index))
                                .collect::<Vec<_>>()
                        }
                      </tbody>
                    </table>
                  </div>
                </div>
                <div class="mdc-card__actions">
                    <button class="material-icons mdc-icon-button mdc-card__action mdc-card__action--icon"
                          title="New Driver"
                          onclick={ctx.link().callback(|_| DriverRosterMsg::AddDriver)}>

                        <div class="mdc-icon-button__ripple"></div>
                        {"add"}
                    </button>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            bindings::enable_icon_button(".mdc-icon-button");
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.planner_context
            .dispatch(RacePlannerAction::SetDriverRoster(
                self.drivers.iter().map(|d| d.clone().into()).collect(),
            ));
    }
}
