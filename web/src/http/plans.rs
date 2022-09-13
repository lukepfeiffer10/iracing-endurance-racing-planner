use endurance_racing_planner_common::{PatchRacePlannerDto, PlanListDto, RacePlannerDto};
use uuid::Uuid;
use yew::Callback;

use super::{get, patch, post};

pub fn create_plan(plan: RacePlannerDto, callback: Callback<RacePlannerDto>) -> () {
    post("/plans".into(), plan, Some(callback))
}

pub fn get_plans(callback: Callback<Vec<PlanListDto>>) -> () {
    get("/plans".into(), callback)
}

pub fn get_plan(id: Uuid, callback: Callback<RacePlannerDto>) -> () {
    get(format!("/plans/{}", id), callback)
}

pub fn patch_plan(id: Uuid, plan: PatchRacePlannerDto) -> () {
    patch(format!("/plans/{}", id), plan)
}
