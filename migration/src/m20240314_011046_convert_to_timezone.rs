use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DbBackend::Postgres | DbBackend::MySql => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Url::Table)
                            .modify_column(
                                ColumnDef::new(Url::CreatedAt)
                                    .timestamp_with_time_zone()
                                    .not_null()
                                    .default(Expr::current_timestamp()),
                            )
                            .to_owned(),
                    )
                    .await
            }
            DbBackend::Sqlite => {
                // Punt. It's not worth it.
                Ok(())
            }
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            DbBackend::Postgres | DbBackend::MySql => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Url::Table)
                            .modify_column(
                                ColumnDef::new(Url::CreatedAt)
                                    .date_time()
                                    .not_null()
                                    .default(Expr::current_timestamp()),
                            )
                            .to_owned(),
                    )
                    .await
            }
            DbBackend::Sqlite => {
                // Punt. It's not worth it.
                Ok(())
            }
        }
    }
}

#[derive(DeriveIden)]
enum Url {
    Table,
    CreatedAt,
}
