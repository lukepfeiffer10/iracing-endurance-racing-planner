use sqlx::PgPool;
use sqlx::types::Uuid;
use crate::entities::Plan;

pub async fn get_plan_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Plan>, sqlx::Error> {
    let plan: Option<Plan> = sqlx::query_as!(
                Plan,
                "SELECT * FROM plans WHERE id = $1",
                id)
        .fetch_optional(pool).await?;

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
                plan.created_date)
        .fetch_one(pool).await?;

    Ok(plan)
}