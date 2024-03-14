/* SPDX-License-Identifier: CC0-1.0
 *
 * src/auth.rs
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

use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::verify_password;
use sea_orm::{DbConn, DbErr};
use serde::Deserialize;
use tokio::task;

use entity::user;
use service::Query;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions)]
pub enum AuthError {
    #[error(transparent)]
    DbErr(#[from] DbErr),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

// We marshall the database user in and out of this
#[derive(Clone)]
pub struct User(pub(crate) user::Model);

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.0.id)
            .field("username", &self.0.username)
            .field("password_hash", &"[redacted]")
            .finish()
    }
}

// XXX - hacky but we can't implement AuthUser for a foreign type
// So we just fake it.
impl User {
    pub(super) async fn find_user_by_username(
        db: &DbConn,
        username: &str,
    ) -> Result<Option<Self>, AuthError> {
        let user = Query::find_user_by_username(db, username).await?;

        Ok(user.map(User))
    }

    pub(super) async fn find_user_by_id(db: &DbConn, id: i64) -> Result<Option<Self>, AuthError> {
        let user = Query::find_user_by_id(db, id).await?;

        Ok(user.map(User))
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.0.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.0.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: Arc<DbConn>,
}

impl Backend {
    pub(crate) fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) authenticity_token: String,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = Self::User::find_user_by_username(&self.db, &creds.username).await?;

        task::spawn_blocking(|| {
            Ok(user.filter(|user| verify_password(creds.password, &user.0.password_hash).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = Self::User::find_user_by_id(&self.db, *id).await?;
        Ok(user)
    }
}

#[allow(clippy::module_name_repetitions)]
pub type AuthSession = axum_login::AuthSession<Backend>;
