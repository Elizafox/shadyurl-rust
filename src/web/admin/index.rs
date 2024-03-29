/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin/index.rs
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

use askama_axum::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_messages::{Message, Messages};
use tracing::{debug, warn};

use crate::{auth::AuthSession, err::AppError, state::AppState};

#[derive(Template)]
#[template(path = "admin/index.html")]
struct IndexTemplate<'a> {
    messages: Vec<Message>,
    sitename: &'a str,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/admin", get(self::get::index))
}

mod get {
    use super::{
        debug, warn, AppError, AppState, AuthSession, IndexTemplate, IntoResponse, Messages,
        Response, State,
    };

    pub(super) async fn index(
        messages: Messages,
        auth_session: AuthSession,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        let Some(user) = auth_session.user else {
            warn!("Unauthorized attempt to access admin page");
            return Err(AppError::Unauthorized);
        };

        debug!("Showing index to {}", user.0.username);

        Ok(IndexTemplate {
            messages: messages.into_iter().collect(),
            sitename: &state.env.sitename,
        }
        .into_response())
    }
}
