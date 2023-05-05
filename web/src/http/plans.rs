use endurance_racing_planner_common::{PatchRacePlannerDto, PlanListDto, RacePlannerDto, User};
use uuid::Uuid;
use yew::Callback;

use super::{get, get_async, patch, post, CustomError};

static PLANS_BASE_ROUTE: &str = "plans";

pub fn create_plan(plan: RacePlannerDto, callback: Callback<RacePlannerDto>) {
    post(PLANS_BASE_ROUTE.into(), plan, Some(callback))
}

pub fn get_plans(callback: Callback<Vec<PlanListDto>>) {
    get(PLANS_BASE_ROUTE.into(), callback)
}

pub async fn get_plan_async(id: Uuid) -> Result<RacePlannerDto, CustomError> {
    get_async(format!("{}/{}", PLANS_BASE_ROUTE, id)).await
}

pub fn patch_plan(id: Uuid, plan: PatchRacePlannerDto) {
    patch(format!("{}/{}", PLANS_BASE_ROUTE, id), plan)
}

pub fn share_plan(id: Uuid, emails: Vec<String>) {
    post::<Vec<String>, ()>(format!("{}/{}/share", PLANS_BASE_ROUTE, id), emails, None)
}

pub fn get_shared_users_for_plan(id: Uuid, callback: Callback<Vec<User>>) {
    get(format!("{}/{}/share", PLANS_BASE_ROUTE, id), callback)
}
