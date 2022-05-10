use endurance_racing_planner_common::RacePlannerDto;
use yew::Callback;

use super::{get, post};

pub fn create_plan(plan: RacePlannerDto, callback: Callback<RacePlannerDto>) -> () {
    post("/plans", plan, callback)
}

pub fn get_plans(callback: Callback<Vec<RacePlannerDto>>) -> () {
    get("/plans", callback)
}
