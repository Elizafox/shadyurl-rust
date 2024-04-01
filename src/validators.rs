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

use ipnetwork::IpNetwork;

use url::{Host, Url};
use validator::ValidationError;

// Ensure a URL is a valid type
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    let err = ValidationError::new("Invalid URL");
    if url.len() > 2048 {
        return Err(err.with_message("URL is too long".into()));
    }

    let url_parsed = Url::parse(url).map_err(|_| err.clone().with_message("Invalid URL".into()))?;
    if !url_parsed.has_host() {
        Err(err.clone().with_message("No host found".into()))?;
    }

    match url_parsed.scheme() {
        "ftp" | "ftps" | "gemini" | "gopher" | "http" | "https" | "irc" | "irc6" | "ircs"
        | "jabber" | "matrix" | "mumble" | "mxc" | "spotify" | "teamspeak" | "xmpp" => {
            match url_parsed
                .host()
                .ok_or_else(|| err.clone().with_message("No host found".into()))?
            {
                Host::Ipv4(_) | Host::Ipv6(_) => Ok(()),
                Host::Domain(host_str) => {
                    let pos = host_str
                        .rfind('.')
                        .ok_or_else(|| err.clone().with_message("No TLD".into()))?;
                    if host_str.len() - pos < 2 {
                        return Err(err.clone().with_message("Invalid TLD".into()));
                    }

                    Ok(())
                }
            }
        }
        _ => Err(err.clone().with_message("Bad URL scheme".into())),
    }
}

// Ensure a CIDR is correct
pub fn validate_network(network: &str) -> Result<(), ValidationError> {
    let _: IpNetwork = network.parse().map_err(|e| {
        ValidationError::new("Invalid network")
            .with_message(format!("Error with network: {e}").into())
    })?;
    Ok(())
}
