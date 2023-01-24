use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        request::Parts,
        HeaderValue, Method, StatusCode,
    },
    routing::{get, post, put},
    Router, TypedHeader,
};
use dotenvy::dotenv;
use endurance_racing_planner_common::{GoogleOpenIdClaims, User};
use jwt_compact::UntrustedToken;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use crate::data_access::user::Users;

mod data_access;
mod drivers;
mod plans;
mod schedules;
mod users;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let db_context = data_access::initialize().await.expect("database to exist");

    // build our application with a route
    let app = Router::new()
        .route("/users/me", get(users::me))
        .route("/users", post(users::add_user))
        .route("/plans", get(plans::get_plans).post(plans::add_plan))
        .route("/plans/:id", get(plans::get_plan).patch(plans::patch_plan))
        .route(
            "/plans/:id/schedule",
            get(schedules::get_schedule)
                .post(schedules::add_schedule)
                .put(schedules::put_schedule),
        )
        .route(
            "/plans/:id/drivers",
            get(drivers::get_plan_drivers).post(drivers::add_driver),
        )
        .route("/drivers/:id", put(drivers::put_driver))
        .layer(cors_layer())
        .with_state(AppState { pool: db_context });

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_origin("http://localhost:9000".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::PUT,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
}

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.pool.clone()
    }
}

pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool: PgPool = AppState::from_ref(state).pool;

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "no bearer token".to_string()))?;

        let oauth_id = UntrustedToken::new(bearer.token())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))
            .and_then(|parsed_token| {
                parsed_token
                    .deserialize_claims_unchecked::<GoogleOpenIdClaims>()
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))
            })
            .map(|claims| claims.custom.sub)?;

        Users::get_user_by_oauth_id(&pool, oauth_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "there was a problem locating the user".to_string(),
                )
            })
            .and_then(|user| user.ok_or((StatusCode::UNAUTHORIZED, "user not found".to_string())))
            .map(Self)
    }
}
