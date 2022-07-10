use crate::entities::plan::{PatchPlan, PlanWithOverview, PlanWithOwner};
use crate::entities::Plan;
use sqlx::postgres::types::PgInterval;
use sqlx::types::Uuid;
use sqlx::PgPool;

pub async fn get_plan_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<PlanWithOverview>, sqlx::Error> {
    let plan: Option<PlanWithOverview> = sqlx::query_as!(
        PlanWithOverview,
        r#"SELECT p.id, p.title, 
                ec.race_duration as "race_duration: Option<_>", 
                ec.session_start_utc as "session_start_utc: Option<_>", 
                ec.race_start_tod as "race_start_tod: Option<_>", 
                ec.green_flag_offset as "green_flag_offset: Option<_>" 
            FROM plans p 
                LEFT OUTER JOIN event_configs ec ON ec.plan_id = p.id
            WHERE p.id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(plan)
}

pub async fn create_plan(pool: &PgPool, plan: Plan) -> Result<Plan, sqlx::Error> {
    let plan: Plan = sqlx::query_as!(
        Plan,
        r#"INSERT INTO plans (id, title, created_by, created_date) VALUES ($1, $2, $3, $4)
                RETURNING *"#,
        plan.id,
        plan.title,
        plan.created_by,
        plan.created_date
    )
    .fetch_one(pool)
    .await?;

    sqlx::query!(
        r#"INSERT INTO user_plans (user_id, plan_id) VALUES ($1, $2)"#,
        plan.created_by,
        plan.id
    )
    .execute(pool)
    .await?;

    Ok(plan)
}

pub async fn get_plans_by_user_id(
    pool: &PgPool,
    user_id: i32,
) -> Result<Vec<PlanWithOwner>, sqlx::Error> {
    let plans: Vec<PlanWithOwner> = sqlx::query_as!(
        PlanWithOwner,
        r#"SELECT p.*, u.name as owner FROM plans p 
            INNER JOIN user_plans up ON up.plan_id = p.id 
            INNER JOIN users u ON u.id = p.created_by
            WHERE up.user_id = $1"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(plans)
}

pub async fn patch_plan(pool: &PgPool, id: Uuid, plan: PatchPlan) -> bool {
    match plan {
        PatchPlan::Title(title) => {
            let result = sqlx::query!(r#"UPDATE plans SET title = $1 WHERE id = $2"#, title, id)
                .execute(pool)
                .await;

            match result {
                Ok(query_result) => query_result.rows_affected() == 1,
                Err(_) => false,
            }
        }
        PatchPlan::EventConfig(data) => {
            let race_duration: PgInterval = data.race_duration.try_into().unwrap();
            let green_flag_offset: PgInterval = data.green_flag_offset.try_into().unwrap();
            let result = sqlx::query!(
                r#"
                UPDATE event_configs SET 
                    race_duration = $1, 
                    session_start_utc = $2, 
                    race_start_tod = $3, 
                    green_flag_offset = $4
                WHERE plan_id = $5"#,
                race_duration,
                data.session_start_utc,
                data.race_start_tod,
                green_flag_offset,
                id
            )
            .execute(pool)
            .await;

            match result {
                Ok(query_result) => query_result.rows_affected() == 1,
                Err(_) => false,
            }
        }
    }
}
