use lambda_http::http::response::Builder;
use lambda_http::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, LOCATION};
use lambda_http::{Body, IntoResponse, Response};
use lambda_http::http::StatusCode;
use serde::Serialize;

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
