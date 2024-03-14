/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin/urls.rs
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
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use csrf::CsrfProtection;
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{debug, info};

use entity::url;
use service::{Mutation, Query};

use crate::{
    auth::AuthSession, csrf as csrf_crate, err::AppError, state::AppState, util::format,
};

#[derive(Template)]
#[template(path = "admin/urls.html")]
struct UrlsTemplate<'a> {
    authenticity_token: &'a str,
    messages: Vec<Message>,
    sitename: &'a str,
    urls: Vec<url::Model>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeleteForm {
    authenticity_token: String,
    id: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/urls", get(self::get::urls))
        .route("/admin/urls/delete", post(self::post::delete))
}

mod post {
    use super::{
        csrf_crate, info, AppError, AppState, AuthSession, DeleteForm, Form, IntoResponse,
        Messages, Mutation, Redirect, Response, Session, State,
    };

    pub(super) async fn delete(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(delete_form): Form<DeleteForm>,
    ) -> Result<Response, AppError> {
        csrf_crate::verify(&session, &delete_form.authenticity_token, &state.protect).await?;

        if auth_session.user.is_none() {
            return Err(AppError::Unauthorized);
        }

        Mutation::delete_url(&state.db, delete_form.id).await?;

        info!("Deleted URL ID # {}", delete_form.id);
        messages.success(format!("Deleted URL #{} successfully", delete_form.id));
        Ok(Redirect::to("/admin/urls").into_response())
    }
}

mod get {
    use super::{
        debug, AppError, AppState, AuthSession, CsrfProtection, IntoResponse, Messages, Query,
        Response, Session, State, UrlsTemplate,
    };

    pub(super) async fn urls(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        if auth_session.user.is_none() {
            return Err(AppError::Unauthorized);
        }

        let (authenticity_token, session_token) = state.protect.generate_token_pair(None, 300)?;

        let authenticity_token = authenticity_token.b64_string();
        let session_token = session_token.b64_string();

        session.insert("authenticity_token", &session_token).await?;

        let urls = Query::fetch_all_urls(&state.db).await?;

        debug!(
            "Fetching all URLs for {}",
            auth_session.user.unwrap().0.username
        );

        Ok(UrlsTemplate {
            authenticity_token: &authenticity_token,
            messages: messages.into_iter().collect(),
            sitename: &state.env.sitename,
            urls,
        }
        .into_response())
    }
}
