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

use once_cell::sync::Lazy;
use rand::{
    distributions::{DistString, Uniform},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use time::{Duration, OffsetDateTime};
use tokio::task::{spawn_blocking, JoinError};
use tower_sessions::Session;
use tracing::debug;

use crate::util::string::WebsafeAlphabet;

const SESSION_KEY: &str = "shadyurl.csrf";
const MAX_DURATION: Duration = Duration::minutes(10); // FIXME - configurable?

// Actual session data, a random token and the time the session began.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionData {
    pub(super) token: String,
    pub(super) time: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),

    #[error(transparent)]
    Join(#[from] JoinError),

    #[error("No CSRF token")]
    NoToken,

    #[error("CSRF token mismatch")]
    Mismatch,

    #[error("CSRF token expired")]
    Expired,
}

impl SessionData {
    async fn new() -> Self {
        let len_distr = Lazy::new(|| Uniform::new(24usize, 48usize));
        spawn_blocking(move || {
            let mut rng = thread_rng();
            let len = (*len_distr).sample(&mut rng);
            Self {
                token: WebsafeAlphabet.sample_string(&mut rng, len),
                time: OffsetDateTime::now_utc(),
            }
        })
        .await
        .expect("Unexpectedly failed to create session data")
    }

    async fn cmp(self, token: &str) -> Result<(), SessionError> {
        let token = token.to_owned();
        spawn_blocking(move || {
            if token.as_bytes().ct_ne(self.token.as_bytes()).into() {
                debug!("CSRF tokens mismatched");
                return Err(SessionError::Mismatch);
            }

            // This isn't sensitive info, so it's okay not to compare in constant time
            if OffsetDateTime::now_utc() - self.time > MAX_DURATION {
                debug!(
                    "CSRF token expired: {}",
                    OffsetDateTime::now_utc() - self.time
                );
                return Err(SessionError::Expired);
            }

            Ok(())
        })
        .await?
    }

    pub async fn new_into_session(session: &Session) -> Result<String, SessionError> {
        let data = Self::new().await;
        let token = data.token.clone();

        session.insert(SESSION_KEY, data).await?;

        Ok(token)
    }

    pub async fn check_session(session: &Session, token: &str) -> Result<(), SessionError> {
        // Smokey the Bear sez: Only YOU can prevent forest fi... err, session reuse
        let data: Self = session
            .remove(SESSION_KEY)
            .await?
            .ok_or(SessionError::NoToken)?;

        // Verify the token
        data.cmp(token).await
    }
}
