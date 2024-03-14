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

use std::str::FromStr;

use askama_axum::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use csrf::CsrfProtection;
use ipnetwork::IpNetwork;
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
        net::{find_networks, vec_to_ipaddr, AddressError, NetworkPrefixError},
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

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/cidr_bans", get(self::get::cidr_bans))
        .route("/admin/cidr_bans", post(self::post::cidr_bans))
        .route("/admin/cidr_bans/delete", post(self::post::delete))
}

mod render {
    use super::{find_networks, vec_to_ipaddr, AddressError, NetworkPrefixError};

    #[derive(Debug, thiserror::Error)]
    pub(super) enum RangeDisplayError {
        #[error(transparent)]
        NetworkPrefix(#[from] NetworkPrefixError),

        #[error(transparent)]
        Addr(#[from] AddressError),
    }

    pub(super) fn range_to_display(
        begin: Vec<u8>,
        end: Vec<u8>,
    ) -> Result<Vec<String>, RangeDisplayError> {
        if begin.len() != end.len() {
            return Err(RangeDisplayError::NetworkPrefix(
                NetworkPrefixError::IpTypeMismatch,
            ));
        }

        let begin = vec_to_ipaddr(begin)?;
        let end = vec_to_ipaddr(end)?;

        let nets = find_networks(begin, end)?;
        Ok(nets.into_iter().map(|v| format!("{v}")).collect())
    }
}

mod post {
    use super::{
        csrf_crate, find_networks, vec_to_ipaddr, AppError, AppState, AuthSession, DeleteForm,
        Form, FromStr, IntoResponse, IpNetwork, Messages, Mutation, Query, Redirect, Response,
        Session, State, SubmitBanForm,
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

        let network = match IpNetwork::from_str(&submit_ban_form.range) {
            Ok(network) => network,
            Err(e) => {
                messages.error(format!("Invalid IP range specified: {e}"));
                return Ok(Redirect::to("/admin/cidr_bans").into_response());
            }
        };

        // Invalidate so users who aren't banned will now be
        state.bancache.invalidate(network.clone());

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

        let ban = Query::find_cidr_ban(&state.db, delete_form.id)
            .await?
            .ok_or_else(|| AppError::NotFound)?;
        let begin = vec_to_ipaddr(ban.range_begin)?;
        let end = vec_to_ipaddr(ban.range_end)?;

        // Invalidate the ban cache for all networks in this range
        for network in find_networks(begin, end)? {
            state.bancache.invalidate(network);
        }

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
