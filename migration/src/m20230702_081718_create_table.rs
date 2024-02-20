/* SPDX-License-Identifier: CC0-1.0
 *
 * migration/src/m20230702_081718_create_table.rs
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

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum Url {
    Table,
    Id,
    Url,
    Shady,
    CreatedAt,
    Ip,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Url::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Url::Id)
                            .integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Url::Url).string().not_null())
                    .col(ColumnDef::new(Url::Shady).string().not_null())
                    .col(
                        ColumnDef::new(Url::CreatedAt)
                            .date_time()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Url::Ip).string())
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                sea_query::Index::drop()
                    .table(Url::Table)
                    .name("idx-url-shady")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Url::Table).to_owned())
            .await?;

        Ok(())
    }
}
