/* SPDX-License-Identifier: CC0-1.0
 *
 * migration/src/m20240312_050549_create_url_filter_table.rs
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UrlFilter::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UrlFilter::Id)
                            .big_integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(string(UrlFilter::Filter).not_null().unique_key())
                    .col(string(UrlFilter::Reason))
                    .col(
                        ColumnDef::new(UrlFilter::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(big_integer(UrlFilter::UserCreatedId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-url_filter-user_created")
                            .from(UrlFilter::Table, UrlFilter::UserCreatedId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UrlFilter::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UrlFilter {
    Table,
    Id,
    Filter,
    Reason,
    CreatedAt,
    UserCreatedId,
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum User {
    Table,
    Id,
}
