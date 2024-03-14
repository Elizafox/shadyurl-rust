/* SPDX-License-Identifier: CC0-1.0
 *
 * src/web/admin/cidr_ban.rs
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

use std::{convert::TryInto, net::IpAddr, str::FromStr};

use askama_axum::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use csrf::CsrfProtection;
use ipnetwork::{IpNetwork, IpNetworkError};
use itertools::Itertools;
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;

use entity::{cidr_ban, user};
use service::{Mutation, Query};

use crate::{
    auth::AuthSession,
    csrf as csrf_crate,
    error_response::AppError,
    state::AppState,
    util::{
        format,
        net::{find_networks, NetworkPrefixError},
    },
};

#[derive(Template)]
#[template(path = "admin/cidr_ban.html")]
struct CidrBansTemplate<'a> {
    authenticity_token: &'a str,
    messages: Vec<Message>,
    sitename: &'a str,
    cidr_bans: Vec<(cidr_ban::Model, Option<user::Model>)>,
}

#[derive(Debug, Clone, Deserialize)]
struct SubmitBanForm {
    authenticity_token: String,
    range: String,
    reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeleteForm {
    authenticity_token: String,
    id: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/cidr_bans", get(self::get::cidr_bans))
        .route("/admin/cidr_bans", post(self::post::cidr_bans))
        .route("/admin/cidr_bans/delete", post(self::post::delete))
}

mod render {
    use super::{find_networks, IpAddr, IpNetworkError, Itertools, NetworkPrefixError, TryInto};

    pub(super) fn range_to_display(
        start: Vec<u8>,
        end: Vec<u8>,
    ) -> Result<Vec<String>, NetworkPrefixError> {
        let Some((start, end)) = match (start.len(), end.len()) {
            (4, 4) => [
                // These should never fail
                IpAddr::from(
                    TryInto::<[u8; 4]>::try_into(start).expect("Failed to convert start IP"),
                ),
                IpAddr::from(TryInto::<[u8; 4]>::try_into(end).expect("Failed to convert end IP")),
            ],
            (16, 16) => [
                // These should never fail
                IpAddr::from(
                    TryInto::<[u8; 16]>::try_into(start).expect("Failed to convert start IP"),
                ),
                IpAddr::from(TryInto::<[u8; 16]>::try_into(end).expect("Failed to convert end IP")),
            ],
            _ => {
                return Err(NetworkPrefixError::IpNetwork(IpNetworkError::InvalidAddr(
                    "Invalid range".to_string(),
                )))
            }
        }
        .into_iter()
        .map(|v| v.to_canonical())
        .collect_tuple() else {
            unreachable!();
        };

        let nets = find_networks(start.clone(), end.clone())?;
        Ok(nets.into_iter().map(|v| format!("{v}")).collect())
    }
}

mod post {
    use super::{
        csrf_crate, AppError, AppState, AuthSession, DeleteForm, Form, FromStr, IntoResponse,
        IpAddr, IpNetwork, Messages, Mutation, Redirect, Response, Session, State, SubmitBanForm,
    };

    pub(super) async fn cidr_bans(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(submit_ban_form): Form<SubmitBanForm>,
    ) -> Result<Response, AppError> {
        csrf_crate::verify(
            &session,
            &submit_ban_form.authenticity_token,
            &state.protect,
        )
        .await?;

        let Some(user) = auth_session.user else {
            return Err(AppError::Unauthorized);
        };

        if submit_ban_form.range.is_empty() {
            messages.error("Range cannot be empty");
            return Ok(Redirect::to("/admin/cidr_bans").into_response());
        }

        let range = submit_ban_form.range.clone();
        let network = match range.rsplit_once("/") {
            Some((address, prefix)) => {
                let Ok(addr) = IpAddr::from_str(address) else {
                    messages.error("Invalid IP range");
                    return Ok(Redirect::to("/admin/cidr_bans").into_response());
                };

                let Ok(prefix) = prefix.parse::<u8>() else {
                    messages.error("Invalid network prefix");
                    return Ok(Redirect::to("/admin/cidr_bans").into_response());
                };

                match IpNetwork::new(addr, prefix) {
                    Ok(i) => i,
                    Err(_) => {
                        messages.error("Invalid IP range");
                        return Ok(Redirect::to("/admin/cidr_bans").into_response());
                    }
                }
            }
            None => {
                let Ok(addr) = IpAddr::from_str(&range) else {
                    messages.error("Invalid IP range");
                    return Ok(Redirect::to("/admin/cidr_bans").into_response());
                };

                let prefix = match addr {
                    IpAddr::V4(_) => 32,
                    IpAddr::V6(_) => 128,
                };

                match IpNetwork::new(addr, prefix) {
                    Ok(i) => i,
                    Err(_) => {
                        messages.error("Invalid IP range");
                        return Ok(Redirect::to("/admin/cidr_bans").into_response());
                    }
                }
            }
        };

        Mutation::create_cidr_ban(&state.db, network, submit_ban_form.reason, &user.0).await?;

        messages.success(format!(
            "Added CIDR ban {} successfullly",
            submit_ban_form.range
        ));
        Ok(Redirect::to("/admin/cidr_bans").into_response())
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

        Mutation::delete_cidr_ban(&state.db, delete_form.id).await?;

        messages.success(format!("Deleted CIDR ban #{} successfully", delete_form.id));
        Ok(Redirect::to("/admin/cidr_bans").into_response())
    }
}

mod get {
    use super::{
        AppError, AppState, AuthSession, CidrBansTemplate, CsrfProtection, IntoResponse, Messages,
        Query, Response, Session, State,
    };

    pub(super) async fn cidr_bans(
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

        let cidr_bans = Query::fetch_all_cidr_bans(&state.db).await?;

        Ok(CidrBansTemplate {
            authenticity_token: &authenticity_token,
            messages: messages.into_iter().collect(),
            sitename: &state.env.sitename,
            cidr_bans,
        }
        .into_response())
    }
}
