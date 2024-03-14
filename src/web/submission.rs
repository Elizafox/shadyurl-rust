/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/submission.rs
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
    routing::{get, post},
    Form, Router,
};
use axum_client_ip::SecureClientIp;
use axum_messages::{Message, Messages};
use regex::Regex;
use serde::Deserialize;
use tracing::{debug, info};
use validator::Validate;

use service::{Mutation, Query};

use crate::{
    err::AppError, generate::Generator, state::AppState, validators::validate_url,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    messages: Vec<Message>,
    base_host: &'a str,
    sitename: &'a str,
}

#[derive(Template)]
#[template(path = "submit.html")]
struct SubmissionTemplate<'a> {
    messages: Vec<Message>,
    shady_host: &'a str,
    url: &'a str,
    shady: &'a str,
}

#[derive(Debug, Clone, Validate, Deserialize)]
struct UrlForm {
    #[validate(custom(function = validate_url))]
    pub(super) url: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(self::get::index))
        .route("/submit", post(self::post::submit))
}

mod get {
    use super::{AppState, IndexTemplate, IntoResponse, Messages, State};

    pub(super) async fn index(
        messages: Messages,
        State(state): State<AppState>,
    ) -> impl IntoResponse {
        IndexTemplate {
            messages: messages.into_iter().collect(),
            base_host: &state.env.base_host,
            sitename: &state.env.sitename,
        }
        .into_response()
    }
}

mod post {
    use super::{
        debug, info, AppError, AppState, Form, Generator, IntoResponse, Messages, Mutation, Query,
        Regex, Response, SecureClientIp, State, SubmissionTemplate, UrlForm, Validate,
    };

    pub(super) async fn submit(
        messages: Messages,
        SecureClientIp(addr): SecureClientIp,
        State(state): State<AppState>,
        Form(url_form): Form<UrlForm>,
    ) -> Result<Response, AppError> {
        if state.bancache.check_ban(addr).await? {
            info!(
                "Banned client ({addr}) attempted to submit url: {}",
                url_form.url
            );
            return Err(AppError::Unauthorized);
        }

        if let Err(e) = url_form.validate() {
            let error_reason = e
                .field_errors()
                .get("url")
                .map_or("Unknown error".to_string(), |v| v[0].code.to_string());
            debug!("Invalid URL submitted ({}): {error_reason}", url_form.url);
            return Err(AppError::UrlValidation(url_form.url, error_reason));
        }

        for url_filter in Query::fetch_all_url_filters(&state.db).await? {
            let creg = Regex::new(&url_filter.0.filter)?;
            if creg.is_match(&url_form.url) {
                return Err(AppError::UrlValidation(
                    url_form.url,
                    "URL is banned".to_string(),
                ));
            }
            info!(
                "Blacklisted URL submitted matching \"{}\": {}",
                url_filter.0.filter, url_form.url
            );
        }

        let shady = Generator::shady_filename();
        Mutation::create_url(&state.db, &url_form.url, &shady, Some(addr.to_string())).await?;

        debug!("URL created: {} -> {shady}", url_form.url);

        Ok(SubmissionTemplate {
            url: &url_form.url,
            shady: &shady,
            messages: messages.into_iter().collect(),
            shady_host: &state.env.shady_host,
        }
        .into_response())
    }
}
