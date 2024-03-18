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

use url::{Host, Url};
use validator::ValidationError;

// Ensure a URL is a valid type
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.len() > 2048 {
        return Err(ValidationError::new("URL is too long"));
    }

    let url_parsed = Url::parse(url).map_err(|_| ValidationError::new("Invalid URL"))?;
    if !url_parsed.has_host() {
        Err(ValidationError::new("No host found"))?;
    }

    match url_parsed.scheme() {
        "ftp" | "ftps" | "gemini" | "gopher" | "http" | "https" | "irc" | "irc6" | "ircs"
        | "jabber" | "matrix" | "mumble" | "mxc" | "spotify" | "teamspeak" | "xmpp" => {
            match url_parsed
                .host()
                .ok_or_else(|| ValidationError::new("No host found"))?
            {
                Host::Ipv4(_) | Host::Ipv6(_) => Ok(()),
                Host::Domain(host_str) => {
                    let pos = host_str
                        .rfind('.')
                        .ok_or_else(|| ValidationError::new("No TLD"))?;
                    if host_str.len() - pos < 2 {
                        return Err(ValidationError::new("Invalid TLD"));
                    }

                    Ok(())
                }
            }
        }
        _ => Err(ValidationError::new("Bad scheme")),
    }
}
