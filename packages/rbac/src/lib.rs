use entities::sea_orm_active_enums::Role;
use entities::{invites, organizations, projects, users};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    Manage,
    Create,
    Read,
    Update,
    Delete,
    TransferOwnership,
}

#[derive(Debug, Clone)]
pub enum Resource<'a> {
    Organization(&'a organizations::Model),
    Project(&'a projects::Model),
    Invite(&'a invites::Model),
    User(&'a users::Model),
    Subject(&'a str),
    All,
}

pub struct Ability {
    user_id: Uuid,
    role: Role,
}

impl Ability {
    pub fn can(&self, action: &Action, resource: &Resource) -> bool {
        match self.role {
            Role::Admin => self.can_admin(action, resource),
            Role::Member => self.can_member(action, resource),
            Role::Billing => self.can_billing(action, resource),
        }
    }

    pub fn cannot(&self, action: &Action, resource: &Resource) -> bool {
        !self.can(action, resource)
    }

    fn can_admin(&self, action: &Action, resource: &Resource) -> bool {
        match (action, resource) {
            (Action::Create, Resource::Invite(invite)) if invite.role != Role::Member => false,
            (Action::Update, Resource::Organization(org)) if org.owner_id != self.user_id => false,
            (Action::TransferOwnership, Resource::Organization(org))
                if org.owner_id != self.user_id =>
            {
                false
            }
            _ => true,
        }
    }

    fn can_member(&self, action: &Action, resource: &Resource) -> bool {
        match (action, resource) {
            (Action::Read, Resource::Project(_)) => true,
            (Action::Read, Resource::Organization(_)) => true,
            (Action::Update, Resource::User(user)) if user.id == self.user_id => true,
            (Action::Manage, Resource::Project(project)) if project.owner_id == self.user_id => {
                true
            }
            (Action::Create, Resource::Project(_)) => true,
            (Action::Update, Resource::Project(project)) if project.owner_id == self.user_id => {
                true
            }
            (Action::Delete, Resource::Project(project)) if project.owner_id == self.user_id => {
                true
            }
            (Action::Create, Resource::Invite(invite)) if invite.role == Role::Member => true,
            _ => false,
        }
    }

    fn can_billing(&self, action: &Action, resource: &Resource) -> bool {
        match (action, resource) {
            (Action::Read, Resource::Organization(_)) => true,
            _ => false,
        }
    }
}

pub fn get_user_permission(user_id: Uuid, role: Role) -> Ability {
    Ability { user_id, role }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities::sea_orm_active_enums::Role;
    use uuid::Uuid;

    #[test]
    fn test_admin_permissions() {
        let admin_id = Uuid::new_v4();
        let ability = get_user_permission(admin_id, Role::Admin);
        let other_user_id = Uuid::new_v4();
        let org_owned_by_other = organizations::Model {
            id: Uuid::new_v4(),
            owner_id: other_user_id,
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            domain: None,
            should_attach_users_by_domain: false,
            avatar_url: None,
        };
        let org_owned_by_admin = organizations::Model {
            id: Uuid::new_v4(),
            owner_id: admin_id,
            ..org_owned_by_other.clone()
        };
        let project = projects::Model {
            id: Uuid::new_v4(),
            owner_id: admin_id,
            organization_id: org_owned_by_admin.id,
            name: "Test Project".to_string(),
            slug: "test-project".to_string(),
            description: "A test project".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            avatar_url: None,
        };

        assert!(ability.can(&Action::Manage, &Resource::All));
        assert!(ability.can(&Action::Read, &Resource::Subject("User"))); // Admin can read users
        assert!(ability.can(&Action::Create, &Resource::Subject("Project")));
        assert!(ability.can(&Action::Create, &Resource::Project(&project)));

        assert!(ability.cannot(
            &Action::Update,
            &Resource::Organization(&org_owned_by_other)
        ));
        assert!(ability.cannot(
            &Action::TransferOwnership,
            &Resource::Organization(&org_owned_by_other)
        ));

        assert!(ability.can(
            &Action::Update,
            &Resource::Organization(&org_owned_by_admin)
        ));

        let invite_for_admin = invites::Model {
            id: Uuid::new_v4(),
            author_id: Some(admin_id),
            organization_id: org_owned_by_admin.id,
            email: "new@member.com".to_string(),
            role: Role::Admin,
            created_at: Default::default(),
        };
        assert!(ability.cannot(&Action::Create, &Resource::Invite(&invite_for_admin)));

        let invite_for_member = invites::Model {
            role: Role::Member,
            ..invite_for_admin
        };
        assert!(ability.can(&Action::Create, &Resource::Invite(&invite_for_member)));
    }

    #[test]
    fn test_member_permissions() {
        let member_id = Uuid::new_v4();
        let ability = get_user_permission(member_id, Role::Member);
        let other_user_id = Uuid::new_v4();
        let org = organizations::Model {
            id: Uuid::new_v4(),
            owner_id: other_user_id,
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            domain: None,
            should_attach_users_by_domain: false,
            avatar_url: None,
        };

        let own_project = projects::Model {
            id: Uuid::new_v4(),
            owner_id: member_id,
            organization_id: org.id,
            name: "Own Project".to_string(),
            slug: "own-project".to_string(),
            description: "A test project".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            avatar_url: None,
        };
        let other_project = projects::Model {
            id: Uuid::new_v4(),
            owner_id: other_user_id,
            organization_id: org.id,
            name: "Other Project".to_string(),
            slug: "other-project".to_string(),
            description: "Another test project".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            avatar_url: None,
        };

        assert!(ability.can(&Action::Read, &Resource::Organization(&org)));
        assert!(ability.can(&Action::Read, &Resource::Project(&own_project)));

        assert!(ability.cannot(&Action::Manage, &Resource::All));
        assert!(ability.cannot(&Action::Read, &Resource::Subject("User"))); // Member cannot read users

        let user_model = users::Model {
            id: member_id,
            email: "test@test.com".to_string(),
            name: None,
            password_hash: None,
            avatar_url: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        assert!(ability.can(&Action::Update, &Resource::User(&user_model)));
        let other_user_model = users::Model {
            id: other_user_id,
            ..user_model
        };
        assert!(ability.cannot(&Action::Update, &Resource::User(&other_user_model)));

        assert!(ability.can(&Action::Manage, &Resource::Project(&own_project)));
        assert!(ability.can(&Action::Delete, &Resource::Project(&own_project)));

        assert!(ability.cannot(&Action::Update, &Resource::Project(&other_project)));
        assert!(ability.cannot(&Action::Delete, &Resource::Project(&other_project)));

        let invite_for_member = invites::Model {
            id: Uuid::new_v4(),
            author_id: Some(member_id),
            organization_id: org.id,
            email: "new@member.com".to_string(),
            role: Role::Member,
            created_at: Default::default(),
        };
        assert!(ability.can(&Action::Create, &Resource::Invite(&invite_for_member)));

        let invite_for_admin = invites::Model {
            role: Role::Admin,
            ..invite_for_member
        };
        assert!(ability.cannot(&Action::Create, &Resource::Invite(&invite_for_admin)));
    }

    #[test]
    fn test_billing_permissions() {
        let billing_id = Uuid::new_v4();
        let ability = get_user_permission(billing_id, Role::Billing);
        let owner_id = Uuid::new_v4();
        let org = organizations::Model {
            id: Uuid::new_v4(),
            owner_id,
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            domain: None,
            should_attach_users_by_domain: false,
            avatar_url: None,
        };
        let project = projects::Model {
            id: Uuid::new_v4(),
            owner_id,
            organization_id: org.id,
            name: "Test Project".to_string(),
            slug: "test-project".to_string(),
            description: "A test project".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            avatar_url: None,
        };

        assert!(ability.can(&Action::Read, &Resource::Organization(&org)));

        assert!(ability.cannot(&Action::Update, &Resource::Organization(&org)));
        assert!(ability.cannot(&Action::Manage, &Resource::All));
        assert!(ability.cannot(&Action::Create, &Resource::Project(&project)));
    }
}