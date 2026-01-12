use crate::{error::ErrorResponse, AppState};
use axum::{
    http::StatusCode,
    Json,
};
use entities::{members, organizations};
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
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

pub fn get_current_user_id(
    token: &str,
    state: &AppState,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|err| {
        error!("JWT decoding error: {}", err);
        let msg = match err.kind() {
            ErrorKind::ExpiredSignature => "Token has expired",
            _ => "Invalid token",
        };
        ErrorResponse::new(StatusCode::UNAUTHORIZED, msg)
    })?;

    Uuid::from_str(&token_data.claims.sub)
        .map_err(|_| ErrorResponse::new(StatusCode::UNAUTHORIZED, "Invalid token subject"))
}

pub async fn get_user_membership(
    state: &AppState,
    slug: &str,
    token: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = get_current_user_id(token, state)?;

    let result = members::Entity::find()
        .filter(members::Column::UserId.eq(user_id))
        .find_also_related(organizations::Entity)
        .filter(organizations::Column::Slug.eq(slug))
        .one(&state.db)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            ErrorResponse::internal_error()
        })?;

    match result {
        Some((member, Some(organization))) => Ok(Json(json!({
            "organization": organization,
            "membership": member,
        }))),
        _ => Err(ErrorResponse::new(
            StatusCode::FORBIDDEN,
            "You're not a member of this organization.",
        )),
    }
}