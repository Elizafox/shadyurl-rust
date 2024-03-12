/* SPDX-License-Identifier: CC0-1.0
 *
 * migration/src/m20230713_214521_add_unique_constraint.rs
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
