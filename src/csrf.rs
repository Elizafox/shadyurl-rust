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

use base64::{prelude::*, DecodeError};
use csrf::{ChaCha20Poly1305CsrfProtection, CsrfError, CsrfProtection};
use tower_sessions::Session;

#[derive(Debug, thiserror::Error)]
pub enum VerifyCsrfError {
    #[error(transparent)]
    Decode(#[from] DecodeError),

    #[error(transparent)]
    Csrf(#[from] CsrfError),

    #[error("Could not validate CSRF token: {}", .0)]
    CsrfValidation(String),
}

pub async fn verify(
    session: &Session,
    form_token: &str,
    protect: &ChaCha20Poly1305CsrfProtection,
) -> Result<(), VerifyCsrfError> {
    let authenticity_token: String = session
        .remove("authenticity_token")
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let token_bytes = BASE64_STANDARD.decode(form_token.as_bytes())?;
    let session_bytes = BASE64_STANDARD.decode(authenticity_token.as_bytes())?;

    let parsed_token = protect.parse_token(&token_bytes)?;
    let parsed_session = protect.parse_cookie(&session_bytes)?;

    if !protect.verify_token_pair(&parsed_token, &parsed_session) {
        return Err(VerifyCsrfError::CsrfValidation(
            "CSRF tokens could not be verified".to_string(),
        ));
    }

    Ok(())
}
