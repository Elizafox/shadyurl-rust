/* SPDX-License-Identifier: CC0-1.0
 *
 * src/csrf.rs
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

use aes_gcm_siv::{
    aead::{Aead, AeadCore, OsRng},
    Aes256GcmSiv,
};
use once_cell::sync::Lazy;
use rand::{
    distributions::{DistString, Uniform},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;
use tracing::debug;

use crate::util::string::WebsafeAlphabet;

pub type CryptoEngine = Aes256GcmSiv;

const SESSION_KEY: &str = "shadyurl.csrf";
const MAX_DURATION: Duration = Duration::minutes(10);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsrfSessionData {
    pub(super) token: String,
    pub(super) time: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsrfSessionEntry {
    encrypted: Vec<u8>,
    nonce: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum CsrfSessionError {
    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),

    #[error(transparent)]
    Crypto(#[from] aes_gcm_siv::Error),

    #[error(transparent)]
    Serialization(#[from] bincode::Error),

    #[error("No CSRF token")]
    NoToken,

    #[error("CSRF token mismatch")]
    Mismatch,

    #[error("CSRF token expired")]
    Expired,
}

impl CsrfSessionData {
    fn new() -> Self {
        let len_distr = Lazy::new(|| Uniform::new(24usize, 48usize));
        let mut rng = thread_rng();
        let len = (*len_distr).sample(&mut rng);
        Self {
            token: WebsafeAlphabet.sample_string(&mut rng, len),
            time: OffsetDateTime::now_utc(),
        }
    }

    pub fn cmp(&self, token: &str) -> Result<(), CsrfSessionError> {
        if token.as_bytes().ct_ne(self.token.as_bytes()).into() {
            debug!("CSRF tokens mismatched: {token} != {}", self.token);
            return Err(CsrfSessionError::Mismatch);
        }

        // This isn't sensitive info, so it's okay not to compare in constant time
        if OffsetDateTime::now_utc() - self.time > MAX_DURATION {
            debug!(
                "CSRF token expired: {}",
                OffsetDateTime::now_utc() - self.time
            );
            return Err(CsrfSessionError::Expired);
        }

        Ok(())
    }
}

impl CsrfSessionEntry {
    pub async fn insert_session(
        engine: &CryptoEngine,
        session: &Session,
    ) -> Result<String, CsrfSessionError> {
        let data = CsrfSessionData::new();

        let buf = bincode::serialize(&data)?;
        let nonce = Aes256GcmSiv::generate_nonce(&mut OsRng);
        let encrypted: Vec<u8> = engine.encrypt(&nonce, buf.as_ref())?;

        let entry = Self {
            encrypted,
            nonce: nonce.as_slice().into(),
        };

        session.insert(SESSION_KEY, entry).await?;

        Ok(data.token)
    }

    pub async fn check_session(
        engine: &CryptoEngine,
        session: &Session,
        token: &str,
    ) -> Result<(), CsrfSessionError> {
        // Smokey the Bear sez: Only YOU can prevent forest fi... err, session reuse
        let entry: Self = session
            .remove(SESSION_KEY)
            .await?
            .ok_or(CsrfSessionError::NoToken)?;

        let decrypted = engine.decrypt(entry.nonce.as_slice().into(), entry.encrypted.as_ref())?;
        let data: CsrfSessionData = bincode::deserialize(&decrypted)?;

        data.cmp(token)
    }
}
