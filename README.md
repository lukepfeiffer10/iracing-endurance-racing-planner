# Developing
1. Use `docker compose up` to create the database
1. Run `cargo sqlx database setup` to apply the database migrations
1. Run the api with `cargo run`
1. Run the web application
    ```sh
    cd web
    npm start
    ```
1. This should launch your default browser at `localhost:9000`