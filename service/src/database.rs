/* SPDX-License-Identifier: CC0-1.0
 *
 * service/src/database.rs
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

// Database connection services.

use sea_orm::{ConnectOptions, DbConn, DbErr};
use tracing::log::LevelFilter;

use migration::{Migrator, MigratorTrait};

pub struct Database;

impl Database {
    // Get a DbConn with the given connection options.
    pub async fn get_with_connect_options(opt: ConnectOptions) -> Result<DbConn, DbErr> {
        let db = sea_orm::Database::connect(opt).await?;

        Migrator::up(&db, None).await?;

        Ok(db)
    }

    // Get a DbConn with some default options.
    pub async fn get(url: &str) -> Result<DbConn, DbErr> {
        let mut opt = ConnectOptions::new(url);
        opt.sqlx_logging(false)
            .sqlx_logging_level(LevelFilter::Warn);

        let db = Self::get_with_connect_options(opt).await?;
        Ok(db)
    }
}
