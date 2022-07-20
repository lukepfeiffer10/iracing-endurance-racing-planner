use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("fuel_stint_configs", |t| {
        t.add_column("plan_id", types::uuid().nullable(false).unique(true));
        t.add_foreign_key(&["plan_id"], "plans", &["id"]);
        t.set_primary_key(&["plan_id"]);

        t.add_column("pit_duration", crate::types::interval().nullable(false));
        t.add_column("fuel_tank_size", types::integer().nullable(false));
        t.add_column("tire_change_time", crate::types::interval().nullable(false));
        t.add_column("add_tire_time", types::boolean().nullable(false));
    });

    m.make::<Pg>()
}
