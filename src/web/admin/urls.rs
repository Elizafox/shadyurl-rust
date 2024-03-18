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
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{debug, info};

use entity::url;
use service::{Mutation, Query};

use crate::{
    auth::AuthSession, csrf::CsrfSessionEntry, err::AppError, state::AppState, util::string,
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
        info, AppError, AppState, AuthSession, CsrfSessionEntry, DeleteForm, Form, IntoResponse,
        Messages, Mutation, Redirect, Response, Session, State,
    };

    pub(super) async fn delete(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(delete_form): Form<DeleteForm>,
    ) -> Result<Response, AppError> {
        CsrfSessionEntry::check_session(
            &state.csrf_crypto_engine,
            &session,
            &delete_form.authenticity_token,
        )
        .await?;

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
        debug, AppError, AppState, AuthSession, CsrfSessionEntry, IntoResponse, Messages, Query,
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

        let authenticity_token =
            CsrfSessionEntry::insert_session(&state.csrf_crypto_engine, &session).await?;

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
