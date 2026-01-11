use axum::{extract::State, response::IntoResponse, Json};
use entities::{sea_orm_active_enums::TokenType, tokens, users};
use reqwest::StatusCode;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use utoipa::ToSchema;
use validator::Validate;

use crate::AppState;

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RequestPasswordRecoverBody {
    #[validate(email)]
    pub email: String,
}

pub async fn request_password_recover(
    State(state): State<AppState>,
    Json(body): Json<RequestPasswordRecoverBody>,
) -> impl IntoResponse {
    let user_from_email = match users::Entity::find()
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

    let token = tokens::ActiveModel {
        user_id: Set(user_from_email.id),
        r#type: Set(TokenType::PasswordRecover),
        ..Default::default()
    };

    let code = token.id.unwrap();

    // Implementation goes here
    // This is a placeholder to show where the logic would be implemented
    (
        StatusCode::OK,
        Json(json!({
            "message": "If the email exists, a recovery link has been sent.",
            "code": code.clone()
        })),
    )
}
