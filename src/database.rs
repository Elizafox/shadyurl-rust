/* SPDX-License-Identifier: CC0-1.0
 *
 * src/database.rs
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

use anyhow::{Error, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::time::Duration;

use crate::loadenv::EnvVars;

pub(crate) async fn get_db(env: &EnvVars) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(env.database_url().to_string());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(false);

    let db = Database::connect(opt)
        .await
        .map_err(|e| Error::new(e).context("Could not open database"))?;

    Migrator::up(&db, None)
        .await
        .map_err(|e| Error::new(e).context("Could not migrate database"))?;

    Ok(db)
}
