use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Event::Table)
                    .add_column(json(Event::EventType))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Event::Table)
                    .drop_column(Event::EventType)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
#[allow(dead_code)]
enum Event {
    Table,
    Id,
    Name,
    CreatedAt,
    VisitedAt,
    Times,
    Timezone,
    #[expect(clippy::enum_variant_names)]
    EventType,
}
