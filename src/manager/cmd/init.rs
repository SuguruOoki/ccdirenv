use crate::paths;
use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs as unix_fs;

pub fn run() -> Result<()> {
    let root = paths::root()?;
    let bin = paths::bin_dir()?;
    let default_profile = paths::profile_dir("default")?;

    fs::create_dir_all(&bin).context("creating bin dir")?;
    fs::create_dir_all(&default_profile).context("creating default profile dir")?;

    let cfg_path = paths::config_file()?;
    if !cfg_path.exists() {
        crate::config::Config::default().save(&cfg_path)?;
    }

    let shim_link = bin.join("claude");
    let self_path = std::env::current_exe().context("resolving current binary")?;
    if shim_link.exists() || shim_link.symlink_metadata().is_ok() {
        fs::remove_file(&shim_link).ok();
    }
    #[cfg(unix)]
    unix_fs::symlink(&self_path, &shim_link).context("creating shim symlink")?;

    println!("ccdirenv initialized at {}", root.display());
    println!();
    println!("Add to your shell rc:");
    println!("    export PATH=\"{}:$PATH\"", bin.display());
    Ok(())
}
