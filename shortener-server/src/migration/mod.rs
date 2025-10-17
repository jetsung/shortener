use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_urls_table::Migration),
            Box::new(m20240101_000002_create_histories_table::Migration),
        ]
    }
}

mod m20240101_000001_create_urls_table;
mod m20240101_000002_create_histories_table;
