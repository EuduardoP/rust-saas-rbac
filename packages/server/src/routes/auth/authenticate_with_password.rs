use crate::AppState;
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use entities::users;
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use tracing::error;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct AuthenticateWithPasswordBody {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthenticateWithPasswordResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: i64,
}

#[utoipa::path(
    post,
    path = "/sessions/password",
    tag = "Auth",
    request_body = AuthenticateWithPasswordBody,
    responses(
        (status = 201, description = "Authenticated successfully", body = AuthenticateWithPasswordResponse),
        (status = 400, description = "Validation error or user without password"),
        (status = 403, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
/// Login with password & email
pub async fn authenticate_with_password(
    State(state): State<AppState>,
    Json(body): Json<AuthenticateWithPasswordBody>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        error!("Validation error: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Validation error: {}", e) })),
        );
    }

    let user = match users::Entity::find()
        .filter(users::Column::Email.eq(body.email.clone()))
        .one(&state.db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "Invalid credentials."})),
            );
        }
        Err(e) => {
            error!("Db query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            );
        }
    };

    let password_hash = match user.password_hash {
        Some(hash) => hash,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "User does not have a password, use social login."})),
            );
        }
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to parse password hash from database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            );
        }
    };

    if Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Invalid credentials."})),
        );
    }

    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::days(7);

    let claims = Claims {
        sub: user.id.to_string(),
        exp: exp.unix_timestamp(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to generate JWT: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            );
        }
    };

    (
        StatusCode::CREATED,
        Json(json!(AuthenticateWithPasswordResponse { token })),
    )
}
