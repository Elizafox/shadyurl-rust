/* SPDX-License-Identifier: CC0-1.0
 *
 * src/daemon.rs
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

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::process::exit;

use anyhow::{anyhow, bail, Error, Result};
use nix::fcntl::{flock, FlockArg};
use nix::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use nix::sys::stat::{umask, Mode};
use nix::unistd::*;

use crate::loadenv::EnvVars;

// This bad hack is because nix erroneously doesn't define initgroups for macOS
#[cfg(target_os = "macos")]
#[inline]
fn do_initgroups(name: &CStr, gid: Gid) -> Result<()> {
    use nix::libc;
    use std::io;
    // SAFETY: name is not mutated and should be a safe value, gid is a safe cast
    if unsafe { libc::initgroups(name.as_ptr() as _, gid.as_raw() as _) } != 0 {
        return Err(Error::new(io::Error::last_os_error()).context("initgroups() failed"));
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[inline]
fn do_initgroups(name: CStr, gid: Gid) -> Result<()> {
    initgroups(name, gid).map_err(|e| Error::new(e).context("initgroups() failed"))?;
    Ok(())
}

#[inline]
pub(crate) fn close_stdio() -> Result<()> {
    close(STDIN_FILENO)?;

    // As a result of the above this should now be stdin
    let dev_null_file = Box::leak(Box::new(
        File::options()
            .write(true)
            .open("/dev/null")
            .map_err(|e| Error::new(e).context("Failed to open /dev/null"))?,
    ));

    // Close the major file descriptors
    if dev_null_file.as_raw_fd() != STDIN_FILENO {
        bail!("Failed to reopen stdin");
    }

    if dup2(STDIN_FILENO, STDOUT_FILENO)? != STDOUT_FILENO {
        bail!("Failed to close stdout");
    }

    if dup2(STDIN_FILENO, STDERR_FILENO)? != STDERR_FILENO {
        bail!("Failed to close stderr");
    }

    Ok(())
}

#[inline]
pub(crate) fn open_pid_file(env: &EnvVars) -> Result<File> {
    // Open the PID file with current privileges before dropping
    // We leak because we want this to live forever
    let mut pid_file = File::options().create(true).read(true).write(true).open(
        env.pid_file()
            .expect("Invariant not upheld: PID file not set"),
    )?;

    // Lock exclusively
    flock(pid_file.as_raw_fd(), FlockArg::LockExclusiveNonblock)
        .map_err(|e| Error::new(e).context("PID file is locked (process already running?)"))?;

    // Erase and write new PID
    ftruncate(pid_file.as_raw_fd(), 0)?;
    pid_file
        .write_all(getpid().to_string().as_bytes())
        .map_err(|e| Error::new(e).context("Failed to write to PID file"))?;

    Ok(pid_file)
}

pub(crate) fn drop_privileges(env: &EnvVars) -> Result<()> {
    if let Some(daemon_user) = env.daemon_user() {
        let user = User::from_name(daemon_user)?.ok_or(anyhow!("No such user {daemon_user}"))?;

        let group = match env.daemon_group() {
            Some(daemon_group) => {
                Group::from_name(daemon_group)?.ok_or(anyhow!("No such group {daemon_group}"))?
            }
            None => Group::from_gid(user.gid)?
                .ok_or(anyhow!("User {daemon_user} GID is nonexistent"))?,
        };

        // Drop privileges
        let username_c = CString::new(user.name)?;
        do_initgroups(&username_c.as_c_str(), group.gid)?;
        setgid(group.gid).map_err(|e| Error::new(e).context("setgid() failed"))?;
        setuid(user.uid).map_err(|e| Error::new(e).context("setuid() failed"))?;
    } else if let Some(daemon_group) = env.daemon_group() {
        let group =
            Group::from_name(daemon_group)?.ok_or(anyhow!("No such group {daemon_group}"))?;
        setgid(group.gid).map_err(|e| Error::new(e).context("setgid() failed"))?;
    }

    Ok(())
}

#[inline]
pub(crate) fn to_background() -> Result<()> {
    // Perform double fork to ensure we can't get a TTY
    // SAFETY: it's fork, what do you want?
    match unsafe { fork().map_err(|e| Error::new(e).context("fork() failed"))? } {
        ForkResult::Parent { .. } => exit(0),
        ForkResult::Child => {}
    }

    setsid()?;

    // Second fork, parent is session leader but dead, so we can never regain a TTY
    // SAFETY: it's fork, what do you want?
    match unsafe { fork().map_err(|e| Error::new(e).context("fork() failed"))? } {
        ForkResult::Parent { .. } => exit(0),
        ForkResult::Child => {}
    }

    // On many OSes, it's *unsafe* to proceed in the child, so we HAVE to exec.
    let mut args: Vec<_> = std::env::args_os()
        .map(|v| CString::new(v.as_bytes()).expect("Unexpected failure getting arguments"))
        .collect();
    let arg0 = args[0].clone();
    args[0] = CString::new("shadyurl-rust").expect("Unexpected failure setting args[0]");
    execve(
        arg0.as_c_str(),
        &args,
        &[CString::new("__SHADYURL_POST_EXEC=1")?],
    )
    .map_err(|e| Error::new(e).context("execve() failed (uh oh)"))?;
    unreachable!();
}

#[inline]
pub(crate) fn set_umask() {
    umask(Mode::from_bits(0o137).expect("Unexpected failure in getting mode bits"));
}
