use barrel::{backend::Pg, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("users", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::varchar(255));
        t.add_column("email", types::varchar(255));
        t.add_column("oauth_id", types::varchar(255).unique(true));
    });

    m.make::<Pg>()
}
