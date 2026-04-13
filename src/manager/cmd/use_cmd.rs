use crate::paths::{self, MARKER_FILENAME};
use anyhow::{bail, Result};
use std::fs;

pub fn run(profile: &str) -> Result<()> {
    let profile_dir = paths::profile_dir(profile)?;
    if !profile_dir.is_dir() {
        bail!("profile '{profile}' does not exist. Create with `ccdirenv login {profile}`.");
    }
    let cwd = std::env::current_dir()?;
    let marker = cwd.join(MARKER_FILENAME);
    fs::write(&marker, format!("{profile}\n"))?;
    println!("bound {} -> {profile}", cwd.display());
    Ok(())
}
