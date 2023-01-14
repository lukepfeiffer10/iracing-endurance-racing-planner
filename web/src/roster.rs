use crate::bindings;
use crate::http::drivers::{create_plan_driver, update_driver};
use crate::md_text_field::{
    MaterialTextField, MaterialTextFieldIcon, MaterialTextFieldIconStyle, MaterialTextFieldProps,
};
use crate::planner::{RacePlannerAction, RacePlannerContext};
use serde::{Deserialize, Serialize};
use yew::context::ContextHandle;
use yew::prelude::*;
use yew::{html::Scope, props, Component, Context, Html};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Driver {
    pub id: i32,
    pub name: String,
    pub total_stints: i32,
    pub fair_share: bool,
    pub color: String,
    pub utc_offset: i16,
    pub irating: i16,
    pub stint_preference: i16,
}

impl From<&endurance_racing_planner_common::Driver> for Driver {
    fn from(driver: &endurance_racing_planner_common::Driver) -> Self {
        Self {
            id: driver.id,
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
            id: self.id,
            name: self.name,
            total_stints: self.total_stints,
            fair_share: self.fair_share,
            color: self.color,
            utc_offset: self.utc_offset,
            irating: self.irating,
            stint_preference: self.stint_preference,
            lap_time: None,
        }
    }
}

impl Driver {
    fn new() -> Self {
        Driver {
            id: 0,
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
                let value = value.parse::<i16>().unwrap();
                DriverRosterMsg::UpdateDriverUtcOffset(value, index)
            }),
        }};
        let irating_props = props! {MaterialTextFieldProps {
            value: self.irating.to_string(),
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i16>().unwrap();
                DriverRosterMsg::UpdateDriverIrating(value, index)
            }),
        }};
        let stint_preference_props = props! {MaterialTextFieldProps {
            value: self.stint_preference.to_string(),
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i16>().unwrap();
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
            id: self.id,
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
    UpdateDriverUtcOffset(i16, usize),
    UpdateDriverIrating(i16, usize),
    UpdateDriverStintPreference(i16, usize),
    UpdateDriverId(i32, usize),
}

pub struct DriverRoster {
    drivers: Vec<Driver>,
    planner_context: RacePlannerContext,
    _planner_context_handle: ContextHandle<RacePlannerContext>,
}

impl Component for DriverRoster {
    type Message = DriverRosterMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (planner_context, _planner_context_handle) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context must be set");
        Self {
            drivers: planner_context
                .data
                .driver_roster
                .iter()
                .map(|d| d.into())
                .collect(),
            planner_context,
            _planner_context_handle,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let render = match msg {
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
                if driver_to_update.id == 0 {
                    create_plan_driver(
                        self.planner_context.data.id,
                        driver_to_update.clone().into(),
                        ctx.link().callback(
                            move |driver: endurance_racing_planner_common::Driver| {
                                DriverRosterMsg::UpdateDriverId(driver.id, index)
                            },
                        ),
                    );
                } else {
                    update_driver(driver_to_update.clone().into());
                }
                false
            }
            DriverRosterMsg::UpdateDriverId(id, index) => {
                let driver_to_update = &mut self.drivers[index];
                driver_to_update.id = id;
                false
            }
        };

        self.planner_context
            .dispatch(RacePlannerAction::SetDriverRoster(
                self.drivers.iter().map(|d| d.clone().into()).collect(),
            ));
        return render;
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

    fn destroy(&mut self, _ctx: &Context<Self>) {}
}
