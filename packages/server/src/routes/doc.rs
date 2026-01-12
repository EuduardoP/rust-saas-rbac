use crate::routes::auth::reset_password::ResetPasswordRequest;
use crate::routes::auth::{
    authenticate_with_github::{AuthenticateWithGithubBody, AuthenticateWithGithubResponse},
    authenticate_with_password::{AuthenticateWithPasswordBody, AuthenticateWithPasswordResponse},
    create_account::{CreateAccountBody, CreateAccountResponse},
    get_profile::ProfileResponse,
    request_password_recover::{RequestPasswordRecoverBody, RequestPasswordRecoverResponse},
};
use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::{Html, IntoResponse},
};
use scalar_doc::Documentation;
use std::fs;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::auth::create_account::create_account,
        crate::routes::auth::authenticate_with_password::authenticate_with_password,
        crate::routes::auth::authenticate_with_github::authenticate_with_github,
        crate::routes::auth::get_profile::get_profile,
        crate::routes::auth::request_password_recover::request_password_recover,
        crate::routes::auth::reset_password::reset_password,
    ),
    components(schemas(
        CreateAccountBody,
        CreateAccountResponse,
        AuthenticateWithPasswordBody,
        AuthenticateWithPasswordResponse,
        AuthenticateWithGithubBody,
        AuthenticateWithGithubResponse,
        ProfileResponse,
        RequestPasswordRecoverBody,
        RequestPasswordRecoverResponse,
        ResetPasswordRequest
    )),
    info(title = "Rust SaaS RBAC API", version = "1.0.0"),
    modifiers(&SecurityAddon)
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
