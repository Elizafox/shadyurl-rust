/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin/auth.rs
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
use tower_sessions::Session;
use tracing::{info, warn};

use crate::{
    auth::{AuthSession, Credentials},
    csrf as csrf_crate,
    error_response::AppError,
    state::AppState,
};

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    authenticity_token: String,
    messages: Vec<Message>,
    sitename: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
}

mod post {
    use super::{
        csrf_crate, info, warn, AppError, AppState, AuthSession, Credentials, Form, IntoResponse,
        Messages, Redirect, Response, Session, State,
    };

    pub(super) async fn login(
        session: Session,
        mut auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(creds): Form<Credentials>,
    ) -> Result<Response, AppError> {
        csrf_crate::verify(&session, &creds.authenticity_token, &state.protect).await?;

        let Some(user) = auth_session.authenticate(creds.clone()).await? else {
            warn!(
                "Invalid credentials received (username: {})",
                creds.username
            );
            messages.error("Invalid credentials");
            return Ok(Redirect::to("/login").into_response());
        };

        auth_session.login(&user).await?;

        info!("Successful login from {}", creds.username);
        messages.success(format!("Successfully logged in as {}", user.0.username));
        Ok(Redirect::to("/admin").into_response())
    }
}

mod get {
    use super::{
        info, AppError, AppState, AuthSession, CsrfProtection, IntoResponse, LoginTemplate,
        Messages, Redirect, Response, Session, State,
    };

    pub(super) async fn login(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        if auth_session.user.is_some() {
            return Ok(Redirect::to("/admin").into_response());
        }

        let (authenticity_token, session_token) = state.protect.generate_token_pair(None, 300)?;

        let authenticity_token = authenticity_token.b64_string();
        let session_token = session_token.b64_string();

        session.insert("authenticity_token", &session_token).await?;

        Ok(LoginTemplate {
            authenticity_token,
            messages: messages.into_iter().collect(),
            sitename: state.env.sitename.clone(),
        }
        .into_response())
    }

    pub(super) async fn logout(
        session: Session,
        mut auth_session: AuthSession,
        messages: Messages,
    ) -> Result<Response, AppError> {
        auth_session.logout().await?;
        session.clear().await;
        info!("User {} logging out", auth_session.user.unwrap().0.username);
        messages.success("You have logged out");
        Ok(Redirect::to("/").into_response())
    }
}
