use endurance_racing_planner_common::schedule::ScheduleStintDto;
use uuid::Uuid;
use yew::Callback;

use super::{get, post, put};

pub fn create_schedule(plan_id: Uuid, schedule: Vec<ScheduleStintDto>) -> () {
    post::<Vec<ScheduleStintDto>, ()>(format!("/plans/{}/schedule", plan_id), schedule, None)
}

pub fn get_schedule(plan_id: Uuid, callback: Callback<Vec<ScheduleStintDto>>) -> () {
    get(format!("plans/{}/schedule", plan_id), callback)
}

pub fn update_schedule(plan_id: Uuid, schedule: Vec<ScheduleStintDto>) -> () {
    put::<Vec<ScheduleStintDto>>(format!("/plans/{}/schedule", plan_id), schedule)
}
