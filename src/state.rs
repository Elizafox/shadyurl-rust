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

use axum::extract::FromRef;
use axum_csrf::{CsrfConfig, Key};
use sea_orm::DatabaseConnection;

// XXX User shouldn't live in the controller
use crate::{controllers::User, loadenv::EnvVars};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) sitename: String,
    pub(crate) hostname: String,
    pub(crate) user: User,
    pub(crate) csrf_config: CsrfConfig,
}

impl FromRef<AppState> for CsrfConfig {
    fn from_ref(app_state: &AppState) -> CsrfConfig {
        app_state.csrf_config.clone()
    }
}

impl AppState {
    pub(crate) fn new_from_env(db: DatabaseConnection, env: &EnvVars) -> Self {
        let cookie_key = Key::from(env.secret());
        let csrf_config = CsrfConfig::default().with_key(Some(cookie_key));

        let user = User {
            id: 1,
            username: env.username().to_string(),
            password_hash: Arc::new(env.password_hash().clone()),
        };

        Self {
            db,
            sitename: env.sitename().to_string(),
            hostname: env.hostname().to_string(),
            user,
            csrf_config,
        }
    }
}
