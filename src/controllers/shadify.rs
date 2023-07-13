/* SPDX-License-Identifier: CC0-1.0
 *
 * src/controllers/shadify.rs
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

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Result},
    Form,
};
use axum_csrf::CsrfToken;
use axum_login::axum_sessions::extractors::WritableSession;
use lazy_static::lazy_static;
use log::debug;
use rand::thread_rng;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tokio::{sync::Semaphore, task::spawn_blocking};
use validator::Validate;

use crate::{
    err::{respond_internal_server_error, respond_not_authorised, respond_not_found},
    generate::shady_filename,
    templates::{IndexTemplate, PostErrorTemplate, PostTemplate},
    validators::validate_url,
    AppState,
};

use entity::{prelude::*, url as url_db};

lazy_static! {
    static ref SHADY_FILE_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(num_cpus::get()));
}

#[derive(Deserialize, Validate)]
pub(crate) struct UrlPayload {
    #[validate(custom = "validate_url")]
    url: String,
    auth_token: String,
}

pub(crate) async fn root(
    token: CsrfToken,
    mut session: WritableSession,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let auth_token = token.authenticity_token()
        .map_err(|_| respond_not_authorised())?;
    session
        .insert("auth_token", auth_token.clone())
        .map_err(|_| respond_not_authorised())?;

    let t = IndexTemplate {
        sitename: &state.sitename,
        auth_token: &auth_token,
    };

    Ok((token, t).into_response())
}

pub(crate) async fn accept_form(
    token: CsrfToken,
    mut session: WritableSession,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<UrlPayload>,
) -> Result<impl IntoResponse> {
    token
        .verify(&payload.auth_token)
        .map_err(|_| respond_not_authorised())?;

    let auth_token = session
        .get::<String>("auth_token")
        .ok_or(respond_not_authorised())?;

    // Trash auth token
    session.remove("auth_token");

    token
        .verify(&auth_token)
        .map_err(|_| respond_not_authorised())?;

    let url = &payload.url;

    if let Err(e) = payload.validate() {
        let reason = e
            .field_errors()
            .get("url")
            .map_or("Unknown error".to_string(), |v| v[0].code.to_string());
        debug!("User attempted to put in invalid URL: \"{url}\", reason: {reason}");
        let t = PostErrorTemplate {
            url,
            reason: &reason,
        };
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, t).into_response());
    }

    let permit = SHADY_FILE_SEMAPHORE.clone().acquire_owned().await.unwrap();
    let shady = spawn_blocking(move || {
        let result = shady_filename(&mut thread_rng());
        drop(permit);
        result
    })
    .await
    .map_err(|_| respond_internal_server_error())?;

    let ip = addr.ip().to_string();

    let url_db_obj = url_db::ActiveModel {
        url: Set(url.to_string()),
        shady: Set(shady.to_string()),
        ip: Set(Some(ip)),
        ..Default::default()
    };
    url_db_obj
        .insert(&state.db)
        .await
        .map_err(|_| respond_internal_server_error())?;

    let t = PostTemplate {
        hostname: &state.hostname,
        url,
        shady: &shady,
    };
    Ok((StatusCode::OK, t).into_response())
}

pub(crate) async fn get_shady(
    State(state): State<AppState>,
    Path(shady): Path<String>,
) -> Result<impl IntoResponse> {
    let result = Url::find()
        .filter(url_db::Column::Shady.eq(shady))
        .one(&state.db)
        .await
        .map_err(|_| respond_internal_server_error())?;
    Ok(match result {
        Some(row) => Redirect::to(&row.url).into_response(),
        None => respond_not_found(),
    })
}
