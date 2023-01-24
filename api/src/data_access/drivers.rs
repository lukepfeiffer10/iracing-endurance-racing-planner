use sqlx::PgPool;
use uuid::Uuid;

use crate::data_access::entities::driver::Driver;

pub async fn get_drivers_by_plan_id(
    pool: &PgPool,
    plan_id: Uuid,
) -> Result<Vec<Driver>, sqlx::Error> {
    let drivers: Vec<Driver> = sqlx::query_as!(
        Driver,
        "SELECT * FROM drivers WHERE plan_id = $1 ORDER BY id",
        plan_id
    )
    .fetch_all(pool)
    .await?;

    Ok(drivers)
}

pub async fn create_driver(pool: &PgPool, driver: Driver) -> Result<Driver, sqlx::Error> {
    let driver: Driver = sqlx::query_as!(
            Driver,
            r#"INSERT INTO drivers (plan_id, name, color, utc_offset, irating, stint_preference) VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *"#,
            driver.plan_id,
            driver.name,
            driver.color,
            driver.utc_offset,
            driver.irating,
            driver.stint_preference
        )
        .fetch_one(pool)
        .await?;

    Ok(driver)
}

pub async fn update_driver(pool: &PgPool, id: i32, driver: Driver) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE drivers 
               SET 
                name = $1,
                color = $2,
                utc_offset = $3,
                irating = $4,
                stint_preference = $5
            WHERE id = $6"#,
        driver.name,
        driver.color,
        driver.utc_offset,
        driver.irating,
        driver.stint_preference,
        id
    )
    .execute(pool)
    .await;

    Ok(match result {
        Ok(query_result) => query_result.rows_affected() == 1,
        Err(_) => false,
    })
}
