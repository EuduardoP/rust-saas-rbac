use sea_orm::Iterable;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20251229_041332_create_user_and_enums::{Role, Users},
    m20251229_043950_create_organization_table::Organizations,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Tabela Members
        manager
            .create_table(
                table_auto(Members::Table)
                    .col(pk_uuid(Members::Id).default(Expr::cust("gen_random_uuid()")))
                    // Role tem default(MEMBER). O helper 'enumeration' não aceita default facilmente, então usamos ColumnDef manual
                    .col(
                        ColumnDef::new(Members::Role)
                            .custom(Alias::new("role"))
                            .not_null()
                            .default("MEMBER"),
                    )
                    .col(uuid(Members::OrganizationId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Members::Table, Members::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(Members::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Members::Table, Members::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // @@unique([organizationId, userId])
                    .index(
                        Index::create()
                            .name("members_org_user_unique")
                            .table(Members::Table)
                            .col(Members::OrganizationId)
                            .col(Members::UserId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Tabela Invites
        manager
            .create_table(
                Table::create()
                    .table(Invites::Table)
                    .col(pk_uuid(Invites::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(Invites::Email))
                    .col(enumeration(Invites::Role, Alias::new("role"), Role::iter()))
                    .col(
                        timestamp_with_time_zone(Invites::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(uuid_null(Invites::AuthorId)) // Nullable
                    .foreign_key(
                        ForeignKey::create()
                            .from(Invites::Table, Invites::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(uuid(Invites::OrganizationId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Invites::Table, Invites::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // @@unique([email, organizationId])
                    .index(
                        Index::create()
                            .name("invites_email_org_unique")
                            .table(Invites::Table)
                            .col(Invites::Email)
                            .col(Invites::OrganizationId)
                            .unique(),
                    )
                    // @@index([email])
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("invites_email_idx")
                    .table(Invites::Table)
                    .col(Invites::Email)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invites::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Members::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Members {
    Table,
    Id,
    Role,
    OrganizationId,
    UserId,
}

#[derive(DeriveIden)]
enum Invites {
    Table,
    Id,
    Email,
    Role,
    CreatedAt,
    AuthorId,
    OrganizationId,
}


