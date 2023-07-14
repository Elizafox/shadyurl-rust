/* SPDX-License-Identifier: CC0-1.0
 *
 * src/logging.rs
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

use std::ffi::CStr;

use syslog_tracing::{Facility, Options, Syslog};

use crate::loadenv::EnvVars;

pub(crate) fn setup_logger(env: &EnvVars) {
    let identity: &CStr = CStr::from_bytes_with_nul(b"shadyurl-rust\0").unwrap();

    let mut options: Options = Options::LOG_NDELAY | Options::LOG_PID | Options::LOG_CONS;
    if env.log_stderr {
        options = options | Options::LOG_PERROR;
    }

    let facility: Facility = Facility::Daemon;
    let syslog: Syslog = Syslog::new(identity, options, facility).unwrap();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(env.log_level)
        .with_writer(syslog)
        .init();
}
