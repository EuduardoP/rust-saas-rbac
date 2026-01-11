use crate::routes::auth::{
    authenticate_with_github::{AuthenticateWithGithubBody, AuthenticateWithGithubResponse},
    authenticate_with_password::{AuthenticateWithPasswordBody, AuthenticateWithPasswordResponse},
    create_account::{CreateAccountBody, CreateAccountResponse},
};
use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::{Html, IntoResponse},
};
use scalar_doc::Documentation;
use std::fs;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::create_account::create_account,
        crate::routes::auth::authenticate_with_password::authenticate_with_password,
        crate::routes::auth::authenticate_with_github::authenticate_with_github,
    ),
    components(schemas(
        CreateAccountBody,
        CreateAccountResponse,
        AuthenticateWithPasswordBody,
        AuthenticateWithPasswordResponse,
        AuthenticateWithGithubBody,
        AuthenticateWithGithubResponse,
    )),
    info(title = "Rust SaaS RBAC API", version = "1.0.0")
)]
pub struct ApiDoc;

/// Serves the Scalar API documentation UI.
pub async fn doc() -> impl IntoResponse {
    Html(
        Documentation::new("API Docs", "/openapi.json")
            .build()
            .unwrap(),
    )
}

/// Serves the raw OpenAPI specification file.
pub async fn openapi_spec_handler() -> Response<Body> {
    match fs::read_to_string("openapi.json") {
        Ok(spec) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(spec))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
