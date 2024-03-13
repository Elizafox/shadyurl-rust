/* SPDX-License-Identifier: CC0-1.0
 *
 * migration/src/m20240312_184421_create_ip_filter_table.rs
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
                    .table(CidrBan::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CidrBan::Id)
                        .big_integer()
                        .primary_key()
                        .not_null()
                        .auto_increment()
                    )
                    .col(binary_len(CidrBan::RangeBegin, 16).unique_key().not_null())
                    .col(binary_len(CidrBan::RangeEnd, 16).unique_key().not_null())
                    .col(string(CidrBan::Reason).unique_key().not_null())
                    .col(big_integer(CidrBan::UserCreatedId))
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-cidr_ban-user_created")
                            .from(CidrBan::Table, CidrBan::UserCreatedId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .check(Expr::col(CidrBan::RangeBegin).lte(Expr::col(CidrBan::RangeEnd)))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(CidrBan::Table)
                    .name("idx-cidr_ban-range_begin-range_end")
                    .col(CidrBan::RangeBegin)
                    .col(CidrBan::RangeEnd)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(CidrBan::Table)
                    .name("idx-cidr_ban-range_begin-range_end")
                    .to_owned()
            ).await?;

        manager
            .drop_table(Table::drop().table(CidrBan::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CidrBan {
    Table,
    Id,
    RangeBegin,
    RangeEnd,
    Reason,
    UserCreatedId,
}

#[allow(clippy::enum_variant_names)]
#[derive(Iden)]
enum User {
    Table,
    Id,
}
