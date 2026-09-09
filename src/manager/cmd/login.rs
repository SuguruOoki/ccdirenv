use crate::{paths, shim::real, tool::Tool};
use anyhow::{bail, Result};
use std::ffi::OsString;
use std::process::Command;

pub fn run(profile: Option<String>, tool: Tool, args: Vec<OsString>) -> Result<()> {
    let name = profile.unwrap_or_else(|| "default".to_string());
    let profile_dir = tool.ensure_config_dir(&name)?;

    let shim_dir = paths::bin_dir()?;
    let real_bin = real::locate_real(tool, &shim_dir)?;

    let binary = tool.binary();
    let login = tool.login_arg();
    println!("launching `{binary} {login}` in profile '{name}'...");
    let status = Command::new(&real_bin)
        .arg(login)
        .args(args)
        .env(tool.config_env(), &profile_dir)
        .status()?;
    if !status.success() {
        bail!("`{binary} {login}` exited with {:?}", status.code());
    }
    Ok(())
}
