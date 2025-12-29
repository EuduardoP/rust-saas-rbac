use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Criar Enums
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("token_type"))
                    .values(TokenType::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("account_provider"))
                    .values(AccountProvider::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("role"))
                    .values(Role::iter())
                    .to_owned(),
            )
            .await?;

        // 2. Criar tabela Users
        manager
            .create_table(
                table_auto(Users::Table)
                    .col(pk_uuid(Users::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string_null(Users::Name))
                    .col(string(Users::Email).unique_key())
                    .col(string_null(Users::PasswordHash))
                    .col(string_null(Users::AvatarUrl))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("role")).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("account_provider")).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("token_type")).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Name,
    Email,
    PasswordHash,
    AvatarUrl,
}

#[derive(Iden, EnumIter)]
pub enum TokenType {
    #[iden = "PASSWORD_RECOVER"]
    PasswordRecover,
}

#[derive(Iden, EnumIter)]
pub enum AccountProvider {
    #[iden = "GITHUB"]
    Github,
    #[iden = "GOOGLE"]
    Google,
}

#[derive(Iden, EnumIter)]
pub enum Role {
    #[iden = "ADMIN"]
    Admin,
    #[iden = "MEMBER"]
    Member,
    #[iden = "BILLING"]
    Billing,
}
