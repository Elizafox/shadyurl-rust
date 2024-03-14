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

use csrf::ChaCha20Poly1305CsrfProtection;
use sea_orm::DbConn;

use crate::{bancache::BanCache, env::Vars};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: Arc<DbConn>,
    pub(crate) env: Vars,
    pub(crate) protect: Arc<ChaCha20Poly1305CsrfProtection>,
    pub(crate) bancache: BanCache,
}
