use endurance_racing_planner_common::User;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

pub struct Users;

impl Users {
    pub async fn get_user_by_id(pool: &PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
        let user: Option<User> = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_oauth_id(
        pool: &PgPool,
        oauth_id: String,
    ) -> Result<Option<User>, sqlx::Error> {
        let user: Option<User> =
            sqlx::query_as!(User, "SELECT * FROM users WHERE oauth_id = $1", oauth_id)
                .fetch_optional(pool)
                .await?;

        Ok(user)
    }

    pub async fn get_users_by_emails(
        pool: &PgPool,
        emails: &Vec<String>,
    ) -> Result<Vec<User>, sqlx::Error> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM users WHERE email IN (");
        let mut in_clause = query_builder.separated(", ");
        for email in emails {
            in_clause.push_bind(email);
        }
        in_clause.push_unseparated(");");

        query_builder
            .build()
            .try_map(|row| {
                Ok(User {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    email: row.try_get("email")?,
                    oauth_id: row.try_get("oauth_id")?,
                })
            })
            .fetch_all(pool)
            .await
    }

    pub async fn get_shared_users_by_plan_id(
        pool: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<User>, sqlx::Error> {
        let users: Vec<User> = sqlx::query_as!(
            User,
            r#"SELECT u.* FROM users u 
                INNER JOIN user_plans up ON up.user_id = u.id
                INNER JOIN plans p ON up.plan_id = p.id AND p.created_by != up.user_id
            WHERE up.plan_id = $1"#,
            plan_id
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn create_user(pool: &PgPool, user: User) -> Result<User, sqlx::Error> {
        let user: User = sqlx::query_as!(
            User,
            r#"INSERT INTO users (name, email, oauth_id) VALUES ($1, $2, $3)
                RETURNING *"#,
            user.name,
            user.email,
            user.oauth_id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
