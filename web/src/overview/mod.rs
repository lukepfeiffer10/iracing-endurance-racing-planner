use crate::planner::RacePlannerContext;
use chrono::Duration;
use yew::context::ContextHandle;
use yew::{html, props, Callback, Component, Context, Html};

use self::fuel_stint_times::{FuelStintTimes, FuelStintTimesProps};
use self::overall_event_config::EventConfig;
use self::overall_fuel_stint_config::OverallFuelStintConfig;
use self::per_driver_lap_factors::PerDriverLapFactors;
use self::time_of_day_lap_factors::TimeOfDayLapFactors;

pub(crate) mod fuel_stint_times;
pub(crate) mod overall_event_config;
pub(crate) mod overall_fuel_stint_config;
pub(crate) mod per_driver_lap_factors;
pub(crate) mod time_of_day_lap_factors;

pub struct Overview {
    _context_listener: ContextHandle<RacePlannerContext>,
}

pub enum OverviewMsg {
    ContextUpdate,
}

impl Component for Overview {
    type Message = OverviewMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (_, context_handle) = ctx
            .link()
            .context::<RacePlannerContext>(ctx.link().callback(|_| OverviewMsg::ContextUpdate))
            .expect("planner context to be set");
        Self {
            _context_listener: context_handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            OverviewMsg::ContextUpdate => true,
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (planner_context, _) = ctx
            .link()
            .context::<RacePlannerContext>(Callback::noop())
            .expect("planner context to be set");

        let fuel_stint_times_props = planner_context
            .data
            .overall_fuel_stint_config
            .as_ref()
            .map(|config| {
                props! {
                    FuelStintTimesProps {
                        fuel_tank_size: config.fuel_tank_size,
                        pit_duration: config.pit_duration
                    }
                }
            })
            .unwrap_or(FuelStintTimesProps {
                fuel_tank_size: 0,
                pit_duration: Duration::zero(),
            });

        let standard_lap_time = planner_context
            .data
            .fuel_stint_average_times
            .as_ref()
            .map(|times| times.standard_fuel_stint.lap_time)
            .unwrap_or(Duration::zero());
        html! {
            <div class="mdc-typography flex-container flex-row">
                <div id="left-column" class="flex-container flex-column">
                    <EventConfig />
                    <OverallFuelStintConfig />
                </div>
                <div id="right-column" class="flex-container flex-column">
                    <div class="flex-container flex-row flex-justify-content-center">
                        <FuelStintTimes ..fuel_stint_times_props />
                    </div>
                    <div class="flex-container flex-row">
                        <TimeOfDayLapFactors lap_time={standard_lap_time} />
                        <PerDriverLapFactors lap_time={standard_lap_time} />
                    </div>
                    <div class="flex-container flex-row">
                        <div style="flex-grow: 1">
                            <h3>{ "Realtime Deltas" }</h3>
                        </div>
                        <div style="flex-grow: 1">
                            <h3>{ "Manual Fuel Stint Calculator"} </h3>
                        </div>
                    </div>
                    <div class="flex-container flex-row">
                        <div style="flex-grow: 1">
                            <h3>{ "Final Fuel Stint Calculator" }</h3>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
