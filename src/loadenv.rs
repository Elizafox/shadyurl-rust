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

use std::{
    env,
    ffi::OsString,
    marker::{Send, Sync},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Error, Result};
use argon2_kdf::Hash;
use dotenv::dotenv;
use lazy_static::lazy_static;
use os_str_bytes::RawOsString;
use tracing::Level;

lazy_static! {
    // If this fails, we want to know right away!
    static ref DEFAULT_PID_FILE: PathBuf = PathBuf::from_str("/var/run/shadyurl.pid")
        .expect("Unexpected failure unwrapping default PID file");
}

pub(crate) struct EnvVars {
    hostname: String,
    sitename: String,
    bind: String,
    database_url: String,
    redis_url: String,
    username: String,
    password_hash: Hash,
    secret: Vec<u8>,
    log_level: Level,
    log_stderr: bool,
    daemon: bool,
    pid_file: PathBuf,
    daemon_user: Option<String>,
    daemon_group: Option<String>,
}

impl EnvVars {
    fn get_env_var<T>(var: &str) -> Result<T>
    where
        T: FromStr + Clone,
        <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
    {
        env::var(var)
            .map_err(|e| {
                let ctx = format!("Could not get environment variable {var}");
                Error::new(e).context(ctx)
            })?
            .parse::<T>()
            .map_err(|e| {
                let ctx = format!("Could not parse environment variable {var}");
                Error::new(e).context(ctx)
            })
    }

    fn get_env_var_optional<T>(var: &str) -> Result<Option<T>>
    where
        T: FromStr + Clone,
        <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
    {
        match env::var(var) {
            Ok(value) => value.parse::<T>().map_or_else(
                |e| {
                    let ctx = format!("Could not parse environment variable {var}");
                    Err(Error::new(e).context(ctx))
                },
                |v| Ok(Some(v)),
            ),
            Err(e) => match e {
                env::VarError::NotPresent => Ok(None),
                _ => Err(Error::new(e)),
            },
        }
    }

    fn get_env_var_os(var: &str) -> Result<OsString> {
        env::var_os(var).ok_or_else(|| anyhow!("No {var} variable found in environment"))
    }

    pub(crate) fn new() -> Result<Self> {
        dotenv().map_err(|e| Error::new(e).context("Could not get environment variable"))?;

        let hostname: String = Self::get_env_var("HOSTNAME")?;
        let sitename = Self::get_env_var_optional("SITENAME")?.unwrap_or(hostname.clone());

        let bind = Self::get_env_var("BIND")?;

        let database_url = Self::get_env_var("DATABASE_URL")?;
        let redis_url = Self::get_env_var("REDIS_URL")?;

        let username = Self::get_env_var("USERNAME")?;
        let password_hash = Hash::from_str(&Self::get_env_var::<String>("PASSWORD_HASH")?)
            .map_err(|e| Error::new(e).context("Invalid PASSWORD_HASH variable"))?;

        let secret = Self::get_env_var_os("SECRET_KEY")?;
        let secret = Vec::from(RawOsString::new(secret).as_raw_bytes());

        let log_level = match Self::get_env_var_optional::<u8>("LOG_LEVEL")? {
            Some(level) => match level {
                0 => Level::ERROR,
                1 => Level::WARN,
                2 => Level::INFO,
                3 => Level::DEBUG,
                _ => Level::TRACE,
            },
            None => Level::WARN,
        };
        let log_stderr = Self::get_env_var_optional::<bool>("LOG_STDERR")?.unwrap_or(false);

        let daemon = Self::get_env_var_optional::<bool>("DAEMON")?.unwrap_or(false);
        let pid_file = Self::get_env_var_os("PID_FILE")
            .map_or(DEFAULT_PID_FILE.clone(), |v| PathBuf::from(&v));
        let (daemon_user, daemon_group) = if daemon {
            let daemon_user = Self::get_env_var_optional("DAEMON_USER")?;
            let daemon_group = Self::get_env_var_optional("DAEMON_GROUP")?;
            (daemon_user, daemon_group)
        } else {
            (None, None)
        };

        Ok(Self {
            hostname,
            sitename,
            bind,
            database_url,
            redis_url,
            username,
            password_hash,
            secret,
            log_level,
            log_stderr,
            daemon,
            pid_file,
            daemon_user,
            daemon_group,
        })
    }

    pub(crate) fn hostname(&self) -> &str {
        &self.hostname
    }

    pub(crate) fn sitename(&self) -> &str {
        &self.sitename
    }

    pub(crate) fn bind(&self) -> &str {
        &self.bind
    }

    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn redis_url(&self) -> &str {
        &self.redis_url
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password_hash(&self) -> &Hash {
        &self.password_hash
    }

    pub(crate) fn secret(&self) -> &[u8] {
        &self.secret
    }

    pub(crate) fn log_level(&self) -> Level {
        self.log_level
    }

    pub(crate) fn log_stderr(&self) -> bool {
        self.log_stderr
    }

    pub(crate) fn daemon(&self) -> bool {
        self.daemon
    }

    pub(crate) fn pid_file(&self) -> &PathBuf {
        &self.pid_file
    }

    pub(crate) fn daemon_user(&self) -> Option<&str> {
        self.daemon_user.as_deref()
    }

    pub(crate) fn daemon_group(&self) -> Option<&str> {
        self.daemon_group.as_deref()
    }
}
