/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin.rs
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

mod auth;
mod cidr_ban;
mod index;
mod url_filter;
mod urls;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(cidr_ban::router())
        .merge(index::router())
        .merge(urls::router())
        .merge(url_filter::router())
}
