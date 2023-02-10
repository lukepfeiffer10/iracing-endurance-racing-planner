use endurance_racing_planner_common::{PatchRacePlannerDto, PlanListDto, RacePlannerDto};
use uuid::Uuid;
use yew::Callback;

use super::{get, get_async, patch, post, CustomError};

pub fn create_plan(plan: RacePlannerDto, callback: Callback<RacePlannerDto>) {
    post("/plans".into(), plan, Some(callback))
}

pub fn get_plans(callback: Callback<Vec<PlanListDto>>) {
    get("/plans".into(), callback)
}

pub async fn get_plan_async(id: Uuid) -> Result<RacePlannerDto, CustomError> {
    get_async(format!("/plans/{}", id)).await
}

pub fn patch_plan(id: Uuid, plan: PatchRacePlannerDto) {
    patch(format!("/plans/{}", id), plan)
}
