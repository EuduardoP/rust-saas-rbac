use crate::{error::ErrorResponse, AppState};
use axum::{http::StatusCode, Json};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub fn get_current_user_id(
    token: &str,
    state: &AppState,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
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
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: error_message.to_string(),
                }),
            ));
        }
    };

    match Uuid::from_str(&claims.sub) {
        Ok(sub) => Ok(sub),
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: String::from("Invalid token subject"),
            }),
        )),
    }
}
