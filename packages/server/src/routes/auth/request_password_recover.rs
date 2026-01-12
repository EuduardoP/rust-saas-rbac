use crate::{error::ErrorResponse, AppState};
use axum::{extract::State, response::IntoResponse, Json};
use entities::{sea_orm_active_enums::TokenType, tokens, users};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RequestPasswordRecoverBody {
    #[validate(email)]
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct RequestPasswordRecoverResponse {
    pub code: Uuid,
}

#[utoipa::path(
    post,
    path = "/request-password-recover",
    tag = "Auth",
    request_body = RequestPasswordRecoverBody,
    responses(
        (status = 200, description = "Password recovery requested successfully", body = RequestPasswordRecoverResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn request_password_recover(
    State(state): State<AppState>,
    Json(body): Json<RequestPasswordRecoverBody>,
) -> impl IntoResponse {
    // NOTE: This implementation is for testing purposes only.
    // In a real application, the recovery code should not be returned directly in the response.
    // Instead, an email containing a link with the recovery code should be sent to the user.
    let user_from_email = match users::Entity::find()
        .filter(users::Column::Email.eq(body.email.clone()))
        .one(&state.db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: String::from("Invalid credentials."),
                }),
            ));
        }
        Err(e) => {
            error!("Db query error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    let token = tokens::ActiveModel {
        user_id: Set(user_from_email.id),
        r#type: Set(TokenType::PasswordRecover),
        ..Default::default()
    };

    let code = token.id.unwrap();

    Ok((
        StatusCode::OK,
        Json(RequestPasswordRecoverResponse { code }),
    ))
}
