//! `ccdirenv git` subcommands: enable/disable git remote-based detection
//! and configure which remote to inspect.

use crate::config::Config;
use crate::paths::config_file;
use anyhow::Result;

pub fn show() -> Result<()> {
    let cfg = Config::load(&config_file()?)?;
    println!("git enabled: {}", cfg.git.enabled);
    println!("git remote:  {}", cfg.git.remote);
    Ok(())
}

pub fn enable() -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.git.enabled = true;
    cfg.save(&path)?;
    println!("git detection enabled");
    Ok(())
}

pub fn disable() -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.git.enabled = false;
    cfg.save(&path)?;
    println!("git detection disabled");
    Ok(())
}

pub fn set_remote(name: &str) -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.git.remote = name.to_string();
    cfg.save(&path)?;
    println!("git remote set to {name}");
    Ok(())
}
