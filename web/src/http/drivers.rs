use endurance_racing_planner_common::Driver;
use uuid::Uuid;
use yew::Callback;

use super::{get_async, post, put, CustomError};

pub fn create_plan_driver(plan_id: Uuid, driver: Driver, callback: Callback<Driver>) -> () {
    post(
        format!("/plans/{}/drivers", plan_id).into(),
        driver,
        Some(callback),
    )
}

pub async fn get_plan_drivers_async(plan_id: Uuid) -> Result<Vec<Driver>, CustomError> {
    get_async(format!("/plans/{}/drivers", plan_id).into()).await
}

pub fn update_driver(driver: Driver) -> () {
    put(format!("/drivers/{}", driver.id), driver)
}
