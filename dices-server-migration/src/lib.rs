pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241023_221153_create_table_users::Migration),
            Box::new(m20241101_171000_create_table_sessions::Migration),
        ]
    }
}
mod m20241023_221153_create_table_users;
mod m20241101_171000_create_table_sessions;
