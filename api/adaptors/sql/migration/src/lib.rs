pub use sea_orm_migration::prelude::*;

mod m20260508_204615_setup_tables;
mod m20260509_224458_day_based_events;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260508_204615_setup_tables::Migration),
            Box::new(m20260509_224458_day_based_events::Migration),
        ]
    }
}
