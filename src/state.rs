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

// State for handlers and such

use std::sync::Arc;

use sea_orm::DbConn;

use crate::{bancache::BanCache, csrf::CryptoEngine, env::Vars, urlcache::UrlCache};

// This is the struct that holds state for handlers
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct AppState {
    // We can't keep references to DbConn, and this can span threads
    // So this needs to be an Arc.
    pub(crate) db: Arc<DbConn>,
    pub(crate) env: Vars,
    pub(crate) bancache: BanCache,
    pub(crate) urlcache: UrlCache,
    pub(crate) csrf_crypto_engine: CryptoEngine,
}
