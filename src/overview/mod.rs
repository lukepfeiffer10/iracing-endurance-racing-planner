use yew::{Component, ComponentLink, Html, html, ShouldRender};
use self::overall_event_config::EventConfig;
use self::overall_fuel_stint_config::OverallFuelStintConfig;
use self::fuel_stint_times::FuelStintTimes;
use self::time_of_day_lap_factors::TimeOfDayLapFactors;
use self::per_driver_lap_factors::PerDriverLapFactors;

pub(crate) mod fuel_stint_times;
pub(crate) mod overall_fuel_stint_config;
mod time_of_day_lap_factors;
mod per_driver_lap_factors;
pub(crate) mod overall_event_config;

pub struct Overview;

impl Component for Overview {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html!{
            <div class="mdc-typography flex-container flex-row">
                <div id="left-column" class="flex-container flex-column">
                    <EventConfig />
                    <OverallFuelStintConfig />
                </div>
                <div id="right-column" class="flex-container flex-column">
                    <div class="flex-container flex-row flex-justify-content-center">
                        <FuelStintTimes />
                    </div>
                    <div class="flex-container flex-row">
                        <TimeOfDayLapFactors />
                        <PerDriverLapFactors />
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