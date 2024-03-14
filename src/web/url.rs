/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/url.rs
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

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use itertools::join;

use service::Query;

use crate::{error_response::AppError, state::AppState};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/reverse-map/*url", get(self::get::url))
        .route("/*shady", get(self::get::shady))
}

mod get {
    use super::{join, AppError, AppState, IntoResponse, Path, Query, Redirect, Response, State};

    pub(super) async fn url(
        Path(url): Path<String>,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        Ok(join(
            Query::find_url_by_string(&state.db, &url)
                .await?
                .iter()
                .map(|u| &u.shady),
            "\n",
        )
        .into_response())
    }

    pub(super) async fn shady(
        Path(shady): Path<String>,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        Query::find_url_by_shady_string(&state.db, &shady)
            .await?
            .map_or_else(
                || Err(AppError::NotFound),
                |url| Ok(Redirect::to(&url.url).into_response()),
            )
    }
}
