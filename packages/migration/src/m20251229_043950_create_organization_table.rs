use sea_orm_migration::{prelude::*, schema::*};

use crate::m20251229_041332_create_user_and_enums::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Organizations::Table)
                    .col(pk_uuid(Organizations::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(Organizations::Name))
                    .col(string(Organizations::Slug).unique_key())
                    .col(string_null(Organizations::Domain).unique_key())
                    .col(boolean(Organizations::ShouldAttachUsersByDomain).default(false))
                    .col(string_null(Organizations::AvatarUrl))
                    // Owner relation (É bom verificar sua lógica de negócio, onDelete)
                    .col(uuid(Organizations::OwnerId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Organizations::Table, Organizations::OwnerId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Organizations {
    Table,
    Id,
    Name,
    Slug,
    Domain,
    ShouldAttachUsersByDomain,
    AvatarUrl,
    OwnerId,
}

