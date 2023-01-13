use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("stints", |t| {
        t.add_column("driver_id", types::integer().nullable(true));
    });

    m.change_table("stints", |t| {
        t.add_foreign_key(&["driver_id"], "drivers", &["id"]);
    });

    m.make::<Pg>()
}
