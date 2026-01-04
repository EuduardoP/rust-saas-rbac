use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use entities::{
    members, organizations, projects,
    sea_orm_active_enums::{self},
    users,
};
use fake::{
    faker::{internet::en::*, name::en::*},
    Fake, Faker,
};
use rand::seq::IndexedRandom;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // DELETING EXISTING DATA
        members::Entity::delete_many().exec(db).await?;
        projects::Entity::delete_many().exec(db).await?;
        organizations::Entity::delete_many().exec(db).await?;
        users::Entity::delete_many().exec(db).await?;

        // HASHING PASSWORD
        let password_hash = argon2
            .hash_password("123456".as_bytes(), &salt)
            .unwrap()
            .to_string();

        // CREATING USERS
        let john_doe = users::ActiveModel {
            name: Set(Some("John Doe".to_owned())),
            email: Set("john@acme.com".to_owned()),
            password_hash: Set(Some(password_hash.clone())),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let another_user = users::ActiveModel {
            name: Set(Some(Name().fake())),
            email: Set(FreeEmail().fake()),
            password_hash: Set(Some(password_hash.clone())),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let another_user2 = users::ActiveModel {
            name: Set(Some(Name().fake())),
            email: Set(FreeEmail().fake()),
            password_hash: Set(Some(password_hash.clone())),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let user_ids = [john_doe.id, another_user.id, another_user2.id];

        // CREATING ORGANIZATIONS
        let acme_admin_org = organizations::ActiveModel {
            name: Set("Acme Inc (Admin)".to_owned()),
            domain: Set(Some("acme.com".to_owned())),
            slug: Set("acme-admin".to_owned()),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            should_attach_users_by_domain: Set(true),
            owner_id: Set(john_doe.id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for _ in 0..3 {
            let project_owner_id = *user_ids.choose(&mut rand::rng()).unwrap();
            projects::ActiveModel {
                name: Set(Faker.fake::<String>()),
                slug: Set(Faker.fake::<String>()),
                description: Set(Faker.fake::<String>()),
                avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
                organization_id: Set(acme_admin_org.id),
                owner_id: Set(project_owner_id),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        members::ActiveModel {
            organization_id: Set(acme_admin_org.id),
            user_id: Set(john_doe.id),
            role: Set(sea_orm_active_enums::Role::Admin),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_admin_org.id),
            user_id: Set(another_user.id),
            role: Set(sea_orm_active_enums::Role::Member),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_admin_org.id),
            user_id: Set(another_user2.id),
            role: Set(sea_orm_active_enums::Role::Member),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let acme_billing_org = organizations::ActiveModel {
            name: Set("Acme Inc (Billing)".to_owned()),
            slug: Set("acme-billing".to_owned()),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            owner_id: Set(john_doe.id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for _ in 0..3 {
            let project_owner_id = *user_ids.choose(&mut rand::rng()).unwrap();
            projects::ActiveModel {
                name: Set(Faker.fake::<String>()),
                slug: Set(Faker.fake::<String>()),
                description: Set(Faker.fake::<String>()),
                avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
                organization_id: Set(acme_billing_org.id),
                owner_id: Set(project_owner_id),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        members::ActiveModel {
            organization_id: Set(acme_billing_org.id),
            user_id: Set(john_doe.id),
            role: Set(sea_orm_active_enums::Role::Billing),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_billing_org.id),
            user_id: Set(another_user.id),
            role: Set(sea_orm_active_enums::Role::Admin),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_billing_org.id),
            user_id: Set(another_user2.id),
            role: Set(sea_orm_active_enums::Role::Member),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let acme_member_org = organizations::ActiveModel {
            name: Set("Acme Inc (Member)".to_owned()),
            slug: Set("acme-member".to_owned()),
            avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
            owner_id: Set(john_doe.id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for _ in 0..3 {
            let project_owner_id = *user_ids.choose(&mut rand::rng()).unwrap();
            projects::ActiveModel {
                name: Set(Faker.fake::<String>()),
                slug: Set(Faker.fake::<String>()),
                description: Set(Faker.fake::<String>()),
                avatar_url: Set(Some("https://avatar.iran.liara.run/public".to_owned())),
                organization_id: Set(acme_member_org.id),
                owner_id: Set(project_owner_id),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        members::ActiveModel {
            organization_id: Set(acme_member_org.id),
            user_id: Set(john_doe.id),
            role: Set(sea_orm_active_enums::Role::Member),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_member_org.id),
            user_id: Set(another_user.id),
            role: Set(sea_orm_active_enums::Role::Admin),
            ..Default::default()
        }
        .insert(db)
        .await?;

        members::ActiveModel {
            organization_id: Set(acme_member_org.id),
            user_id: Set(another_user2.id),
            role: Set(sea_orm_active_enums::Role::Member),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        members::Entity::delete_many().exec(db).await?;
        projects::Entity::delete_many().exec(db).await?;
        organizations::Entity::delete_many().exec(db).await?;
        users::Entity::delete_many().exec(db).await?;
        Ok(())
    }
}
