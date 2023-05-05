use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    routing::{get, post, put},
    Router, TypedHeader,
};
use dotenvy::dotenv;
use endurance_racing_planner_common::{GoogleOpenIdClaims, User};
use http_cache_reqwest::{Cache, CacheMode, HttpCache, MokaManager};
use jwt_compact::{
    alg::{Rsa, RsaPublicKey},
    jwk::JsonWebKey,
    AlgorithmExt, TimeOptions, UntrustedToken,
};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Deserialize;
use sqlx::PgPool;
use std::net::SocketAddr;

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
    let http_client = ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: MokaManager::default(),
            options: None,
        }))
        .build();

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
        .route("/plans/:id/share", post(plans::share_plan))
        .route("/plans/:id/share", get(plans::get_plan_shared_users))
        .route("/drivers/:id", put(drivers::put_driver))
        .with_state(AppState {
            pool: db_context,
            http_client,
        });

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    http_client: ClientWithMiddleware,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.pool.clone()
    }
}

impl FromRef<AppState> for ClientWithMiddleware {
    fn from_ref(app_state: &AppState) -> ClientWithMiddleware {
        app_state.http_client.clone()
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

        let http_client = AppState::from_ref(state).http_client;
        let oauth_signing_keys = get_google_signing_keys(http_client).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "no oauth signing keys".to_string(),
            )
        })?;
        let oauth_id = UntrustedToken::new(bearer.token())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))
            .and_then(|parsed_token| {
                let mut signing_key = &oauth_signing_keys.keys[0].key;
                let token_key_id = &parsed_token.header().key_id;
                if let Some(key_id) = token_key_id {
                    signing_key = oauth_signing_keys
                        .keys
                        .iter()
                        .find(|key| &key.kid == key_id)
                        .map(|k| &k.key)
                        .ok_or((StatusCode::UNAUTHORIZED, "bad signing key".to_string()))?;
                }
                let rsa_public_key = RsaPublicKey::try_from(signing_key).unwrap();
                let token_message = Rsa::rs256()
                    .validate_integrity::<GoogleOpenIdClaims>(&parsed_token, &rsa_public_key);
                token_message.map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))
            })
            .and_then(|token| {
                let claims = token
                    .claims()
                    .validate_expiration(&TimeOptions::default())
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "token expired".to_string()))?;
                Ok(claims.custom.sub.clone())
            })?;

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

#[derive(Deserialize)]
struct GoogleDiscoveryResponse {
    jwks_uri: String,
}

#[derive(Deserialize)]
struct GoogleSigningKey<'a> {
    kid: String,
    #[serde(flatten)]
    key: JsonWebKey<'a>,
}

#[derive(Deserialize)]
struct GoogleSigningKeysResponse<'a> {
    keys: Vec<GoogleSigningKey<'a>>,
}

async fn get_google_signing_keys(
    client: ClientWithMiddleware,
) -> Result<GoogleSigningKeysResponse<'static>, Box<dyn std::error::Error>> {
    const GOOGLE_DISCOVERY_URL: &str =
        "https://accounts.google.com/.well-known/openid-configuration";

    let discovery_info = client
        .get(GOOGLE_DISCOVERY_URL)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<GoogleDiscoveryResponse>()
        .await?;

    let signing_keys = client
        .get(&discovery_info.jwks_uri)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<GoogleSigningKeysResponse>()
        .await?;

    Ok(signing_keys)
}
