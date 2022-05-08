use jwt_compact::UntrustedToken;
use lambda_http::http::response::Builder;
use lambda_http::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE, LOCATION};
use lambda_http::{Body, IntoResponse, Response};
use lambda_http::http::{HeaderMap, HeaderValue, StatusCode};
use serde::Serialize;
use sqlx::PgPool;
use data_access::user::Users;
use endurance_racing_planner_common::{GoogleOpenIdClaims, User};

fn add_standard_headers(builder: Builder) -> Builder {
    builder
        .header(CONTENT_TYPE, "application/json")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "http://localhost:9000")
}

pub fn ok_response<T>(response: ApiResponse<T>) -> Response<Body>
    where 
        T: Serialize {
    let builder = Response::builder()
        .status(StatusCode::OK);
    add_standard_headers(builder)
        .body(serde_json::to_string(&response.body)
            .expect("failed to serialize T")
            .into())
        .expect("failed to build the ok response")
}

pub fn created_response<T>(resource: &T, location: String) -> Response<Body> 
    where 
        T: Serialize {
    let builder = Response::builder()
        .status(StatusCode::CREATED)
        .header(LOCATION, location);
    add_standard_headers(builder)
        .body(serde_json::to_string(resource)
            .expect("failed to serialize the created resource")
            .into())
        .expect("failed to build the created response")
}

pub fn unauthorized_response() -> Response<Body> {
    let builder = Response::builder()
        .status(StatusCode::UNAUTHORIZED);
    add_standard_headers(builder)
        .body(().into())
        .expect("failed to build the unauthorized response")
}

pub fn not_found_response() -> Response<Body> {
    let builder = Response::builder()
        .status(StatusCode::NOT_FOUND);
    add_standard_headers(builder)
        .body(().into())
        .expect("failed to build the not found response")
}

pub fn bad_request_response(error: String) -> Response<Body> {
    let builder = Response::builder()
        .status(StatusCode::BAD_REQUEST);
    add_standard_headers(builder)
        .body(error.into())
        .expect("failed to build the bad request response")
}

pub struct ApiResponse<T>
    where 
        T: Serialize {
    pub body: T
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response<Body> {
        ok_response(self)
    }
}

pub async fn get_current_user(headers: &HeaderMap<HeaderValue>, db_context_ref: &PgPool) -> Option<User> {
    let auth_header = headers.get(AUTHORIZATION);
    match auth_header {
        Some(header_value) => {
            let token = header_value.to_str()
                .expect("authorization header as a string")
                .replace("Bearer ", "");
            let parsed_token = UntrustedToken::new(&token).unwrap();
            let claims = parsed_token.deserialize_claims_unchecked::<GoogleOpenIdClaims>().unwrap();
            let oauth_id = claims.custom.sub;

            Users::get_user_by_oauth_id(db_context_ref, oauth_id).await.unwrap()
        }
        None => None
    }
}
