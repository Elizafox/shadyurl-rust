use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Url {
    Table,
    Shady,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the old index and create a new one with the correct constraints
        manager
            .drop_index(
                sea_query::Index::drop()
                    .table(Url::Table)
                    .name("idx-url-shady")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx-url-shady")
                    .table(Url::Table)
                    .col(Url::Shady)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the index and create a new one without the unique constraint
        manager
            .drop_index(
                sea_query::Index::drop()
                    .table(Url::Table)
                    .name("idx-url-shady")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx-url-shady")
                    .table(Url::Table)
                    .col(Url::Shady)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
