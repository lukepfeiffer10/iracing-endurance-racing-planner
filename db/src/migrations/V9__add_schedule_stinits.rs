use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("stints", |t| {
        t.add_column("id", types::uuid().nullable(false).primary(true));
        t.add_column("plan_id", types::uuid().nullable(false).unique(false));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);

        t.add_column("stint_type", crate::types::smallint().nullable(false));
        t.add_column("number", types::integer().nullable(false));
        t.add_column(
            "utc_start",
            crate::types::datetime_with_timezone().nullable(false),
        );
        t.add_column(
            "utc_end",
            crate::types::datetime_with_timezone().nullable(false),
        );
        t.add_column("tod_start", types::datetime().nullable(false));
        t.add_column("tod_end", types::datetime().nullable(false));
        t.add_column(
            "actual_end",
            crate::types::datetime_with_timezone().nullable(false),
        );
        t.add_column("duration_delta", crate::types::interval().nullable(false));
        t.add_column("damage_modifier", crate::types::interval().nullable(false));
        t.add_column("calculated_laps", types::integer().nullable(false));
        t.add_column("actual_laps", types::integer().nullable(false));
        t.add_column("driver_stint_count", types::integer().nullable(false));

        // TODO: add driver references
    });

    m.make::<Pg>()
}
