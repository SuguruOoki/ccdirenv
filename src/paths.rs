//! Path helpers for ~/.ccdirenv and subdirectories.

use anyhow::{Context, Result};
use std::path::PathBuf;

pub const MARKER_FILENAME: &str = ".ccdirenv";

pub fn root() -> Result<PathBuf> {
    if let Ok(explicit) = std::env::var("CCDIRENV_HOME") {
        return Ok(PathBuf::from(explicit));
    }
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".ccdirenv"))
}

pub fn bin_dir() -> Result<PathBuf> {
    Ok(root()?.join("bin"))
}

pub fn profiles_dir() -> Result<PathBuf> {
    Ok(root()?.join("profiles"))
}

pub fn profile_dir(name: &str) -> Result<PathBuf> {
    Ok(profiles_dir()?.join(name))
}

pub fn config_file() -> Result<PathBuf> {
    Ok(root()?.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn root_respects_ccdirenv_home_env() {
        std::env::set_var("CCDIRENV_HOME", "/custom/path");
        assert_eq!(root().unwrap(), PathBuf::from("/custom/path"));
        std::env::remove_var("CCDIRENV_HOME");
    }

    #[test]
    #[serial]
    fn bin_dir_is_under_root() {
        std::env::set_var("CCDIRENV_HOME", "/x");
        assert_eq!(bin_dir().unwrap(), PathBuf::from("/x/bin"));
        std::env::remove_var("CCDIRENV_HOME");
    }

    #[test]
    #[serial]
    fn profile_dir_joins_name() {
        std::env::set_var("CCDIRENV_HOME", "/x");
        assert_eq!(profile_dir("work").unwrap(), PathBuf::from("/x/profiles/work"));
        std::env::remove_var("CCDIRENV_HOME");
    }
}
