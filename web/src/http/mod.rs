pub mod plans;

use std::error::Error;

use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use reqwest::{Method, RequestBuilder, Url};
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use crate::auth::ID_TOKEN_KEY;

const BASE_PATH: &str = "http://localhost:3000";

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

pub fn post<T>(route: String, body: T, callback: Callback<T>) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        let response = get_request_builder(Method::POST, &route)
            .unwrap()
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap();

        callback.emit(response)
    })
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
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await
            .unwrap();
    })
}
