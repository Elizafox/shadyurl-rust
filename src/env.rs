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
use dotenvy::dotenv;
use envy::from_env;
use rand::{prelude::*, thread_rng};
use serde::{
    de::{Deserializer, Error},
    Deserialize,
};
use validator::Validate;

mod defaults {
    use super::{thread_rng, Rng};

    pub(super) fn redis_url() -> String {
        "redis://127.0.0.1".to_string()
    }

    pub(super) const fn redis_pool_size() -> usize {
        2
    }

    pub(super) fn sitename() -> String {
        "ShadyURL".to_string()
    }

    pub(super) fn csrf_key() -> [u8; 32] {
        let mut ret = [0u8; 32];
        thread_rng().fill(&mut ret[..]);
        ret
    }
}

mod deserializers {
    use super::{Deserialize, Deserializer, Engine, Error, BASE64_STANDARD};

    pub(super) fn csrf_key<'de, D>(d: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let key_b64 = String::deserialize(d)?;
        let val = BASE64_STANDARD
            .decode(key_b64)
            .map_err(|e| Error::custom(format!("Could not decode csrf key: {e}")))?;

        let ret: [u8; 32] = val.try_into().map_err(|v: Vec<u8>| {
            Error::custom(format!(
                "Could not decode csrf key: length was incorrect (expected 32 bytes, got {})",
                v.len()
            ))
        })?;

        Ok(ret)
    }
}

#[derive(Clone, Validate, Deserialize)]
pub(crate) struct Vars {
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
        deserialize_with = "deserializers::csrf_key",
        default = "defaults::csrf_key"
    )]
    pub(crate) csrf_key: [u8; 32],
}

impl Vars {
    pub(crate) fn load_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv()?;
        let mut env: Self = from_env()?;
        if env.shady_host.is_empty() {
            env.shady_host = env.base_host.clone();
        }
        env.validate()?;
        Ok(env)
    }
}
