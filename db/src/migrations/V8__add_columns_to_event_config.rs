use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("event_configs", |t| {
        t.add_column(
            "race_start_utc",
            crate::types::datetime_with_timezone()
                .nullable(false)
                .default("now()"),
        );
        t.add_column(
            "race_end_utc",
            crate::types::datetime_with_timezone()
                .nullable(false)
                .default("now()"),
        );
        t.add_column(
            "race_end_tod",
            types::datetime().nullable(false).default("now()"),
        );
        t.add_column(
            "tod_offset",
            crate::types::interval().nullable(false).default("PT0S"),
        );
    });

    m.change_table("event_configs", |t| {
        t.inject_custom("ALTER COLUMN race_start_utc DROP DEFAULT");
        t.inject_custom("ALTER COLUMN race_end_utc DROP DEFAULT");
        t.inject_custom("ALTER COLUMN race_end_tod DROP DEFAULT");
        t.inject_custom("ALTER COLUMN tod_offset DROP DEFAULT");
    });

    m.make::<Pg>()
}
