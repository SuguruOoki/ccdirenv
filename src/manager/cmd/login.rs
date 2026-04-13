use crate::{paths, shim::real};
use anyhow::{bail, Result};
use std::process::Command;

pub fn run(profile: Option<String>) -> Result<()> {
    let name = profile.unwrap_or_else(|| "default".to_string());
    let profile_dir = paths::profile_dir(&name)?;
    std::fs::create_dir_all(&profile_dir)?;

    let shim_dir = paths::bin_dir()?;
    let real_bin = real::locate_real_claude(&shim_dir)?;

    println!("launching `claude /login` in profile '{name}'...");
    let status = Command::new(&real_bin)
        .arg("/login")
        .env("CLAUDE_CONFIG_DIR", &profile_dir)
        .status()?;
    if !status.success() {
        bail!("`claude /login` exited with {:?}", status.code());
    }
    Ok(())
}
