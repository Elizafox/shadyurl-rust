/* SPDX-License-Identifier: CC0-1.0
 *
 * src/loadenv.rs
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

use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use argon2_kdf::Hash;
use axum_client_ip::SecureClientIpSource;
use dotenvy::dotenv;
use envy::from_env;
use serde::de::{Deserializer, Error};
use serde::Deserialize;
use tracing::Level;

fn default_sitename() -> String {
    "ShadyURL".to_string()
}

fn default_pid_file() -> PathBuf {
    PathBuf::from("/var/run/shadyurl.pid")
}

fn default_log_level() -> Level {
    Level::INFO
}

fn deserialize_hash<'de, D>(d: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    let hash = Hash::from_str(s.as_str())
        .map_err(|e| Error::custom(format!("Invalid hash value: {e}")))?;
    Ok(hash)
}

fn deserialize_level<'de, D>(d: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    let s = u8::deserialize(d)?;
    match s {
        0 => Ok(Level::ERROR),
        1 => Ok(Level::WARN),
        2 => Ok(Level::INFO),
        3 => Ok(Level::DEBUG),
        4 => Ok(Level::TRACE),
        _ => Err(Error::custom(format!("Invalid tracing level, must be 0-4"))),
    }
}

pub(crate) fn load_env() -> Result<EnvVars> {
    dotenv()?;
    let env: EnvVars = from_env()?;
    Ok(env)
}

#[derive(Deserialize)]
pub(crate) struct EnvVars {
    pub(crate) hostname: String,
    #[serde(default = "default_sitename")]
    pub(crate) sitename: String,
    pub(crate) bind: String,
    pub(crate) ip_source: SecureClientIpSource,
    pub(crate) database_url: String,
    pub(crate) redis_url: String,
    pub(crate) username: String,
    #[serde(deserialize_with = "deserialize_hash")]
    pub(crate) password_hash: Hash,
    #[serde(with = "serde_bytes")]
    pub(crate) secret_key: Vec<u8>,
    #[serde(deserialize_with = "deserialize_level", default = "default_log_level")]
    pub(crate) log_level: Level,
    #[serde(default)]
    pub(crate) log_stderr: bool,
    #[serde(default)]
    pub(crate) daemon: bool,
    #[serde(default = "default_pid_file")]
    pub(crate) pid_file: PathBuf,
    #[serde(default)]
    pub(crate) daemon_user: Option<String>,
    #[serde(default)]
    pub(crate) daemon_group: Option<String>,
}
