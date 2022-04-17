use std::str::FromStr;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use serde_json::json;
use sqlx::{PgPool};
use data_access::user::Users;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| {        
        func(event, db_context_ref)
    };
    
    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn func(event: Request, db_context: &PgPool) -> Result<impl IntoResponse, Error> {
    Ok(match event.path_parameters().first("id") {
        Some(id) => {
            let result = Users::get_user_by_id(db_context, i32::from_str(id)?).await;
            match result { 
                Ok(value) => serde_json::to_value(value)?.into_response(),
                Err(e) => json!({"error": e.to_string()}).into_response()
            }
        },
        _ => Response::builder()
            .status(400)
            .body("Empty id".into())
            .expect("failed to render response"),
    })
}