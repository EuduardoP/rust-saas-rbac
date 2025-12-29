use crate::m20251229_041332_create_user_and_enums::{AccountProvider, TokenType, Users};
use sea_orm::Iterable;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Tabela Tokens
        manager
            .create_table(
                Table::create()
                    .table(Tokens::Table)
                    .col(pk_uuid(Tokens::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(enumeration(
                        Tokens::Type,
                        Alias::new("token_type"),
                        TokenType::iter(),
                    ))
                    .col(
                        timestamp_with_time_zone(Tokens::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(uuid(Tokens::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tokens::Table, Tokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Tabela Accounts
        manager
            .create_table(
                table_auto(Accounts::Table)
                    .col(pk_uuid(Accounts::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(enumeration(
                        Accounts::Provider,
                        Alias::new("account_provider"),
                        AccountProvider::iter(),
                    ))
                    .col(string(Accounts::ProviderAccountId).unique_key())
                    .col(uuid(Accounts::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Accounts::Table, Accounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("accounts_provider_user_unique")
                            .table(Accounts::Table)
                            .col(Accounts::Provider)
                            .col(Accounts::UserId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tokens::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tokens {
    Table,
    Id,
    Type,
    CreatedAt,
    UserId,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Provider,
    ProviderAccountId,
    UserId,
}
