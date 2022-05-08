use endurance_racing_planner_common::RacePlannerDto;
use yew::Callback;

use super::post;

pub fn create_plan(plan: RacePlannerDto, callback: Callback<RacePlannerDto>) -> () {
    post("/plans", plan, callback)
}
