pub mod fast_path;
pub mod real;
pub mod replace;

use crate::{config::Config, env as cc_env, paths, profile::resolve, tool::Tool};
use anyhow::Result;
use std::ffi::OsString;

pub fn run(tool: Tool, args: Vec<OsString>) -> Result<std::convert::Infallible> {
    let shim_dir = paths::bin_dir()?;

    if cc_env::is_disabled() {
        let real = real::locate_real(tool, &shim_dir)?;
        return replace::replace_process(&real, &args, &[]);
    }

    if fast_path::is_fast_path_for(tool, &args) {
        let real = real::locate_real(tool, &shim_dir)?;
        return replace::replace_process(&real, &args, &[]);
    }

    let real = real::locate_real(tool, &shim_dir)?;
    let cwd = std::env::current_dir()?;
    let config = Config::load(&paths::config_file()?)?;
    let profile = resolve::resolve(&cwd, &config);
    let profile_path = tool.ensure_config_dir(&profile)?;

    if cc_env::is_debug() {
        eprintln!(
            "ccdirenv: profile={} dir={} real={}",
            profile,
            profile_path.display(),
            real.display()
        );
    }

    replace::replace_process(
        &real,
        &args,
        &[(
            tool.config_env().into(),
            profile_path.to_string_lossy().into_owned(),
        )],
    )
}
