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

use std::{fs::File, net::SocketAddr};

use anyhow::Result;
use axum::Server;
use nix::unistd::ftruncate;
use proctitle::set_title;
use tokio::signal;
use tracing::error;

mod auth;
mod controllers;
mod daemon;
mod database;
mod err;
mod generate;
mod loadenv;
mod logging;
mod router;
mod state;
mod templates;
mod util;
mod validators;

use crate::{
    daemon::{close_stdio, drop_privileges, open_pid_file, set_umask, to_background},
    database::get_db,
    loadenv::EnvVars,
    logging::setup_logger,
    router::get_router,
    state::AppState,
};

// XXX passing in the PID file here is a hack
async fn shutdown_signal(pid_file: &mut File) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C/SIGINT handler");
    };

    let alarm = async {
        signal::unix::signal(signal::unix::SignalKind::alarm())
            .expect("failed to install SIGALRM handler")
            .recv()
            .await;
    };

    let hangup = async {
        signal::unix::signal(signal::unix::SignalKind::hangup())
            .expect("failed to install SIGHUP handler")
            .recv()
            .await;
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    // Maybe use this for stats later?
    let user_defined1 = async {
        signal::unix::signal(signal::unix::SignalKind::user_defined1())
            .expect("failed to install SIGUSR1 handler")
            .recv()
            .await;
    };

    let user_defined2 = async {
        signal::unix::signal(signal::unix::SignalKind::user_defined2())
            .expect("failed to install SIGUSR2 handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = alarm => {},
        _ = hangup => {},
        _ = terminate => {},
        _ = user_defined1 => {},
        _ = user_defined2 => {}
    }

    error!("signal received, starting graceful shutdown");

    // Clear PID file
    let _ = ftruncate(pid_file, 0);
}

// We must fork before we do anything else.
// We might as well do other environmental init stuff too.
fn main() -> Result<()> {
    let env = loadenv::load_env()?;

    if env.daemon {
        // Tokio can't survive a fork. This MUST be done first.
        to_background()?;
        close_stdio()?;
    }

    set_umask();

    let mut pid_file = open_pid_file(&env)?;

    setup_logger(&env);

    set_title("shadyurl-rust");

    tokio_main(&env, &mut pid_file)
}

#[tokio::main]
async fn tokio_main(env: &EnvVars, pid_file: &mut File) -> Result<()> {
    let db = get_db(env).await?;

    let state = AppState::new_from_env(db, env);

    let app = get_router(env, state).await?;
    let server = Server::try_bind(&env.bind.parse()?)?
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(pid_file));

    // We can only do this after the above
    drop_privileges(env)?;

    server.await?;

    // FIXME - do we actually get here?
    let _ = ftruncate(pid_file, 0);

    Ok(())
}
