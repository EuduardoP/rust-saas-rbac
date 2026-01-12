use crate::{error::ErrorResponse, AppState};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use entities::members as Members;
use entities::organizations as Orgs;
use entities::sea_orm_active_enums::Role;
use entities::users as Users;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateAccountBody {
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateAccountResponse {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "Auth",
    request_body = CreateAccountBody,
    responses(
        (status = 201, description = "User created successfully", body = CreateAccountResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "User with same e-mail already exists"),
        (status = 500, description = "Internal server error")
    )
)]
/// Create account
pub async fn create_account(
    State(state): State<AppState>,
    Json(body): Json<CreateAccountBody>,
) -> impl IntoResponse {
    if let Err(e) = body.validate() {
        error!("Validation error: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        ));
    }

    let user_exists = match Users::Entity::find()
        .filter(Users::Column::Email.eq(body.email.clone()))
        .one(&state.db)
        .await
    {
        Ok(user) => user,
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

    if user_exists.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: String::from("User with same e-mail already exists."),
            }),
        ));
    }

    let domain = body.email.split('@').nth(1).unwrap_or("");

    let auto_join_organization = match Orgs::Entity::find()
        .filter(Orgs::Column::Domain.eq(domain))
        .filter(Orgs::Column::ShouldAttachUsersByDomain.eq(true))
        .one(&state.db)
        .await
    {
        Ok(org) => org,
        Err(e) => {
            error!("Error fetching organization: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(body.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!("Error generating hash: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    let new_user = Users::ActiveModel {
        name: Set(Some(body.name.clone())),
        email: Set(body.email.clone()),
        password_hash: Set(Some(password_hash)),
        ..Default::default()
    };

    let inserted_user = match new_user.insert(&state.db).await {
        Ok(user) => user,
        Err(e) => {
            error!("Error creating user: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    if let Some(org) = auto_join_organization {
        let new_member = Members::ActiveModel {
            user_id: Set(inserted_user.id),
            organization_id: Set(org.id),
            role: Set(Role::Member),
            ..Default::default()
        };

        if let Err(e) = new_member.insert(&state.db).await {
            error!("Error adding member: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateAccountResponse {
            user_id: inserted_user.id,
        }),
    ))
}
