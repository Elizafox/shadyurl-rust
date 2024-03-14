/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/files.rs
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

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::state::AppState;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/ads.txt", ServeFile::new("static/ads.txt"))
        .nest_service("/app-ads.txt", ServeFile::new("static/app-ads.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        .nest_service("/assets", ServeDir::new("static/assets"))
}
