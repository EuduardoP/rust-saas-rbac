use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20251229_041332_create_user_and_enums::Users,
    m20251229_043950_create_organization_table::Organizations,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Projects::Table)
                    .col(pk_uuid(Projects::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(Projects::Name))
                    .col(string(Projects::Description))
                    .col(string(Projects::Slug).unique_key())
                    .col(string_null(Projects::AvatarUrl))
                    // Relations
                    .col(uuid(Projects::OrganizationId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Projects::Table, Projects::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(Projects::OwnerId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Projects::Table, Projects::OwnerId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    Name,
    Description,
    Slug,
    AvatarUrl,
    OrganizationId,
    OwnerId,
}
