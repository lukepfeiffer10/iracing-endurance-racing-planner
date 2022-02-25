pub(crate) mod fuel_stint_schedule;

use yew::prelude::*;
use yew::{Component, Context, Html};
use fuel_stint_schedule::FuelStintSchedule;

pub struct Schedule;

impl Component for Schedule {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
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