use chrono::Duration;
use yew::{ComponentLink, Component, ShouldRender, Html, html, props};
use crate::{md_text_field::{MaterialTextField, MaterialTextFieldProps}, format_duration, DurationFormat, AppRoutes};
use yew_router::prelude::*;

struct DriverLapFactor {
    driver_name: String,
    lap_time: Duration,
    factor: f64,
}

pub struct PerDriverLapFactors {
    link: ComponentLink<Self>,
    factors: Vec<DriverLapFactor>,
}

impl Component for PerDriverLapFactors {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            factors: vec![]
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
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
    let driver_name_props = props!(MaterialTextFieldProps {
        value: factor.driver_name.clone(),
        label: None,
        on_change: link.callback(|_| ()),
    });
    let lap_time_props = props!(MaterialTextFieldProps {
        value: format_duration(factor.lap_time, DurationFormat::MinSecMilli),
        label: None,
        on_change: link.callback(|_| ()),
        end_aligned: true,
    });
    html! {
        <tr class="mdc-data-table__row">
          <td class="mdc-data-table__cell">
            <MaterialTextField with driver_name_props />
          </td>
          <td class="mdc-data-table__cell">
            <MaterialTextField with lap_time_props />
          </td>
          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format!("{:.2}", factor.factor) }</td>
        </tr>
    }
}