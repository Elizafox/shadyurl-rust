/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/fallback.rs
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

use axum::{response::IntoResponse, Router};

use crate::{err::ErrorResponse, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().fallback(get::fallback)
}

mod get {
    use super::{ErrorResponse, IntoResponse};

    pub(super) async fn fallback() -> impl IntoResponse {
        ErrorResponse::not_found()
    }
}
