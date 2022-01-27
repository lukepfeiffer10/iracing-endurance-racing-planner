pub(crate) mod fuel_stint_schedule;

use yew::prelude::*;
use yew::{Component, ComponentLink, Html, ShouldRender};
use fuel_stint_schedule::FuelStintSchedule;

pub struct Schedule;

impl Component for Schedule {
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
                <FuelStintSchedule />
                <div id="driver-availability" class="mdc-card">
                    <div class="mdc-card-wrapper__text-section">
                        <div class="card-title">{ "Driver Availability" }</div>
                    </div>
                </div>
            </div>
        }
    }
}