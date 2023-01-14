use endurance_racing_planner_common::schedule::ScheduleStintDto;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::entities::schedule::{Stint, StintType};

pub async fn get_schedule_by_plan_id(
    pool: &PgPool,
    plan_id: Uuid,
) -> Result<Vec<ScheduleStintDto>, sqlx::Error> {
    let stints = sqlx::query_as!(
        Stint,
        r#"select 
            id
            ,stint_type as "stint_type: StintType"
            ,"number"
            ,utc_start
            ,utc_end
            ,tod_start
            ,tod_end
            ,actual_end
            ,duration_delta
            ,damage_modifier
            ,calculated_laps
            ,actual_laps
            ,driver_stint_count
            ,driver_id
            from public.stints s
            WHERE s.plan_id = $1
            ORDER BY s.number"#,
        plan_id
    )
    .fetch_all(pool)
    .await?;

    Ok(stints
        .iter()
        .map(|s| s.into())
        .collect::<Vec<ScheduleStintDto>>())
}

pub async fn create_schedule(
    pool: &PgPool,
    plan_id: Uuid,
    schedule: Vec<Stint>,
) -> Result<bool, sqlx::Error> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO stints (id, plan_id, stint_type, number, utc_start, utc_end, tod_start, tod_end, actual_end, duration_delta, damage_modifier, calculated_laps, actual_laps, driver_stint_count, driver_id) "
    );

    query_builder.push_values(schedule.iter(), |mut builder, stint_data| {
        builder
            .push_bind(stint_data.id)
            .push_bind(plan_id)
            .push_bind(stint_data.stint_type.clone() as i16)
            .push_bind(stint_data.number)
            .push_bind(stint_data.utc_start)
            .push_bind(stint_data.utc_end)
            .push_bind(stint_data.tod_start)
            .push_bind(stint_data.tod_end)
            .push_bind(stint_data.actual_end)
            .push_bind(stint_data.duration_delta.clone())
            .push_bind(stint_data.damage_modifier.clone())
            .push_bind(stint_data.calculated_laps)
            .push_bind(stint_data.actual_laps)
            .push_bind(stint_data.driver_stint_count)
            .push_bind(stint_data.driver_id);
    });

    let query = query_builder.build();
    query.execute(pool).await?;

    Ok(true)
}

pub async fn update_schedule(pool: &PgPool, schedule: Vec<Stint>) -> Result<(), sqlx::Error> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "UPDATE stints as s SET
                stint_type = u.stint_type,
                number = u.number,
                utc_start = u.utc_start,
                utc_end = u.utc_end,
                tod_start = u.tod_start,
                tod_end = u.tod_end,
                actual_end = u.actual_end,
                duration_delta = u.duration_delta,
                damage_modifier = u.damage_modifier,
                calculated_laps = u.calculated_laps,
                actual_laps = u.actual_laps,
                driver_stint_count = u.driver_stint_count,
                driver_id = u.driver_id
            FROM (",
    );

    query_builder.push_values(schedule.iter(), |mut builder, stint| {
        builder
            .push_bind(stint.id)
            .push_bind(stint.stint_type.clone() as i16)
            .push_bind(stint.number)
            .push_bind(stint.utc_start)
            .push_bind(stint.utc_end)
            .push_bind(stint.tod_start)
            .push_bind(stint.tod_end)
            .push_bind(stint.actual_end)
            .push_bind(stint.duration_delta.clone())
            .push_bind(stint.damage_modifier.clone())
            .push_bind(stint.calculated_laps)
            .push_bind(stint.actual_laps)
            .push_bind(stint.driver_stint_count)
            .push_bind(stint.driver_id);
    });

    query_builder.push(
        ") as u (id,
                stint_type,
                number,
                utc_start,
                utc_end,
                tod_start,
                tod_end,
                actual_end,
                duration_delta,
                damage_modifier,
                calculated_laps,
                actual_laps,
                driver_stint_count,
                driver_id
            )
            where s.id = u.id",
    );

    let query = query_builder.build();
    query.execute(pool).await?;

    Ok(())
}
