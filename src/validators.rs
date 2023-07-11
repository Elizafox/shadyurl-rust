/* SPDX-License-Identifier: CC0-1.0
 *
 * src/validators.rs
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

use url::Url;
use validator::ValidationError;

pub(crate) fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.len() > 2047 {
        return Err(ValidationError::new("URL is too long"));
    }

    let url_parsed = Url::parse(url).map_err(|_| ValidationError::new("Invalid URL"))?;
    if !url_parsed.has_host() {
        return Err(ValidationError::new("No host found"))?;
    }

    match url_parsed.scheme() {
        "http" | "https" | "ftp" | "ftps" | "gopher" | "irc" | "ircs" | "gemini" => {
            let host_str = url_parsed.host_str().unwrap();
            match host_str.rfind('.') {
                None => {
                    return Err(ValidationError::new("Invalid hostname"));
                }
                Some(pos) => {
                    if host_str.len() - pos < 3 {
                        return Err(ValidationError::new("Invalid TLD"));
                    }

                    return Ok(());
                }
            }
        }
        _ => {
            return Err(ValidationError::new("Bad scheme"));
        }
    }
}
