use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use entities::{tokens, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{error::ErrorResponse, AppState};

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct ResetPasswordRequest {
    pub code: Uuid,
    #[validate(length(min = 6))]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/password/reset",
    tag = "Auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 204, description = "Password reset successfully"),
        (status = 403, description = "Invalid or expired recovery code"),
        (status = 500, description = "Internal server error")
    )
)]
/// Reset user password using a recovery code.
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        ));
    }

    let token = match tokens::Entity::find()
        .filter(tokens::Column::Id.eq(body.code))
        .one(&state.db)
        .await
    {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Invalid or expired recovery code.".into(),
                }),
            ));
        }
        Err(e) => {
            error!("Error fetching token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".into(),
                }),
            ));
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(body.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!("Error hashing password: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".into(),
                }),
            ));
        }
    };

    let tx_result = state
        .db
        .transaction(|txn| {
            Box::pin(async move {
                let mut user: users::ActiveModel = users::Entity::find_by_id(token.user_id)
                    .one(txn)
                    .await?
                    .ok_or(DbErr::RecordNotFound("User not found".into()))?
                    .into();

                user.password_hash = Set(Some(password_hash));
                user.update(txn).await?;

                tokens::Entity::delete_by_id(token.id).exec(txn).await?;

                Ok::<(), DbErr>(())
            })
        })
        .await;

    match tx_result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Reset password transaction failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".into(),
                }),
            ))
        }
    }
}
