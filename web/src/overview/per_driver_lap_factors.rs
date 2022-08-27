use crate::event_bus::{EventBusInput, EventBusOutput};
use crate::planner::PlannerContext;
use crate::{
    event_bus::EventBus,
    md_text_field::{MaterialTextField, MaterialTextFieldProps},
    planner::{format_duration, parse_duration_from_str, DurationFormat, PlannerRoutes},
    roster::Driver,
};
use chrono::Duration;
use gloo_console::error;
use yew::html::Scope;
use yew::prelude::*;
use yew::{props, Properties};
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

pub struct DriverLapFactor {
    driver_name: String,
    driver_color: String,
    lap_time: Duration,
    factor: f64,
}

impl DriverLapFactor {
    fn compute_factor_from_reference(&mut self, reference: Duration) {
        self.factor =
            (self.lap_time.num_milliseconds() as f64) / (reference.num_milliseconds() as f64);
    }
}

pub struct PerDriverLapFactors {
    factors: Vec<DriverLapFactor>,
    standard_lap_time: Duration,
    _producer: Box<dyn Bridge<EventBus>>,
}

pub enum PerDriverLapFactorsMsg {
    LoadDrivers(Vec<Driver>),
    UpdateDriverLapTime(String, usize),
}

#[derive(Properties, PartialEq)]
pub struct PerDriverLapFactorsProps {
    #[prop_or(Duration::zero())]
    pub lap_time: Duration,
}

impl Component for PerDriverLapFactors {
    type Message = PerDriverLapFactorsMsg;
    type Properties = PerDriverLapFactorsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut event_bus_bridge =
            EventBus::bridge(ctx.link().batch_callback(|message| match message {
                EventBusOutput::SendDriverRoster(drivers) => {
                    Some(PerDriverLapFactorsMsg::LoadDrivers(drivers))
                }
                _ => None,
            }));
        event_bus_bridge.send(EventBusInput::GetDriverRoster);
        Self {
            factors: vec![],
            standard_lap_time: Duration::zero(),
            _producer: event_bus_bridge,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PerDriverLapFactorsMsg::LoadDrivers(drivers) => {
                self.factors = drivers
                    .iter()
                    .map(|driver| DriverLapFactor {
                        driver_name: driver.name.clone(),
                        driver_color: driver.color.clone(),
                        lap_time: Duration::zero(),
                        factor: 1.0,
                    })
                    .collect();
                true
            }
            PerDriverLapFactorsMsg::UpdateDriverLapTime(lap_time, index) => {
                let parsed_lap_time =
                    parse_duration_from_str(lap_time.as_str(), DurationFormat::MinSecMilli);
                let driver = &mut self.factors[index];
                match parsed_lap_time {
                    Ok(lap_time) => {
                        driver.lap_time = lap_time;
                        driver.compute_factor_from_reference(self.standard_lap_time);
                        true
                    }
                    Err(message) => {
                        error!(format!(
                            "{} driver's lap factor lap time parse failed: {}",
                            driver.driver_name, message
                        )
                        .as_str());
                        false
                    }
                }
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let Self::Properties { lap_time } = ctx.props();
        self.standard_lap_time = *lap_time;
        for factor in &mut self.factors {
            factor.compute_factor_from_reference(*lap_time);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let (planner_context, _) = ctx
            .link()
            .context::<PlannerContext>(Callback::noop())
            .expect("planner context must be set");
        html! {
            <div id="driver-lap-factors" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Per Driver Lap Factors" }</div>
                </div>
                {
                    if self.factors.is_empty() {
                        html!{
                            <p>{ "Add drivers on the "}<Link<PlannerRoutes> to={PlannerRoutes::Roster { id: planner_context.data.id }}>{"Roster"}</Link<PlannerRoutes>>{" page"}</p>
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
                                            .map(|(index,f)| render_driver_lap_factor(f, link, index))
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

fn render_driver_lap_factor(
    factor: &DriverLapFactor,
    link: &Scope<PerDriverLapFactors>,
    index: usize,
) -> Html {
    let lap_time_props = props! {MaterialTextFieldProps {
        value: format_duration(factor.lap_time, DurationFormat::MinSecMilli),
        on_change: link.callback(move |value| PerDriverLapFactorsMsg::UpdateDriverLapTime(value, index)),
        end_aligned: true,
    }};
    html! {
        <tr class="mdc-data-table__row">
          <td class="mdc-data-table__cell" style={ format!("background-color: {}", factor.driver_color.clone()) }>
            <span>{ factor.driver_name.clone() }</span>
          </td>
          <td class="mdc-data-table__cell">
            <MaterialTextField ..lap_time_props />
          </td>
          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format!("{:.2}", factor.factor) }</td>
        </tr>
    }
}
