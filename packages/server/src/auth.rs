use crate::AppState;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use entities::users;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct CurrentUser(pub users::Model);

pub async fn validate_token(token: &str, state: &AppState) -> Result<CurrentUser, Response> {
    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => token_data.claims,
        Err(err) => {
            let error_message = match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                _ => "Invalid token",
            };
            error!("JWT decoding error: {}", err);
            return Err(
                (StatusCode::UNAUTHORIZED, Json(json!({ "error": error_message }))).into_response(),
            );
        }
    };

    let user_id = match Uuid::from_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid token subject"})),
            )
                .into_response());
        }
    };

    let user = match users::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
                .into_response());
        }
        Err(e) => {
            error!("Database error while fetching user: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            )
                .into_response());
        }
    };

    Ok(CurrentUser(user))
}

