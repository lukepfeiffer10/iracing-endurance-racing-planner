use barrel::types::ReferentialAction;
use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("plans", |t| {
        t.add_column("id", types::uuid());
        t.add_column("title", types::varchar(255));
        t.add_column(
            "created_by",
            types::foreign(
                "users",
                "id",
                ReferentialAction::NoAction,
                ReferentialAction::NoAction,
            ),
        );
        t.add_column("created_date", crate::types::datetime_with_timezone());
    });

    m.make::<Pg>()
}
