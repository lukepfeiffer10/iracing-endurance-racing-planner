pub mod entities;
pub mod plans;
pub mod schedules;
pub mod user;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn initialize() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://race_planner:RacingPlanner!2@planner-db/race_planner")
        .await?;

    Ok(pool)
}
