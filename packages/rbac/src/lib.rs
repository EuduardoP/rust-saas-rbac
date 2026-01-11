use entities::sea_orm_active_enums::Role;
use entities::{invites, organizations, projects, users};
use uuid::Uuid;

// Step 1 & 4: Define base types and translate rules
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    Manage,
    Create,
    Read,
    Update,
    Delete,
    TransferOwnership,
}

// Step 2: Define Unified Resource
#[derive(Debug, Clone)]
pub enum Resource<'a> {
    Organization(&'a organizations::Model),
    Project(&'a projects::Model),
    Invite(&'a invites::Model),
    User(&'a users::Model),
    All,
}

// Step 3: Implement Authorization Logic
pub fn can(user_id: &Uuid, role: &Role, action: &Action, resource: &Resource) -> bool {
    match role {
        Role::Admin => can_admin(user_id, action, resource),
        Role::Member => can_member(user_id, action, resource),
        Role::Billing => can_billing(action, resource),
    }
}

// Logic for ADMIN (mapped from MANAGER)
fn can_admin(user_id: &Uuid, action: &Action, resource: &Resource) -> bool {
    match (action, resource) {
        // Deny creating invites for roles other than member
        (Action::Create, Resource::Invite(invite)) if invite.role != Role::Member => false,
        // Deny updating/transferring org if not owner
        (Action::Update, Resource::Organization(org)) if org.owner_id != *user_id => false,
        (Action::TransferOwnership, Resource::Organization(org)) if org.owner_id != *user_id => {
            false
        }
        // Allow everything else
        _ => true,
    }
}

// Logic for MEMBER (mapped from CANDIDATE)
fn can_member(user_id: &Uuid, action: &Action, resource: &Resource) -> bool {
    match (action, resource) {
        // Can read Projects and Organizations
        (Action::Read, Resource::Project(_)) => true,
        (Action::Read, Resource::Organization(_)) => true,

        // Can update own user info
        (Action::Update, Resource::User(user)) if user.id == *user_id => true,
        
        // Can manage own projects
        (Action::Manage, Resource::Project(project)) if project.owner_id == *user_id => true,
        (Action::Create, Resource::Project(_)) => true,
        (Action::Update, Resource::Project(project)) if project.owner_id == *user_id => true,
        (Action::Delete, Resource::Project(project)) if project.owner_id == *user_id => true,

        // Can create invites for other members
        (Action::Create, Resource::Invite(invite)) if invite.role == Role::Member => true,

        // Deny everything else
        _ => false,
    }
}

// Logic for BILLING
fn can_billing(action: &Action, resource: &Resource) -> bool {
    match (action, resource) {
        // Can read organization details for billing purposes
        (Action::Read, Resource::Organization(_)) => true,
        // Can manage billing-related settings (hypothetically, if they existed)
        // (Action::Manage, Resource::BillingSettings(_)) => true,
        _ => false,
    }
}


// Step 5: Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use entities::sea_orm_active_enums::Role;
    use uuid::Uuid;

    #[test]
    fn test_admin_permissions() {
        let admin_id = Uuid::new_v4();
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

        // Admin can do almost anything
        assert!(can(&admin_id, &Role::Admin, &Action::Manage, &Resource::All));
        assert!(can(&admin_id, &Role::Admin, &Action::Create, &Resource::Project(&project)));

        // Except update an org they don't own
        assert!(!can(&admin_id, &Role::Admin, &Action::Update, &Resource::Organization(&org_owned_by_other)));
        
        // Or transfer ownership
        assert!(!can(&admin_id, &Role::Admin, &Action::TransferOwnership, &Resource::Organization(&org_owned_by_other)));

        // But can if they are the owner
        assert!(can(&admin_id, &Role::Admin, &Action::Update, &Resource::Organization(&org_owned_by_admin)));

        // And cannot create an invite for an Admin
        let invite_for_admin = invites::Model {
            id: Uuid::new_v4(),
            author_id: Some(admin_id),
            organization_id: org_owned_by_admin.id,
            email: "new@member.com".to_string(),
            role: Role::Admin,
            created_at: Default::default(),
        };
        assert!(!can(&admin_id, &Role::Admin, &Action::Create, &Resource::Invite(&invite_for_admin)));
        
        // But can for a Member
        let invite_for_member = invites::Model {
            role: Role::Member,
            ..invite_for_admin
        };
        assert!(can(&admin_id, &Role::Admin, &Action::Create, &Resource::Invite(&invite_for_member)));
    }

    #[test]
    fn test_member_permissions() {
        let member_id = Uuid::new_v4();
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

        // Members can read projects and orgs
        assert!(can(&member_id, &Role::Member, &Action::Read, &Resource::Organization(&org)));
        assert!(can(&member_id, &Role::Member, &Action::Read, &Resource::Project(&own_project)));

        // But cannot manage all
        assert!(!can(&member_id, &Role::Member, &Action::Manage, &Resource::All));

        // Can update their own user model
        let user_model = users::Model {
            id: member_id,
            email: "test@test.com".to_string(),
            name: None,
            password_hash: None,
            avatar_url: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        assert!(can(&member_id, &Role::Member, &Action::Update, &Resource::User(&user_model)));
        let other_user_model = users::Model {
            id: other_user_id,
            ..user_model
        };
        assert!(!can(&member_id, &Role::Member, &Action::Update, &Resource::User(&other_user_model)));

        // Can manage their own project
        assert!(can(&member_id, &Role::Member, &Action::Manage, &Resource::Project(&own_project)));
        assert!(can(&member_id, &Role::Member, &Action::Delete, &Resource::Project(&own_project)));
        
        // But not others' projects
        assert!(!can(&member_id, &Role::Member, &Action::Update, &Resource::Project(&other_project)));
        assert!(!can(&member_id, &Role::Member, &Action::Delete, &Resource::Project(&other_project)));

        // Can create invites for other Members
        let invite_for_member = invites::Model {
            id: Uuid::new_v4(),
            author_id: Some(member_id),
            organization_id: org.id,
            email: "new@member.com".to_string(),
            role: Role::Member,
            created_at: Default::default(),
        };
        assert!(can(&member_id, &Role::Member, &Action::Create, &Resource::Invite(&invite_for_member)));

        // Cannot create invites for Admins
        let invite_for_admin = invites::Model {
            role: Role::Admin,
            ..invite_for_member
        };
        assert!(!can(&member_id, &Role::Member, &Action::Create, &Resource::Invite(&invite_for_admin)));
    }

    #[test]
    fn test_billing_permissions() {
        let billing_id = Uuid::new_v4();
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

        // Can read organization
        assert!(can(&billing_id, &Role::Billing, &Action::Read, &Resource::Organization(&org)));

        // Cannot do much else
        assert!(!can(&billing_id, &Role::Billing, &Action::Update, &Resource::Organization(&org)));
        assert!(!can(&billing_id, &Role::Billing, &Action::Manage, &Resource::All));
        assert!(!can(&billing_id, &Role::Billing, &Action::Create, &Resource::Project(&project)));
    }
}