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
    let base_url = Url::parse(BASE_PATH).expect("base path to be a valid url");
    let client = reqwest::Client::new();
    Ok(client
        .request(method, base_url.join(route)?)
        .bearer_auth(get_auth_token()?))
}

pub fn post<T>(route: &'static str, body: T, callback: Callback<T>) -> ()
where
    T: Serialize + DeserializeOwned + 'static,
{
    spawn_local(async move {
        let response = get_request_builder(Method::POST, route)
            .expect("request to be built successfully")
            .body(serde_json::to_string(&body).expect("failed to convert body to serde json value"))
            .send()
            .await
            .expect("post request to return a successful response")
            .json::<T>()
            .await
            .expect("response body to serialize from json properly");

        callback.emit(response)
    })
}
