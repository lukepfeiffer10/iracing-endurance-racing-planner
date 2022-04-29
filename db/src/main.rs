mod migrations;
mod types;

use postgres::{Client, NoTls };

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations");
}

fn main() {
    let mut conn = Client::connect("host=localhost user=race_planner password=RacingPlanner!2 dbname=race_planner", NoTls).unwrap();
    embedded::migrations::runner().run(&mut conn).unwrap();
}
