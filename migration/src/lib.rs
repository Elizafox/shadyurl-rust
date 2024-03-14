/* SPDX-License-Identifier: CC0-1.0
 *
 * migration/src/lib.rs
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

pub use sea_orm_migration::prelude::*;

mod m20230702_081718_create_table;
mod m20230713_214521_add_unique_constraint;
mod m20240302_082409_create_user_table;
mod m20240312_050549_create_url_filter_table;
mod m20240312_184421_create_ip_filter_table;
mod m20240314_011046_convert_to_timezone;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230702_081718_create_table::Migration),
            Box::new(m20230713_214521_add_unique_constraint::Migration),
            Box::new(m20240302_082409_create_user_table::Migration),
            Box::new(m20240312_050549_create_url_filter_table::Migration),
            Box::new(m20240312_184421_create_ip_filter_table::Migration),
            Box::new(m20240314_011046_convert_to_timezone::Migration),
        ]
    }
}
