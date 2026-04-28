//! `ccdirenv owners` subcommands: manage the shared owner→profile map.

use crate::config::Config;
use crate::paths::config_file;
use anyhow::{anyhow, Result};

pub fn list() -> Result<()> {
    let cfg = Config::load(&config_file()?)?;
    if cfg.owners.is_empty() {
        println!("(no owner mappings configured)");
        return Ok(());
    }
    for (key, profile) in &cfg.owners {
        println!("{key}\t{profile}");
    }
    Ok(())
}

pub fn map(owner: &str, profile: &str) -> Result<()> {
    if !owner.contains('/') {
        return Err(anyhow!(
            "owner must be in 'host/owner' form (e.g. 'github.com/Acme'), got '{owner}'"
        ));
    }
    if profile.is_empty() {
        return Err(anyhow!("profile name cannot be empty"));
    }
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.owners.insert(owner.to_string(), profile.to_string());
    cfg.save(&path)?;
    println!("mapped {owner} -> {profile}");
    Ok(())
}

pub fn unmap(owner: &str) -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    if cfg.owners.shift_remove(owner).is_some() {
        cfg.save(&path)?;
        println!("removed {owner}");
    } else {
        println!("no mapping for {owner}");
    }
    Ok(())
}
