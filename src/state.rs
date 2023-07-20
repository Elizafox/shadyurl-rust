/* SPDX-License-Identifier: CC0-1.0
 *
 * src/state.rs
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

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{auth::User, loadenv::EnvVars};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) sitename: String,
    pub(crate) hostname: String,
    pub(crate) user: User,
}

impl AppState {
    pub(crate) fn new_from_env(db: DatabaseConnection, env: &EnvVars) -> Self {
        let user = User {
            id: 1,
            username: env.username.clone(),
            password_hash: Arc::new(env.password_hash.clone()),
        };

        Self {
            db,
            sitename: env.sitename.clone(),
            hostname: env.hostname.clone(),
            user,
        }
    }
}
