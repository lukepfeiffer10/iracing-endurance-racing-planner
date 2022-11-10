use crate::entities::plan::{
    FuelStintAverageTimes, PatchPlan, PatchPlanType, PlanWithOverview, PlanWithOwner, StintType,
};
use crate::entities::Plan;
use chrono::{DateTime, Duration, Utc};
use endurance_racing_planner_common::{
    EventConfigDto, OverallFuelStintConfigData, RacePlannerDto, StintDataDto,
};
use futures::try_join;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::types::Uuid;
use sqlx::{PgPool, Postgres};

pub async fn get_plan_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<RacePlannerDto>, sqlx::Error> {
    let plan = sqlx::query_as!(
        PlanWithOverview,
        r#"SELECT p.id, p.title, 
                ec.race_duration as "race_duration: Option<_>", 
                ec.session_start_utc as "session_start_utc: Option<_>", 
                ec.race_start_utc as "race_start_utc: Option<_>", 
                ec.race_end_utc as "race_end_utc: Option<_>", 
                ec.race_start_tod as "race_start_tod: Option<_>",
                ec.race_end_tod as "race_end_tod: Option<_>", 
                ec.green_flag_offset as "green_flag_offset: Option<_>", 
                ec.tod_offset as "tod_offset: Option<_>", 
                fsc.pit_duration as "pit_duration: Option<_>", 
                fsc.fuel_tank_size as "fuel_tank_size: Option<_>", 
                fsc.tire_change_time as "tire_change_time: Option<_>", 
                fsc.add_tire_time as "add_tire_time: Option<_>" 
            FROM plans p 
                LEFT OUTER JOIN event_configs ec ON ec.plan_id = p.id
                LEFT OUTER JOIN fuel_stint_configs fsc ON fsc.plan_id = p.id
            WHERE p.id = $1"#,
        id
    )
    .fetch_optional(pool);

    let fuel_stint_average_times = sqlx::query_as!(
        FuelStintAverageTimes,
        r#"SELECT
            plan_id,
            lap_time,
            fuel_per_lap,
            lap_count,
            lap_time_with_pit,
            track_time,
            track_time_with_pit,
            fuel_per_stint,
            stint_type as "stint_type: StintType"
        FROM fuel_stint_average_times
        WHERE plan_id = $1"#,
        id
    )
    .fetch_all(pool);

    let (plan, fuel_stint_average_times) = try_join!(plan, fuel_stint_average_times)?;

    let fuel_stint_average_times = if !fuel_stint_average_times.is_empty() {
        let mut stints = fuel_stint_average_times.into_iter();
        Some(endurance_racing_planner_common::FuelStintAverageTimes {
            standard_fuel_stint: stints
                .find(|s| match s.stint_type {
                    StintType::Standard => true,
                    StintType::FuelSaving => false,
                })
                .map(|f| StintDataDto {
                    lap_time: Duration::microseconds(f.lap_time.microseconds),
                    fuel_per_lap: f.fuel_per_lap,
                    lap_count: f.lap_count,
                    lap_time_with_pit: Duration::microseconds(f.lap_time_with_pit.microseconds),
                    track_time: Duration::microseconds(f.track_time.microseconds),
                    track_time_with_pit: Duration::microseconds(f.track_time_with_pit.microseconds),
                    fuel_per_stint: f.fuel_per_stint,
                })
                .expect("standard fuel stint average times"),
            fuel_saving_stint: stints
                .find(|s| match s.stint_type {
                    StintType::Standard => false,
                    StintType::FuelSaving => true,
                })
                .map(|f| StintDataDto {
                    lap_time: Duration::microseconds(f.lap_time.microseconds),
                    fuel_per_lap: f.fuel_per_lap,
                    lap_count: f.lap_count,
                    lap_time_with_pit: Duration::microseconds(f.lap_time_with_pit.microseconds),
                    track_time: Duration::microseconds(f.track_time.microseconds),
                    track_time_with_pit: Duration::microseconds(f.track_time_with_pit.microseconds),
                    fuel_per_stint: f.fuel_per_stint,
                })
                .expect("fuel saving fuel stint average times"),
        })
    } else {
        None
    };
    let dto = plan.map(|p| RacePlannerDto {
        id: p.id,
        title: p.title,
        overall_event_config: p.race_duration.map(|race_duration| EventConfigDto {
            race_duration: Duration::microseconds(race_duration.microseconds),
            session_start_utc: p.session_start_utc.unwrap(),
            race_start_tod: p.race_start_tod.unwrap(),
            green_flag_offset: Duration::microseconds(p.green_flag_offset.unwrap().microseconds),
            race_start_utc: p.race_start_utc.unwrap(),
            race_end_utc: p.race_end_utc.unwrap(),
            race_end_tod: p.race_end_tod.unwrap(),
            tod_offset: Duration::microseconds(p.tod_offset.unwrap().microseconds),
        }),
        overall_fuel_stint_config: p
            .pit_duration
            .map(|pit_duration| OverallFuelStintConfigData {
                pit_duration: Duration::microseconds(pit_duration.microseconds),
                fuel_tank_size: p.fuel_tank_size.unwrap(),
                tire_change_time: Duration::microseconds(p.tire_change_time.unwrap().microseconds),
                add_tire_time: p.add_tire_time.unwrap(),
            }),
        fuel_stint_average_times,
        time_of_day_lap_factors: vec![],
        per_driver_lap_factors: vec![],
        driver_roster: vec![],
        schedule_rows: None,
    });

    Ok(dto)
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

pub async fn patch_plan(pool: &PgPool, plan: PatchPlan) -> Result<bool, sqlx::Error> {
    match plan.patch_type {
        PatchPlanType::Title(title) => {
            let result = sqlx::query!(
                r#"UPDATE plans SET title = $1, modified_by = $2, modified_date = $3 WHERE id = $4"#, 
                    title, plan.modified_by, plan.modified_date, plan.id)
                .execute(pool)
                .await;

            Ok(match result {
                Ok(query_result) => query_result.rows_affected() == 1,
                Err(_) => false,
            })
        }
        PatchPlanType::EventConfig(data) => {
            let transaction = pool.begin().await?;

            let race_duration: PgInterval = data.race_duration.try_into().unwrap();
            let green_flag_offset: PgInterval = data.green_flag_offset.try_into().unwrap();
            let tod_offset: PgInterval = data.tod_offset.try_into().unwrap();
            let upsert_event_config = sqlx::query!(
                r#"
                INSERT INTO event_configs AS ec (plan_id, race_duration, session_start_utc, race_start_utc, race_end_utc, race_start_tod, race_end_tod, green_flag_offset, tod_offset)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (plan_id) DO UPDATE 
                SET 
                    race_duration = $2, 
                    session_start_utc = $3, 
                    race_start_utc = $4,
                    race_end_utc = $5,
                    race_start_tod = $6,
                    race_end_tod = $7, 
                    green_flag_offset = $8,
                    tod_offset = $9
                WHERE ec.plan_id = $1"#,                
                plan.id,
                race_duration,
                data.session_start_utc,
                data.race_start_utc,
                data.race_end_utc,
                data.race_start_tod,
                data.race_end_tod,
                green_flag_offset,
                tod_offset
            )
            .execute(pool);

            let result = try_join!(
                upsert_event_config,
                update_plan_modified_by(plan.id, plan.modified_by, plan.modified_date)
                    .execute(pool)
            );

            Ok(match result {
                Ok((upsert_result, update_plan_result)) => {
                    transaction.commit().await?;
                    upsert_result.rows_affected() == 1 && update_plan_result.rows_affected() == 1
                }
                Err(_) => {
                    transaction.rollback().await?;
                    false
                }
            })
        }
        PatchPlanType::FuelStintConfig(config) => {
            let transaction = pool.begin().await?;

            let pit_duration: PgInterval = config.pit_duration.try_into().unwrap();
            let tire_change_time: PgInterval = config.tire_change_time.try_into().unwrap();
            let upsert_fuel_stint_config = sqlx::query!(
                r#"
                INSERT INTO fuel_stint_configs AS fsc (plan_id, pit_duration, fuel_tank_size, tire_change_time, add_tire_time)
                VALUES ($5, $1, $2, $3, $4)
                ON CONFLICT (plan_id) DO UPDATE 
                SET 
                    pit_duration = $1, 
                    fuel_tank_size = $2, 
                    tire_change_time = $3, 
                    add_tire_time = $4
                WHERE fsc.plan_id = $5"#,
                pit_duration,
                config.fuel_tank_size,
                tire_change_time,
                config.add_tire_time,
                plan.id
            )
            .execute(pool);

            let result = try_join!(
                upsert_fuel_stint_config,
                update_plan_modified_by(plan.id, plan.modified_by, plan.modified_date)
                    .execute(pool)
            );

            Ok(match result {
                Ok((upsert_result, update_plan_result)) => {
                    transaction.commit().await?;
                    upsert_result.rows_affected() == 1 && update_plan_result.rows_affected() == 1
                }
                Err(_) => {
                    transaction.rollback().await?;
                    false
                }
            })
        }
        PatchPlanType::FuelStintAverageTime(data, stint_type) => {
            let transaction = pool.begin().await?;

            let lap_time: PgInterval = data.lap_time.try_into().unwrap();
            let lap_time_with_pit: PgInterval = data.lap_time_with_pit.try_into().unwrap();
            let track_time: PgInterval = data.track_time.try_into().unwrap();
            let track_time_with_pit: PgInterval = data.track_time_with_pit.try_into().unwrap();
            let upsert_stint_data = sqlx::query!(
                r#"
                INSERT INTO fuel_stint_average_times AS fs
                    (plan_id,
                    lap_time,
                    fuel_per_lap,
                    lap_count,
                    lap_time_with_pit,
                    track_time,
                    track_time_with_pit,
                    fuel_per_stint,
                    stint_type)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (plan_id, stint_type) DO UPDATE 
                SET 
                    lap_time = $2,
                    fuel_per_lap = $3,
                    lap_count = $4,
                    lap_time_with_pit = $5,
                    track_time = $6,
                    track_time_with_pit = $7,
                    fuel_per_stint = $8
                WHERE fs.plan_id = $1 AND fs.stint_type = $9"#,
                plan.id,
                lap_time,
                data.fuel_per_lap,
                data.lap_count,
                lap_time_with_pit,
                track_time,
                track_time_with_pit,
                data.fuel_per_stint,
                stint_type as i16,
            );

            let result = try_join!(
                upsert_stint_data.execute(pool),
                update_plan_modified_by(plan.id, plan.modified_by, plan.modified_date)
                    .execute(pool)
            );

            match result {
                Ok(_) => {
                    transaction.commit().await?;
                    Ok(true)
                }
                Err(_) => {
                    transaction.rollback().await?;
                    Ok(false)
                }
            }
        }
    }
}

fn update_plan_modified_by(
    id: Uuid,
    modified_by: i32,
    modified_date: DateTime<Utc>,
) -> Query<'static, Postgres, PgArguments> {
    sqlx::query!(
        "UPDATE plans SET modified_by = $1, modified_date = $2 WHERE id = $3",
        modified_by,
        modified_date,
        id
    )
}
