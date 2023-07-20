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

use argon2_kdf::Hash;
use axum_login::{
    extractors::AuthContext, memory_store::MemoryStore as AuthMemoryStore, secrecy::SecretVec,
    AuthUser, RequireAuthorizationLayer,
};

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: usize,
    pub(crate) username: String,
    pub(crate) password_hash: Arc<Hash>,
}

impl AuthUser<usize> for User {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.as_bytes().into())
    }
}

pub(crate) type Auth = AuthContext<usize, User, AuthMemoryStore<usize, User>>;
pub(crate) type RequireAuth = RequireAuthorizationLayer<usize, User>;
