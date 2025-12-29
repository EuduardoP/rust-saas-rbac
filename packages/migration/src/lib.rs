pub use sea_orm_migration::prelude::*;

mod m20251229_041332_create_user_and_enums;
mod m20251229_042509_create_auth_tables;
mod m20251229_043950_create_organization_table;
mod m20251229_044000_create_project_table;
mod m20251229_044017_create_member_invite_table;
mod m20251229_052055_seed_data;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251229_041332_create_user_and_enums::Migration),
            Box::new(m20251229_042509_create_auth_tables::Migration),
            Box::new(m20251229_043950_create_organization_table::Migration),
            Box::new(m20251229_044000_create_project_table::Migration),
            Box::new(m20251229_044017_create_member_invite_table::Migration),
            Box::new(m20251229_052055_seed_data::Migration),
        ]
    }
}
