use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("user_plans", |t| {
        t.inject_custom("DROP CONSTRAINT user_plans_plan_id_key");
        t.inject_custom("DROP CONSTRAINT user_plans_plan_id_fkey");
    });

    m.change_table("user_plans", |t| {
        t.rename_column("plan_id", "plan_id_old");
    });

    m.change_table("user_plans", |t| {
        t.add_column("plan_id", types::uuid().unique(false).nullable(true));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);
    });

    m.inject_custom("UPDATE user_plans SET plan_id = plan_id_old");

    m.change_table("user_plans", |t| {
        t.drop_column("plan_id_old");
        t.inject_custom("ALTER COLUMN plan_id SET NOT NULL");
    });

    m.make::<Pg>()
}
