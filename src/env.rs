/* SPDX-License-Identifier: CC0-1.0
 *
 * src/env.rs
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

use axum_client_ip::SecureClientIpSource;
use base64::prelude::*;
use envy::from_env;
use rand::{prelude::*, thread_rng};
use serde::{
    de::{Deserializer, Error},
    Deserialize,
};
use systemd_duration::time::parse;
use time::Duration;
use tracing::error;
use validator::Validate;

// Routines to get configuration information from the environment and .env file

pub type Key = [u8; 64];

mod defaults {
    use super::{thread_rng, Duration, Key, Rng};

    pub(super) fn redis_url() -> String {
        "redis://127.0.0.1".to_string()
    }

    pub(super) const fn redis_pool_size() -> usize {
        2
    }

    pub(super) const fn max_cache_entries() -> u64 {
        5000
    }

    pub(super) fn sitename() -> String {
        "ShadyURL".to_string()
    }

    pub(super) fn csrf_key() -> Key {
        let mut ret = [0u8; 64];
        thread_rng().fill(&mut ret);
        ret
    }

    pub(super) const fn duration_1d() -> Duration {
        Duration::days(1)
    }

    pub(super) const fn duration_3d() -> Duration {
        Duration::days(3)
    }
}

mod deserializers {
    use super::{
        error, parse, Deserialize, Deserializer, Duration, Engine, Error, Key, BASE64_STANDARD,
    };

    pub(super) fn duration<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration_string = String::deserialize(d)?;
        let ret: Duration = parse(&duration_string).map_err(|e| {
            error!("Invalid duration: \"{duration_string}\", error: {e}");
            Error::custom(format!(
                "Invalid duration received: \"{duration_string}\", error: {e}"
            ))
        })?;

        Ok(ret)
    }

    pub(super) fn csrf_key<'de, D>(d: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key_b64 = String::deserialize(d)?;
        let val = BASE64_STANDARD
            .decode(key_b64)
            .map_err(|e| Error::custom(format!("Could not decode session key: {e}")))?;

        let ret: Key = val.try_into().map_err(|v: Vec<u8>| {
            error!(
                "Session key length was incorrect (expected 64 bytes, got {}",
                v.len()
            );
            Error::custom(format!(
                "Could not decode session key: length was incorrect (expected 64 bytes, got {})",
                v.len()
            ))
        })?;

        Ok(ret)
    }
}

#[derive(Clone, Validate, Deserialize)]
pub struct Vars {
    #[validate(length(min = 4))]
    pub(crate) base_host: String,
    #[serde(default)]
    pub(crate) shady_host: String,
    #[serde(default = "defaults::sitename")]
    pub(crate) sitename: String,

    #[validate(length(min = 1))]
    pub(crate) bind: String,

    pub(crate) ip_source: SecureClientIpSource,

    #[validate(url)]
    pub(crate) database_url: String,
    #[serde(default = "defaults::redis_url")]
    #[validate(url)]
    pub(crate) redis_url: String,
    #[serde(default = "defaults::redis_pool_size")]
    #[validate(range(min = 1))]
    pub(crate) redis_pool_size: usize,

    #[serde(
        deserialize_with = "deserializers::duration",
        default = "defaults::duration_3d"
    )]
    pub(crate) ban_cache_ttl: Duration,
    #[serde(
        deserialize_with = "deserializers::duration",
        default = "defaults::duration_1d"
    )]
    pub(crate) ban_cache_idle: Duration,
    #[serde(
        deserialize_with = "deserializers::duration",
        default = "defaults::duration_3d"
    )]
    pub(crate) url_cache_ttl: Duration,
    #[serde(
        deserialize_with = "deserializers::duration",
        default = "defaults::duration_1d"
    )]
    pub(crate) url_cache_idle: Duration,

    #[serde(default = "defaults::max_cache_entries")]
    pub(crate) ban_cache_max_entries: u64,
    #[serde(default = "defaults::max_cache_entries")]
    pub(crate) url_cache_max_entries: u64,

    // FIXME: encrypt entire session with this, but axum-login isn't ready
    #[serde(
        deserialize_with = "deserializers::csrf_key",
        default = "defaults::csrf_key"
    )]
    pub(crate) csrf_key: Key,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, thiserror::Error)]
pub enum EnvError {
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),

    #[error(transparent)]
    Envy(#[from] envy::Error),
}

impl Vars {
    pub(crate) fn load_env() -> Result<Self, EnvError> {
        let mut env: Self = from_env()?;
        if env.shady_host.is_empty() {
            env.shady_host = env.base_host.clone();
        }
        env.validate()?;
        Ok(env)
    }
}
