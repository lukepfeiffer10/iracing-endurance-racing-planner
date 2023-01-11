pub mod entities;
pub mod plans;
pub mod schedules;
pub mod user;

use std::env;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn initialize() -> Result<PgPool, sqlx::Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable to be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(db_url.as_str())
        .expect("database to exist");

    Ok(pool)
}
