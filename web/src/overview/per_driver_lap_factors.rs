use chrono::Duration;
use yew::{ComponentLink, Component, ShouldRender, Html, html, props, Bridged, Bridge};
use yew::services::ConsoleService;
use crate::{md_text_field::{MaterialTextField, MaterialTextFieldProps}, format_duration, DurationFormat, AppRoutes, EventBus, Driver, parse_duration_from_str};
use yew_router::prelude::*;
use crate::event_bus::{EventBusInput, EventBusOutput};

pub struct DriverLapFactor {
    driver_name: String,
    driver_color: String,
    lap_time: Duration,
    factor: f64,
}

impl DriverLapFactor {
    fn compute_factor_from_reference(&mut self, reference: Duration) {
        self.factor = (self.lap_time.num_milliseconds() as f64) / (reference.num_milliseconds() as f64);
    }
}

pub struct PerDriverLapFactors {
    link: ComponentLink<Self>,
    factors: Vec<DriverLapFactor>,
    standard_lap_time: Duration,
    _producer: Box<dyn Bridge<EventBus>>,
}

pub enum PerDriverLapFactorsMsg {
    LoadDrivers(Vec<Driver>),
    UpdateReferenceLapTime(Duration),
    UpdateDriverLapTime(String, usize)
}

impl Component for PerDriverLapFactors {
    type Message = PerDriverLapFactorsMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut event_bus_bridge = EventBus::bridge(link.batch_callback(|message| {
            match message {
                EventBusOutput::SendDriverRoster(drivers) => {
                    Some(PerDriverLapFactorsMsg::LoadDrivers(drivers))
                },
                EventBusOutput::StandardLapTime(lap_time) => {
                    Some(PerDriverLapFactorsMsg::UpdateReferenceLapTime(lap_time.lap_time))
                }
                _ => None
            }
        }));
        event_bus_bridge.send(EventBusInput::GetDriverRoster);
        Self {
            link,
            factors: vec![],
            standard_lap_time: Duration::zero(),
            _producer: event_bus_bridge
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            PerDriverLapFactorsMsg::LoadDrivers(drivers) => {
                self.factors = drivers
                    .iter()
                    .map(|driver| DriverLapFactor {
                        driver_name: driver.name.clone(),
                        driver_color: driver.color.clone(),
                        lap_time: Duration::zero(),
                        factor: 1.0
                    })
                    .collect();
                true
            },
            PerDriverLapFactorsMsg::UpdateDriverLapTime(lap_time, index) => {
                let parsed_lap_time = parse_duration_from_str(lap_time.as_str(), DurationFormat::MinSecMilli);
                let driver = &mut self.factors[index];
                match parsed_lap_time {
                    Ok(lap_time) => {
                        driver.lap_time = lap_time;
                        driver.compute_factor_from_reference(self.standard_lap_time);
                        true
                    },
                    Err(message) => {
                        ConsoleService::error(format!("{} driver's lap factor lap time parse failed: {}", driver.driver_name, message).as_str());
                        false
                    }
                }
            },
            PerDriverLapFactorsMsg::UpdateReferenceLapTime(lap_time) => {
                self.standard_lap_time = lap_time;
                for factor in &mut self.factors {
                    factor.compute_factor_from_reference(lap_time);
                }
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div id="driver-lap-factors" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Per Driver Lap Factors" }</div>
                </div>
                {
                    if self.factors.len() == 0 {
                        html!{
                            <p>{ "Add drivers on the "}<RouterAnchor<AppRoutes> route=AppRoutes::Roster>{"Roster"}</RouterAnchor<AppRoutes>>{" page"}</p>
                        }
                    } else {
                        html!{                           
                            <div class="mdc-data-table">
                              <div class="mdc-data-table__table-container">
                                <table class="mdc-data-table__table">
                                  <thead>
                                    <tr class="mdc-data-table__header-row">
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Driver" }</th>
                                      <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Laptime  (MM:SS.mmm)" }</th>
                                      <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Factor" }</th>
                                    </tr>
                                  </thead>
                                  <tbody class="mdc-data-table__content">
                                    {
                                        self.factors
                                            .iter()
                                            .enumerate()
                                            .map(|(index,f)| render_driver_lap_factor(f, &self.link, index))
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
}

fn render_driver_lap_factor(factor: &DriverLapFactor, link: &ComponentLink<PerDriverLapFactors>, index: usize) -> Html { 
    let lap_time_props = props!(MaterialTextFieldProps {
        value: format_duration(factor.lap_time, DurationFormat::MinSecMilli),
        label: None,
        on_change: link.callback(move |value| PerDriverLapFactorsMsg::UpdateDriverLapTime(value, index)),
        end_aligned: true,
    });
    html! {
        <tr class="mdc-data-table__row">
          <td class="mdc-data-table__cell" style={ format!("background-color: {}", factor.driver_color.clone()) }>
            <span>{ factor.driver_name.clone() }</span> 
          </td>
          <td class="mdc-data-table__cell">
            <MaterialTextField with lap_time_props />
          </td>
          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format!("{:.2}", factor.factor) }</td>
        </tr>
    }
}