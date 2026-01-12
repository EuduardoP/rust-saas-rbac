use crate::{auth::validate_token, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_auth::AuthBearer;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    id: Uuid,
    name: Option<String>,
    email: String,
    avatar_url: Option<String>,
}

#[utoipa::path(
    get,
    path = "/profile",
    tag = "Auth",
    responses(
        (status = 200, description = "Authenticated user's profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("token" = [])
    )
)]
/// Get Authenticate user profile
pub async fn get_profile(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> impl IntoResponse {
    match validate_token(&token, &state).await {
        Ok(user) => {
            let response = ProfileResponse {
                id: user.0.id,
                name: user.0.name,
                email: user.0.email,
                avatar_url: user.0.avatar_url,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(response) => Err(response),
    }
}
