use barrel::{backend::Pg, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("drivers", |t| {
        t.add_column("lap_time", crate::types::interval().nullable(true));
    });

    m.make::<Pg>()
}
