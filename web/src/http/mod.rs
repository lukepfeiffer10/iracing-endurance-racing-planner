pub mod drivers;
pub mod plans;
pub mod schedules;

use std::fmt::Debug;

use endurance_racing_planner_common::GoogleOpenIdClaims;
use gloo_console::info;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use jwt_compact::{ParseError, TimeOptions, UntrustedToken};
use reqwest::{header::CONTENT_TYPE, Method, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::auth::{login, ID_TOKEN_KEY};

const BASE_PATH: &str = match option_env!("API_BASE_PATH") {
    Some(base_path) => base_path,
    None => dotenv!("API_BASE_PATH"),
};

pub enum CustomError {
    TokenNotFound(StorageError),
    BadToken(ParseError),
    TokenExpired,
    BadUrl(oauth2::url::ParseError),
    FailedRequest,
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenNotFound(e) => write!(f, "TokenNotFound: {:?}", e),
            Self::BadToken(e) => write!(f, "BadToken: {:?}", e),
            Self::TokenExpired => write!(f, "TokenExpired"),
            Self::BadUrl(e) => write!(f, "BadUrl: {:?}", e),
            Self::FailedRequest => write!(f, "FailedRequest"),
        }
    }
}

fn get_auth_token() -> Result<String, CustomError> {
    LocalStorage::get(ID_TOKEN_KEY)
        .map_err(|e| CustomError::TokenNotFound(e))
        .and_then(
            move |token_string: String| match UntrustedToken::new(token_string.as_str()) {
                Ok(token) => token
                    .deserialize_claims_unchecked::<GoogleOpenIdClaims>()
                    .map_err(|_| CustomError::TokenExpired)
                    .and_then(
                        |claims| match claims.validate_expiration(&TimeOptions::default()) {
                            Ok(_) => Ok(()),
                            Err(_) => {
                                LocalStorage::delete(ID_TOKEN_KEY);
                                Err(CustomError::TokenExpired)
                            }
                        },
                    )
                    .map(|_| token_string),
                Err(e) => {
                    LocalStorage::delete(ID_TOKEN_KEY);
                    Err(CustomError::BadToken(e))
                }
            },
        )
}

fn get_request_builder(method: Method, route: &str) -> Result<RequestBuilder, CustomError> {
    let base_url = Url::parse(BASE_PATH).map_err(|e| CustomError::BadUrl(e))?;
    let client = reqwest::Client::new();
    Ok(client
        .request(
            method,
            base_url.join(route).map_err(|e| CustomError::BadUrl(e))?,
        )
        .bearer_auth(get_auth_token()?))
}

fn handle_error(e: CustomError) {
    match e {
        CustomError::TokenNotFound(_) | CustomError::BadToken(_) | CustomError::TokenExpired => {
            login()
        }
        CustomError::BadUrl(e) => info!(format!("{:?}", e)),
        CustomError::FailedRequest => info!("request failed"),
    }
}

pub fn post<T, U>(route: String, body: T, callback: Option<Callback<U>>) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
    U: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        match get_request_builder(Method::POST, &route) {
            Ok(builder) => {
                let response = builder
                    .header(CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&body).unwrap())
                    .send()
                    .await
                    .unwrap();

                if let Some(callback) = callback {
                    let response = response.json::<U>().await.unwrap();

                    callback.emit(response)
                }
            }
            Err(e) => handle_error(e),
        }
    })
}

pub async fn post_async<T>(route: String, body: T) -> T
where
    T: Serialize + DeserializeOwned + 'static,
{
    get_request_builder(Method::POST, &route)
        .unwrap()
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .unwrap()
        .json::<T>()
        .await
        .unwrap()
}

pub fn get<T>(route: String, callback: Callback<T>) -> ()
where
    T: DeserializeOwned + 'static,
{
    spawn_local(async move {
        match get_request_builder(Method::GET, &route) {
            Ok(builder) => {
                let response = builder.send().await.unwrap().json::<T>().await.unwrap();

                callback.emit(response)
            }
            Err(e) => handle_error(e),
        }
    })
}

pub async fn get_async<T>(route: String) -> Result<T, CustomError>
where
    T: DeserializeOwned + 'static,
{
    get_request_builder(Method::GET, &route)?
        .send()
        .await
        .unwrap()
        .json::<T>()
        .await
        .map_err(|_| CustomError::FailedRequest)
}

pub fn patch<T>(route: String, body: T) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        match get_request_builder(Method::PATCH, &route) {
            Ok(builder) => {
                builder
                    .header(CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&body).unwrap())
                    .send()
                    .await
                    .unwrap();
            }
            Err(e) => handle_error(e),
        }
    })
}

pub fn put<T>(route: String, body: T) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        match get_request_builder(Method::PUT, &route) {
            Ok(builder) => {
                builder
                    .header(CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&body).unwrap())
                    .send()
                    .await
                    .unwrap();
            }
            Err(e) => handle_error(e),
        }
    })
}
