use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("fuel_stint_average_times", |t| {
        t.add_column("plan_id", types::uuid().nullable(false).unique(false));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);
        t.set_primary_key(&["plan_id", "stint_type"]);

        t.add_column("lap_time", crate::types::interval().nullable(false));
        t.add_column("fuel_per_lap", crate::types::real().nullable(false));
        t.add_column("lap_count", types::integer().nullable(false));
        t.add_column(
            "lap_time_with_pit",
            crate::types::interval().nullable(false),
        );
        t.add_column("track_time", crate::types::interval().nullable(false));
        t.add_column(
            "track_time_with_pit",
            crate::types::interval().nullable(false),
        );
        t.add_column("fuel_per_stint", crate::types::real().nullable(false));
        t.add_column("stint_type", crate::types::smallint().nullable(false));
    });

    m.make::<Pg>()
}
