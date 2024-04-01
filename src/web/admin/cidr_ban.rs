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

// CIDR ban routes

use std::str::FromStr;

use askama_axum::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use ipnetwork::IpNetwork;
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;
use tracing::{debug, warn};
use validator::Validate;

use entity::{cidr_ban, user};
use service::{Mutation, Query};

use crate::{
    auth::AuthSession,
    csrf::SessionData,
    err::AppError,
    state::AppState,
    util::{
        net::{find_networks, vec_to_ipaddr, AddressError, NetworkPrefixError},
        string,
    },
    validators::validate_network,
};

// CIDR ban listing page (also submission)
#[derive(Template)]
#[template(path = "admin/cidr_ban.html")]
struct CidrBansTemplate<'a> {
    authenticity_token: &'a str,
    messages: Vec<Message>,
    sitename: &'a str,
    cidr_bans: Vec<(cidr_ban::Model, Option<user::Model>)>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
struct BanForm {
    authenticity_token: String,
    #[validate(custom(function = validate_network))]
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
        .route("/admin/cidr_bans/flush", get(self::get::flush))
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

    // Given an IP range from the database, render it fit for display
    // Used in templates.
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
        debug, find_networks, vec_to_ipaddr, warn, AppError, AppState, AuthSession, BanForm,
        DeleteForm, Form, FromStr, IntoResponse, IpNetwork, Messages, Mutation, Query, Redirect,
        Response, Session, SessionData, State, Validate,
    };

    pub(super) async fn cidr_bans(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(ban_form): Form<BanForm>,
    ) -> Result<Response, AppError> {
        SessionData::check_session(&session, &ban_form.authenticity_token).await?;

        let Some(user) = auth_session.user else {
            warn!("Unauthorized attempt to add a cidr_ban");
            return Err(AppError::Unauthorized);
        };

        if let Err(e) = ban_form.validate() {
            // Failed the validation checks
            let error_reason = e
                .field_errors()
                .get("range")
                .map_or("Unknown error".to_string(), |v| v[0].to_string());
            debug!(
                "Invalid range ({}) submitted from user {}: {error_reason}",
                ban_form.range, user.0.username
            );
            messages.error(format!("Invalid range: {error_reason}").as_str());
            return Ok(Redirect::to("/admin/cidr_bans").into_response());
        }

        // Validated previously
        let network =
            IpNetwork::from_str(&ban_form.range).map_or_else(|_| unreachable!(), |network| network);

        // Invalidate so users who aren't banned will now be
        state.bancache.invalidate(network);

        Mutation::create_cidr_ban(&state.db, network, ban_form.reason, &user.0).await?;

        warn!("CIDR ban ({}) added by {}", ban_form.range, user.0.username);
        messages.success(format!("Added CIDR ban {} successfullly", ban_form.range));
        Ok(Redirect::to("/admin/cidr_bans").into_response())
    }

    pub(super) async fn delete(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
        Form(delete_form): Form<DeleteForm>,
    ) -> Result<Response, AppError> {
        SessionData::check_session(&session, &delete_form.authenticity_token).await?;

        let Some(user) = auth_session.user else {
            warn!("Unauthorized attempt to delete a cidr_ban");
            return Err(AppError::Unauthorized);
        };

        let ban = Query::find_cidr_ban(&state.db, delete_form.id)
            .await?
            .ok_or_else(|| AppError::NotFound)?;
        let begin = vec_to_ipaddr(ban.range_begin)?;
        let end = vec_to_ipaddr(ban.range_end)?;

        // Delete from the database first
        Mutation::delete_cidr_ban(&state.db, delete_form.id).await?;

        // Invalidate the ban cache for all networks in this range
        for network in find_networks(begin, end)? {
            state.bancache.invalidate(network);
            warn!(
                "CIDR ban ({}) deleted by {}",
                network.to_string(),
                user.0.username
            );
        }

        messages.success(format!("Deleted CIDR ban #{} successfully", delete_form.id));
        Ok(Redirect::to("/admin/cidr_bans").into_response())
    }
}

mod get {
    use super::{
        debug, warn, AppError, AppState, AuthSession, CidrBansTemplate, IntoResponse, Messages,
        Query, Redirect, Response, Session, SessionData, State,
    };

    pub(super) async fn cidr_bans(
        session: Session,
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        let Some(user) = auth_session.user else {
            warn!("Unauthorized attempt to access cidr_bans");
            return Err(AppError::Unauthorized);
        };

        let authenticity_token = SessionData::new_into_session(&session).await?;

        let cidr_bans = Query::fetch_all_cidr_bans(&state.db).await?;

        debug!("CIDR bans retrieved by {}", user.0.username);

        Ok(CidrBansTemplate {
            authenticity_token: &authenticity_token,
            messages: messages.into_iter().collect(),
            sitename: &state.env.sitename,
            cidr_bans,
        }
        .into_response())
    }

    pub(super) async fn flush(
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<AppState>,
    ) -> Result<Response, AppError> {
        let Some(user) = auth_session.user else {
            warn!("Unauthorized attempt to flush ban cache");
            return Err(AppError::Unauthorized);
        };

        state.bancache.invalidate_all();
        messages.success("Flushed CIDR ban cache");
        debug!("User {} flushed ban cache", user.0.username);
        Ok(Redirect::to("/admin/cidr_bans").into_response())
    }
}
