use crate::{auth::get_current_user_id, error::ErrorResponse, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_auth::AuthBearer;
use entities::users;
use sea_orm::EntityTrait;
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
    let user_id = match get_current_user_id(&token, &state) {
        Ok(user_id) => user_id,
        Err(err_response) => return Err(err_response),
    };

    let user = match users::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(ErrorResponse::unauthorized())
        }
        Err(_) => {
            return Err(ErrorResponse::internal_error())
        }
    };

    let response = ProfileResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        avatar_url: user.avatar_url,
    };

    Ok((StatusCode::OK, Json(response)))
}
