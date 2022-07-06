use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("plans", |t| {
        t.add_column("modified_by", types::integer().nullable(true));

        t.add_column(
            "modified_date",
            crate::types::datetime_with_timezone().nullable(true),
        );

        t.add_foreign_key(&["modified_by"], "users", &["id"]);
    });

    m.make::<Pg>()
}
