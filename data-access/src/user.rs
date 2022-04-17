use sqlx::{PgPool};
use endurance_racing_planner_common::user::User;

pub struct Users;

impl Users {    
    pub async fn get_user_by_id(pool: &PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
        let user: Option<User> = sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE id = $1",
                id)
            .fetch_optional(pool).await?;
        
        Ok(user)
    }

    pub async fn get_user_by_oauth_id(pool: &PgPool, oauth_id: String) -> Result<Option<User>, sqlx::Error> {
        let user: Option<User> = sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE oauth_id = $1",
                oauth_id)
            .fetch_optional(pool).await?;

        Ok(user)
    }
    
    pub async fn create_user(pool: &PgPool, user: User) -> Result<User, sqlx::Error> {
        let user: User = sqlx::query_as!(
                User,
                r#"INSERT INTO users (name, email, oauth_id) VALUES ($1, $2, $3)
                RETURNING *"#,
                user.name,
                user.email,
                user.oauth_id)
            .fetch_one(pool).await?;
        
        Ok(user)
    }
}