use endurance_racing_planner_common::schedule::ScheduleStintDto;
use uuid::Uuid;

use super::{get_async, post, put, CustomError};

pub fn create_schedule(plan_id: Uuid, schedule: Vec<ScheduleStintDto>) -> () {
    post::<Vec<ScheduleStintDto>, ()>(format!("/plans/{}/schedule", plan_id), schedule, None)
}

pub async fn get_schedule_async(plan_id: Uuid) -> Result<Vec<ScheduleStintDto>, CustomError> {
    get_async(format!("plans/{}/schedule", plan_id)).await
}

pub fn update_schedule(plan_id: Uuid, schedule: Vec<ScheduleStintDto>) -> () {
    put::<Vec<ScheduleStintDto>>(format!("/plans/{}/schedule", plan_id), schedule)
}
