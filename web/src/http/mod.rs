pub mod plans;
pub mod schedules;

use std::error::Error;

use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use reqwest::{header::CONTENT_TYPE, Method, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::auth::ID_TOKEN_KEY;

const BASE_PATH: &str = dotenv!("API_BASE_PATH");

fn get_auth_token() -> Result<String, StorageError> {
    LocalStorage::get(ID_TOKEN_KEY)
}

fn get_request_builder(method: Method, route: &str) -> Result<RequestBuilder, Box<dyn Error>> {
    let base_url = Url::parse(BASE_PATH)?;
    let client = reqwest::Client::new();
    Ok(client
        .request(method, base_url.join(route)?)
        .bearer_auth(get_auth_token()?))
}

pub fn post<T, U>(route: String, body: T, callback: Option<Callback<U>>) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
    U: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        let response = get_request_builder(Method::POST, &route)
            .unwrap()
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .unwrap();

        if let Some(callback) = callback {
            let response = response.json::<U>().await.unwrap();

            callback.emit(response)
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
        let response = get_request_builder(Method::GET, &route)
            .unwrap()
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap();

        callback.emit(response)
    })
}

pub fn patch<T>(route: String, body: T) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        get_request_builder(Method::PATCH, &route)
            .unwrap()
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .unwrap();
    })
}

pub fn put<T>(route: String, body: T) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        get_request_builder(Method::PUT, &route)
            .unwrap()
            .header(CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .unwrap();
    })
}
