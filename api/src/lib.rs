use std::convert::Infallible;

use axum::{
    async_trait,
    body::HttpBody,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    Router,
};

use axum_aws_lambda::{LambdaLayer, LambdaService};
use data_access::user::Users;
use endurance_racing_planner_common::{GoogleOpenIdClaims, User};
use jwt_compact::UntrustedToken;
use lambda_http::http::HeaderValue;
use lambda_http::Service;
use lambda_http::{http::header::CONTENT_TYPE, tower::ServiceBuilder, Error};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

pub async fn initialize_lambda<T, B>(
    route: &'static str,
    service: T,
) -> Result<LambdaService<Router<B>>, Error>
where
    T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    T::Future: Send + 'static,
    B: HttpBody + Send + 'static,
{
    let db_context = data_access::initialize().await?;

    let app = Router::new()
        .route(route, service)
        .layer(Extension(db_context))
        .layer(cors_layer());

    Ok(ServiceBuilder::new()
        .layer(LambdaLayer::default())
        .service(app))
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_headers([CONTENT_TYPE])
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

pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<B> FromRequest<B> for AuthenticatedUser
where
    B: Send,
{
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool): Extension<PgPool> = Extension::from_request(req)
            .await
            .map_err(|err| err.into_response())?;

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "no bearer token").into_response())?;

        let oauth_id = UntrustedToken::new(bearer.token())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token"))
            .and_then(|parsed_token| {
                parsed_token
                    .deserialize_claims_unchecked::<GoogleOpenIdClaims>()
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token"))
            })
            .and_then(|claims| Ok(claims.custom.sub))
            .map_err(|err| err.into_response())?;

        Users::get_user_by_oauth_id(&pool, oauth_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "there was a problem locating the user",
                )
                    .into_response()
            })
            .and_then(|user| {
                user.ok_or((StatusCode::UNAUTHORIZED, "user not found").into_response())
            })
            .map(Self)
    }
}
