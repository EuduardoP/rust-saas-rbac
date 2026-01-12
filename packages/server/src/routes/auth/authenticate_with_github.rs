use crate::{auth::Claims, error::ErrorResponse, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use entities::{accounts, sea_orm_active_enums::AccountProvider, users};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tracing::error;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AuthenticateWithGithubBody {
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthenticateWithGithubResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct GithubAccessTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GithubUserResponse {
    id: i64,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

#[utoipa::path(
    post,
    path = "/sessions/github",
    tag = "Auth",
    request_body = AuthenticateWithGithubBody,
    responses(
        (status = 201, description = "Authenticated successfully", body = AuthenticateWithGithubResponse),
        (status = 400, description = "Validation error or user without Github"),
        (status = 403, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
/// Login with Github
pub async fn authenticate_with_github(
    State(state): State<AppState>,
    Json(body): Json<AuthenticateWithGithubBody>,
) -> impl IntoResponse {
    let github_token_res = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &state.github_client_id),
            ("client_secret", &state.github_client_secret),
            ("redirect_uri", &state.github_oauth_redirect_url),
            ("code", &body.code),
        ])
        .send()
        .await;

    let github_token_res = match github_token_res {
        Ok(res) => res,
        Err(e) => {
            error!("GitHub token request failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("GitHub authentication failed"),
                }),
            ));
        }
    };

    let token_data: GithubAccessTokenResponse = match github_token_res.json().await {
        Ok(data) => data,
        Err(e) => {
            error!("Invalid GitHub token response: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("GitHub authentication failed"),
                }),
            ));
        }
    };

    let github_user_res = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            format!("Bearer {}", token_data.access_token),
        )
        .header("User-Agent", "axum-app")
        .send()
        .await;

    let github_user_res = match github_user_res {
        Ok(res) => res,
        Err(e) => {
            error!("GitHub user request failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("GitHub authentication failed"),
                }),
            ));
        }
    };

    let github_user: GithubUserResponse = match github_user_res.json().await {
        Ok(data) => data,
        Err(e) => {
            error!("Invalid GitHub user response: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("GitHub authentication failed"),
                }),
            ));
        }
    };

    let email = match github_user.email {
        Some(email) => email,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: String::from("Your GitHub account must have an email to authenticate."),
                }),
            ));
        }
    };

    let github_id = github_user.id.to_string();

    // ======================
    // 3. Buscar ou criar usuário
    // ======================
    let user = match users::Entity::find()
        .filter(users::Column::Email.eq(email.clone()))
        .one(&state.db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            let new_user = users::ActiveModel {
                email: Set(email),
                name: Set(github_user.name),
                avatar_url: Set(Some(github_user.avatar_url)),
                ..Default::default()
            };

            match new_user.insert(&state.db).await {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to create user: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: String::from("Internal server error"),
                        }),
                    ));
                }
            }
        }
        Err(e) => {
            error!("User query failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    // ======================
    // 4. Buscar ou criar account GitHub
    // ======================
    let account = accounts::Entity::find()
        .filter(accounts::Column::Provider.eq(AccountProvider::Github))
        .filter(accounts::Column::ProviderAccountId.eq(github_id.clone()))
        .one(&state.db)
        .await;

    if let Ok(None) = account {
        let new_account = accounts::ActiveModel {
            provider: Set(AccountProvider::Github),
            provider_account_id: Set(github_id),
            user_id: Set(user.id),
            ..Default::default()
        };

        if let Err(e) = new_account.insert(&state.db).await {
            error!("Failed to create account: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    }

    // ======================
    // 5. Gerar JWT
    // ======================
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
        Ok(token) => token,
        Err(e) => {
            error!("JWT generation failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal server error"),
                }),
            ));
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(AuthenticateWithGithubResponse { token }),
    ))
}
