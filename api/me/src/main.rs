use jwt_compact::UntrustedToken;
use lambda_http::{service_fn, Error, IntoResponse, Request};
use lambda_http::http::header::{AUTHORIZATION};
use sqlx::PgPool;
use data_access::user::Users;
use endurance_racing_planner_common::GoogleOpenIdClaims;
use utilities::{ApiResponse, not_found_response, unauthorized_response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| {
        me(event, db_context_ref)
    };

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn me(event: Request, db_context_ref: &PgPool) -> Result<impl IntoResponse, Error> {
    let auth_header = event.headers().get(AUTHORIZATION);
    if let Some(header_value) = auth_header {
        let token = header_value.to_str()?.replace("Bearer ", "");
        let parsed_token = UntrustedToken::new(&token).unwrap();
        let claims = parsed_token.deserialize_claims_unchecked::<GoogleOpenIdClaims>().unwrap();
        let oauth_id = claims.custom.sub;

        let user = Users::get_user_by_oauth_id(db_context_ref, oauth_id).await.unwrap();

        return match user {
            Some(u) => Ok(ApiResponse { body: u }.into_response()),
            None => Ok(not_found_response())
        }
    }

    Ok(unauthorized_response())
}