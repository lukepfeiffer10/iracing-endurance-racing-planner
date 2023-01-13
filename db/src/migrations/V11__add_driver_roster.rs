use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("drivers", |t| {
        t.add_column("id", types::primary());
        t.add_column("plan_id", types::uuid().nullable(false).unique(false));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);

        t.add_column("name", types::varchar(100).nullable(false));
        t.add_column("color", types::varchar(15).nullable(false));
        t.add_column(
            "utc_offset",
            crate::types::smallint().nullable(false).default(0),
        );
        t.add_column(
            "irating",
            crate::types::smallint().nullable(false).default(0),
        );
        t.add_column(
            "stint_preference",
            crate::types::smallint().nullable(false).default(0),
        );
    });

    m.make::<Pg>()
}
