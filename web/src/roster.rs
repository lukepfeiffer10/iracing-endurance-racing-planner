use yew::prelude::*;
use yew::{Component, ComponentLink, Html, ShouldRender, props};
use serde::{Serialize, Deserialize};
use crate::bindings;
use crate::event_bus::{EventBus, EventBusInput, EventBusOutput};
use crate::md_text_field::{MaterialTextField, MaterialTextFieldProps, MaterialTextFieldIcon, MaterialTextFieldIconStyle};

#[derive(Serialize, Deserialize, Debug)]
pub struct Driver {
    pub name: String,
    pub total_stints: i32,
    pub fair_share: bool,
    pub color: String,
    pub utc_offset: i32,
    pub irating: i32,
    pub stint_preference: i32,
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
            stint_preference: 0
        }
    }
    
    fn get_view(&self, link: &ComponentLink<DriverRoster>, index: usize) -> Html {
        let name_props = props!(MaterialTextFieldProps {
            value: self.name.clone(),
            label: None,
            on_change: link.callback(move |value| {
                DriverRosterMsg::UpdateDriverName(value, index)
            })
        });        
        let color_props = props!(MaterialTextFieldProps {
            value: self.color.clone(),
            label: None,
            on_change: link.callback(move |value| {
                DriverRosterMsg::UpdateDriverColor(value, index)
            }),            
            icon: MaterialTextFieldIcon {
                style: MaterialTextFieldIconStyle::Leading,
                icon: "a".to_string(),
                on_click: None,
                background_color: Some(self.color.clone()),
            }
        });
        let utc_offset_props = props!(MaterialTextFieldProps {
            value: self.utc_offset.to_string(),
            label: None,
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverUtcOffset(value, index)
            }),
        });
        let irating_props = props!(MaterialTextFieldProps {
            value: self.irating.to_string(),
            label: None,
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverIrating(value, index)
            }),
        });
        let stint_preference_props = props!(MaterialTextFieldProps {
            value: self.stint_preference.to_string(),
            label: None,
            end_aligned: true,
            on_change: link.callback(move |value: String| {
                let value = value.parse::<i32>().unwrap();
                DriverRosterMsg::UpdateDriverStintPreference(value, index)
            }),
        });
        
        html! {
            <tr class="mdc-data-table__row">
              <td class="mdc-data-table__cell">
                <MaterialTextField with name_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                { self.total_stints }
              </td>
              <td class="mdc-data-table__cell">
                { format!("{}", self.fair_share) }
              </td>
              <td class="mdc-data-table__cell">
                <MaterialTextField with color_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField with utc_offset_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField with irating_props />
              </td>
              <td class="mdc-data-table__cell mdc-data-table__cell--numeric">
                <MaterialTextField with stint_preference_props />
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
            stint_preference: self.stint_preference
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
    link: ComponentLink<Self>,
    drivers: Vec<Driver>,
    producer: Box<dyn Bridge<EventBus>>,
}

impl DriverRoster {
    /// Calculate the average iRating of all drivers in the roster
    /// 
    /// If there are no drivers, returns 0.
    fn avg_ir(&self) -> i32 {
        if self.drivers.len() > 0 {
            self.drivers.iter().map(|d| d.irating).sum::<i32>() / (self.drivers.len() as i32)
        } else {
            0
        }
    }
}

impl Component for DriverRoster {
    type Message = DriverRosterMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut event_bus_bridge = EventBus::bridge(link.batch_callback(|message| {
            match message {
                EventBusOutput::SendDriverRoster(drivers) => {
                    Some(DriverRosterMsg::LoadDrivers(drivers))
                },
                _ => None
            }
        }));
        event_bus_bridge.send(EventBusInput::GetDriverRoster);
        Self {
            link,
            drivers: Vec::new(),
            producer: event_bus_bridge
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                // Need to update the average iRating
                true
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

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let avg_ir_props = props!(MaterialTextFieldProps {
            value: self.avg_ir().to_string(),
            label: None,
            id: "avg-ir".to_string(),
        });

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
                                .map(|(index, driver)| driver.get_view(&self.link, index))
                                .collect::<Vec<_>>()
                        }
                      </tbody>
                    </table>
                  </div>
                </div>
                <div class="mdc-card__actions">
                    <button class="material-icons mdc-icon-button mdc-card__action mdc-card__action--icon" 
                          title="New Driver"
                          onclick=self.link.callback(|_| DriverRosterMsg::AddDriver)>
                        
                        <div class="mdc-icon-button__ripple"></div>
                        {"add"}
                    </button>
                </div>
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Average iRating" }</div>
                </div>
                <MaterialTextField with avg_ir_props />
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            bindings::enable_icon_button(".mdc-icon-button");
        }
    }

    fn destroy(&mut self) {
        self.producer.send(EventBusInput::PutDriverRoster(self.drivers.clone()));
    }
}