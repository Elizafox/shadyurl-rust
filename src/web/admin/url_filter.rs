/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin/url_filter.rs
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
use regex::Regex;
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;

use entity::{url_filter, user};
use service::{Mutation, Query};

use crate::{
    auth::AuthSession, csrf as csrf_crate, error_response::AppError, state::AppState, util::format,
};

#[derive(Template)]
#[template(path = "admin/url_filter.html")]
struct UrlFiltersTemplate<'a> {
    authenticity_token: &'a str,
    messages: Vec<Message>,
    sitename: &'a str,
    url_filters: Vec<(url_filter::Model, Option<user::Model>)>,
}

#[derive(Debug, Clone, Deserialize)]
struct SubmitFilterForm {
    authenticity_token: String,
    filter: String,
    reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeleteForm {
    authenticity_token: String,
    id: i64,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/url_filters", get(self::get::url_filters))
        .route("/admin/url_filters", post(self::post::url_filters))
        .route("/admin/url_filters/delete", post(self::post::delete))
}

mod post {
    use super::{
        csrf_crate, AppError, AppState, AuthSession, DeleteForm, Form, IntoResponse, Messages,
        Mutation, Redirect, Regex, Response, Session, State, SubmitFilterForm,
    };

    pub(super) async fn url_filters(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(submit_filter_form): Form<SubmitFilterForm>,
    ) -> Result<Response, AppError> {
        csrf_crate::verify(
            &session,
            &submit_filter_form.authenticity_token,
            &state.protect,
        )
        .await?;

        let Some(user) = auth_session.user else {
            return Err(AppError::Unauthorized);
        };

        if submit_filter_form.filter.is_empty() {
            messages.error("Filter cannot be empty");
            return Ok(Redirect::to("/admin/url_filters").into_response());
        }

        if let Err(e) = Regex::new(&submit_filter_form.filter) {
            messages.error(format!(
                "Malformed URL filter regex {}: {e}",
                submit_filter_form.filter
            ));
            return Ok(Redirect::to("/admin/url_filters").into_response());
        }

        Mutation::create_url_filter(
            &state.db,
            submit_filter_form.filter.clone(),
            submit_filter_form.reason,
            &user.0,
        )
        .await?;

        messages.success(format!(
            "Added filter {} successfullly",
            submit_filter_form.filter
        ));
        Ok(Redirect::to("/admin/url_filters").into_response())
    }

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
        };

        Mutation::delete_url_filter(&state.db, delete_form.id).await?;

        messages.success(format!(
            "Deleted URL filter #{} successfully",
            delete_form.id
        ));
        Ok(Redirect::to("/admin/url_filters").into_response())
    }
}

mod get {
    use super::{
        AppError, AppState, AuthSession, CsrfProtection, IntoResponse, Messages, Query, Response,
        Session, State, UrlFiltersTemplate,
    };

    pub(super) async fn url_filters(
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

        let url_filters = Query::fetch_all_url_filters(&state.db).await?;

        Ok(UrlFiltersTemplate {
            authenticity_token: &authenticity_token,
            messages: messages.into_iter().collect(),
            sitename: &state.env.sitename,
            url_filters,
        }
        .into_response())
    }
}
