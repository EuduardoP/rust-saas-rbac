use crate::{auth::get_user_membership, error::ErrorResponse, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auth::AuthBearer;
use entities::{members, users};
use rbac::{get_user_permission, Action, Resource};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    Member,
    Billing,
}

impl From<entities::sea_orm_active_enums::Role> for Role {
    fn from(role: entities::sea_orm_active_enums::Role) -> Self {
        match role {
            entities::sea_orm_active_enums::Role::Admin => Role::Admin,
            entities::sea_orm_active_enums::Role::Member => Role::Member,
            entities::sea_orm_active_enums::Role::Billing => Role::Billing,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Member {
    id: Uuid,
    #[serde(rename = "userId")]
    user_id: Uuid,
    role: Role,
    name: Option<String>,
    email: String,
    #[serde(rename = "avatarUrl")]
    avatar_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GetMembersResponse {
    members: Vec<Member>,
}

#[utoipa::path(
    get,
    path = "/organizations/{slug}/members",
    tag = "Members",
    security(
        ("token" = [])
    ),
    responses(
        (status = 200, description = "Get all organization members", body = GetMembersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_members(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    AuthBearer(token): AuthBearer,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let membership_response = get_user_membership(&state, &slug, &token).await?;
    let membership_data = membership_response.as_object().unwrap();

    let membership_json = membership_data.get("membership").unwrap();
    let membership: members::Model = serde_json::from_value(membership_json.clone()).unwrap();

    let ability = get_user_permission(membership.user_id, membership.role);

    if ability.cannot(&Action::Read, &Resource::Subject("User")) {
        return Err(ErrorResponse::new(
            StatusCode::FORBIDDEN,
            "You're not allowed to see organization members.",
        ));
    }

    let organization_json = membership_data.get("organization").unwrap();
    let organization: entities::organizations::Model =
        serde_json::from_value(organization_json.clone()).unwrap();

    let members = members::Entity::find()
        .find_also_related(users::Entity)
        .filter(members::Column::OrganizationId.eq(organization.id))
        .all(&state.db)
        .await
        .map_err(|_| ErrorResponse::internal_error())?;

    let members_with_roles = members
        .into_iter()
        .map(|(member, user)| {
            let user = user.unwrap();
            Member {
                id: member.id,
                user_id: user.id,
                role: member.role.into(),
                name: user.name,
                email: user.email,
                avatar_url: user.avatar_url,
            }
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(GetMembersResponse {
            members: members_with_roles,
        }),
    ))
}

