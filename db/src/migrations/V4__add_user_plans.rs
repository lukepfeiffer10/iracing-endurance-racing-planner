use barrel::types::ReferentialAction;
use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("user_plans", |t| {
        t.add_column(
            "user_id",
            types::foreign(
                "users",
                "id",
                ReferentialAction::NoAction,
                ReferentialAction::NoAction,
            ),
        );
        t.add_column("plan_id", types::uuid());
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);
        t.set_primary_key(&["user_id", "plan_id"]);
    });

    m.make::<Pg>()
}
