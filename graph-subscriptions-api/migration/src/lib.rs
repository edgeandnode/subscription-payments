pub use sea_orm_migration::prelude::*;

mod m20230416_185838_create_table_subscription_query_result;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20230416_185838_create_table_subscription_query_result::Migration,
        )]
    }
}
