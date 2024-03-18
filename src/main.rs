/* SPDX-License-Identifier: CC0-1.0
 *
 * src/main.rs
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

#![warn(unused_extern_crates)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use dotenvy::dotenv;
use mimalloc::MiMalloc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::run_command;

mod auth;
mod bancache;
mod cli;
mod csrf;
mod env;
mod err;
mod generate;
mod state;
mod urlcache;
mod util;
mod validators;
mod web;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Important to do this first, as our logging config may be in .env
    dotenv()?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    run_command().await
}
