use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("event_configs", |t| {
        t.add_column("plan_id", types::uuid().nullable(false));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);
        t.set_primary_key(&["plan_id"]);

        t.add_column("race_duration", crate::types::interval().nullable(false));
        t.add_column(
            "session_start_utc",
            crate::types::datetime_with_timezone().nullable(false),
        );
        t.add_column("race_start_tod", types::datetime().nullable(false));
        t.add_column(
            "green_flag_offset",
            crate::types::interval().nullable(false),
        );
    });

    m.make::<Pg>()
}
