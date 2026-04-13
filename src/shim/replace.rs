//! Process-replacement helper (unix only).

use anyhow::Result;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

pub fn replace_process(
    real: &Path,
    args: &[OsString],
    extra_env: &[(String, String)],
) -> Result<std::convert::Infallible> {
    let mut cmd = Command::new(real);
    if args.len() > 1 {
        cmd.args(&args[1..]);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    // CommandExt replaces the current process on success.
    let replace_fn = CommandExt::exec;
    let err = replace_fn(&mut cmd);
    Err(err.into())
}
