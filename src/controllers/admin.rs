/* SPDX-License-Identifier: CC0-1.0
 *
 * src/controllers/admin.rs
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

use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Result},
    Extension, Form,
};
use axum_client_ip::SecureClientIp;
use axum_csrf::CsrfToken;
use axum_login::axum_sessions::extractors::WritableSession;
use lazy_static::lazy_static;
use sea_orm::EntityTrait;
use serde::Deserialize;
use tokio::{sync::Semaphore, task::spawn_blocking};
use tracing::{error, warn};

use crate::{
    auth::{Auth, User},
    err::{respond_internal_server_error, respond_not_authorised},
    templates::{AdminTemplate, LoginTemplate},
    AppState,
};

use entity::prelude::*;

lazy_static! {
    static ref PASSWORD_HASH_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(num_cpus::get()));
    static ref TOKEN_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(num_cpus::get() * 2));
}

#[derive(Deserialize)]
pub(crate) struct LoginPayload {
    username: String,
    password: String,
    auth_token: String,
}

#[derive(Deserialize)]
pub(crate) struct DeletePayload {
    id: i64,
    auth_token: String,
}

pub(crate) async fn login_page_handler(
    token: CsrfToken,
    mut session: WritableSession,
    auth: Auth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    if auth.current_user.is_some() {
        return Ok(Redirect::to("/admin").into_response());
    }

    // Shove the auth token calculation into a thread
    // This is a CPU intensive operation and we don't wanna block everything.
    // Why the double map_err? we get a Result<Result<...>>.
    let permit = TOKEN_SEMAPHORE.clone().acquire_owned().await.unwrap();
    let t = token.clone();
    let auth_token = spawn_blocking(move || {
        let result = t.authenticity_token();
        drop(permit);
        result
    })
    .await
    .map_err(|e| {
        error!("Error spawning thread: {e}");
        respond_internal_server_error(&state)
    })?
    .map_err(|_| respond_not_authorised(&state))?;

    session
        .insert("auth_token", auth_token.clone())
        .map_err(|_| respond_not_authorised(&state))?;

    let err_str = session
        .get::<String>("login_error")
        .unwrap_or(String::new());

    // Remove stale login error
    session.remove("login_error");

    let t = LoginTemplate {
        base_host: &state.base_host,
        err_str: &err_str,
        sitename: &state.sitename,
        auth_token: &auth_token,
    };
    Ok((token, t).into_response())
}

pub(crate) async fn login_handler(
    token: CsrfToken,
    mut session: WritableSession,
    mut auth: Auth,
    SecureClientIp(addr): SecureClientIp,
    State(state): State<AppState>,
    Form(payload): Form<LoginPayload>,
) -> Result<impl IntoResponse> {
    // Shove the auth token calculation into a thread
    // This is a CPU intensive operation and we don't wanna block everything.
    let permit = TOKEN_SEMAPHORE.clone().acquire_owned().await.unwrap();
    let t = token.clone();
    let pt = payload.auth_token.clone();
    spawn_blocking(move || {
        let result = t.verify(&pt);
        drop(permit);
        result
    })
    .await
    .map_err(|e| {
        error!("Error spawning thread: {e}");
        respond_internal_server_error(&state)
    })?
    .map_err(|_| respond_not_authorised(&state))?;

    let auth_token = session
        .get::<String>("auth_token")
        .ok_or(respond_not_authorised(&state))?;

    // Trash previous auth token after looking
    session.remove("auth_token");

    // Same as above
    let permit = TOKEN_SEMAPHORE.clone().acquire_owned().await.unwrap();
    let t = token.clone();
    let pt = auth_token.clone();
    spawn_blocking(move || {
        let result = t.verify(&pt);
        drop(permit);
        result
    })
    .await
    .map_err(|e| {
        error!("Error spawning thread: {e}");
        respond_internal_server_error(&state)
    })?
    .map_err(|_| respond_not_authorised(&state))?;

    if payload.username != state.user.username {
        let ip = addr.to_string();
        warn!(
            "Login attempt from {ip} (username {}): No such username",
            &payload.username
        );
        // Save the error and redirect
        session
            .insert("login_error", "Invalid username".to_string())
            .map_err(|_| respond_not_authorised(&state))?;
        return Ok(Redirect::to("/login").into_response());
    }

    let permit = PASSWORD_HASH_SEMAPHORE
        .clone()
        .acquire_owned()
        .await
        .unwrap();
    let password_hash = Arc::clone(&state.user.password_hash);
    let hash_verify = spawn_blocking(move || {
        let result = password_hash.verify(payload.password.as_bytes());
        drop(permit);
        result
    })
    .await
    .map_err(|_| respond_not_authorised(&state))?;

    if !hash_verify {
        let ip = addr.to_string();
        warn!(
            "Login attempt from {ip} (username {}): Invalid password",
            &payload.username
        );
        // Save the error and redirect
        session
            .insert("login_error", "Invalid password".to_string())
            .map_err(|_| respond_not_authorised(&state))?;
        return Ok(Redirect::to("/login").into_response());
    }

    // Don't hold onto the session, login needs it, it will spin forever otherwise.
    drop(session);

    auth.login(&state.user).await.unwrap();
    Ok((token, Redirect::to("/admin").into_response()).into_response())
}

pub(crate) async fn logout_handler(mut auth: Auth) -> Redirect {
    auth.logout().await;
    Redirect::to("/")
}

pub(crate) async fn admin_handler(
    token: CsrfToken,
    mut session: WritableSession,
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
) -> Result<impl IntoResponse> {
    let results = Url::find()
        .all(&state.db)
        .await
        .map_err(|_| respond_internal_server_error(&state))?;

    let auth_token = token
        .authenticity_token()
        .map_err(|_| respond_not_authorised(&state))?;
    session
        .insert("auth_token", auth_token.clone())
        .map_err(|_| respond_not_authorised(&state))?;

    let t = AdminTemplate {
        base_host: &state.base_host,
        sitename: &state.sitename,
        urls: results,
        auth_token: &auth_token,
    };
    Ok((token, t).into_response())
}

pub(crate) async fn delete_handler(
    token: CsrfToken,
    mut session: WritableSession,
    State(state): State<AppState>,
    Extension(_user): Extension<User>,
    Form(payload): Form<DeletePayload>,
) -> Result<impl IntoResponse> {
    token
        .verify(&payload.auth_token)
        .map_err(|_| respond_not_authorised(&state))?;

    let auth_token = session
        .get::<String>("auth_token")
        .ok_or(respond_not_authorised(&state))?;

    session.remove("auth_token");

    token
        .verify(&auth_token)
        .map_err(|_| respond_not_authorised(&state))?;

    let _ = Url::delete_by_id(payload.id).exec(&state.db).await;
    Ok(Redirect::to("/admin").into_response())
}
