use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use endurance_racing_planner_common::{PatchRacePlannerDto, PlanListDto, RacePlannerDto};
use sqlx::{types::Uuid, PgPool};

use crate::{
    data_access::{
        self,
        entities::{
            plan::{PatchPlan, PatchPlanType, StintType},
            Plan,
        },
        plans::{create_plan, get_plan_by_id, get_plans_by_user_id},
    },
    AuthenticatedUser,
};

pub(crate) async fn add_plan(
    AuthenticatedUser(user): AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(plan): Json<RacePlannerDto>,
) -> impl IntoResponse {
    let mut new_plan: Plan = plan.into();
    new_plan.created_by = user.id;

    let new_plan_result = create_plan(&pool, new_plan).await;
    if let Ok(new_plan) = new_plan_result {
        (
            StatusCode::CREATED,
            [(header::CONTENT_LOCATION, format!("/plans/{}", new_plan.id))],
            Json::<RacePlannerDto>(new_plan.into()),
        )
            .into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save the plan").into_response()
    }
}

pub(crate) async fn get_plans(
    AuthenticatedUser(user): AuthenticatedUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let plans = get_plans_by_user_id(&pool, user.id).await.map(|plans| {
        plans
            .iter()
            .map(|p| PlanListDto {
                id: p.id,
                title: p.title.clone(),
                owner: p.owner.clone(),
                last_modified: p.modified_date.unwrap_or(p.created_date),
            })
            .collect::<Vec<PlanListDto>>()
    });

    match plans {
        Ok(plans) => Json(plans).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load the plans",
        )
            .into_response(),
    }
}

pub(crate) async fn get_plan(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> impl IntoResponse {
    get_plan_by_id(&pool, id, user.id)
        .await
        .map(|plan| match plan {
            Some(plan) => Json(plan).into_response(),
            None => (StatusCode::NOT_FOUND).into_response(),
        })
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to get the plan").into_response()
        })
}

pub(crate) async fn patch_plan(
    AuthenticatedUser(user): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(plan): Json<PatchRacePlannerDto>,
) -> impl IntoResponse {
    let patch = if plan.overall_event_config.is_some() {
        PatchPlanType::EventConfig(plan.overall_event_config.unwrap())
    } else if plan.overall_fuel_stint_config.is_some() {
        PatchPlanType::FuelStintConfig(plan.overall_fuel_stint_config.unwrap())
    } else if plan.fuel_stint_average_times.is_some() {
        let fuel_stint_average_times = plan.fuel_stint_average_times.unwrap();
        let stint_type = if fuel_stint_average_times.standard_fuel_stint.is_some() {
            StintType::Standard
        } else {
            StintType::FuelSaving
        };
        PatchPlanType::FuelStintAverageTime(
            fuel_stint_average_times
                .standard_fuel_stint
                .or(fuel_stint_average_times.fuel_saving_stint)
                .unwrap(),
            stint_type,
        )
    } else if plan.title.is_some() {
        PatchPlanType::Title(plan.title.unwrap())
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json("failed to supply any values to patch".to_string()),
        );
    };

    let result = data_access::plans::patch_plan(&pool, PatchPlan::new(id, user.id, patch)).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(id.to_string())),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("something went wrong saving the plan".to_string()),
        ),
    }
}

pub(crate) async fn share_plan(
    AuthenticatedUser(_): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(emails): Json<Vec<String>>,
) -> impl IntoResponse {
    match data_access::user::Users::get_users_by_emails(&pool, &emails).await {
        Ok(users) => {
            let user_ids = users.iter().map(|user| user.id).collect::<Vec<_>>();
            if user_ids.is_empty() {
                return StatusCode::OK;
            }
            match data_access::plans::add_users_to_plan(&pool, id, &user_ids).await {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn get_plan_shared_users(
    AuthenticatedUser(_): AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match data_access::user::Users::get_shared_users_by_plan_id(&pool, id).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Something went wrong getting shared users. Please try again later.".to_string()),
        )
            .into_response(),
    }
}
