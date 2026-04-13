pub mod fast_path;
pub mod real;
pub mod replace;

use crate::{config::Config, env as cc_env, paths, profile::resolve};
use anyhow::Result;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn run(args: Vec<OsString>) -> Result<std::convert::Infallible> {
    let shim_dir = current_binary_dir()?;

    if cc_env::is_disabled() {
        let real = real::locate_real_claude(&shim_dir)?;
        return replace::replace_process(&real, &args, &[]);
    }

    if fast_path::is_fast_path(&args) {
        let real = real::locate_real_claude(&shim_dir)?;
        return replace::replace_process(&real, &args, &[]);
    }

    let real = real::locate_real_claude(&shim_dir)?;
    let cwd = std::env::current_dir()?;
    let config = Config::load(&paths::config_file()?).unwrap_or_default();
    let profile = resolve::resolve(&cwd, &config);
    let profile_path = paths::profile_dir(&profile)?;
    std::fs::create_dir_all(&profile_path).ok();

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
            "CLAUDE_CONFIG_DIR".into(),
            profile_path.to_string_lossy().into_owned(),
        )],
    )
}

fn current_binary_dir() -> Result<PathBuf> {
    let me = std::env::current_exe()?;
    let canonical = std::fs::canonicalize(&me).unwrap_or(me);
    Ok(canonical
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default())
}
