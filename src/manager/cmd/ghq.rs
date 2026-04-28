//! `ccdirenv ghq` subcommands: manage owner→profile mappings.

use crate::config::Config;
use crate::manager::cmd::ensure_ghq::{ensure, EnsureMode};
use crate::paths::config_file;
use anyhow::{anyhow, Result};

pub fn list() -> Result<()> {
    let cfg = Config::load(&config_file()?)?;
    if let Some(root) = cfg.ghq.root.as_ref() {
        println!("ghq root: {root}");
    } else if let Ok(env_root) = std::env::var("GHQ_ROOT") {
        if !env_root.is_empty() {
            println!("ghq root: {env_root} (from $GHQ_ROOT)");
        }
    } else {
        println!("ghq root: ~/ghq (default)");
    }
    if cfg.ghq.owners.is_empty() {
        println!("(no owner mappings configured)");
        return Ok(());
    }
    for (key, profile) in &cfg.ghq.owners {
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
    cfg.ghq.owners.insert(owner.to_string(), profile.to_string());
    cfg.save(&path)?;
    println!("mapped {owner} -> {profile}");
    Ok(())
}

pub fn unmap(owner: &str) -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    if cfg.ghq.owners.shift_remove(owner).is_some() {
        cfg.save(&path)?;
        println!("removed {owner}");
    } else {
        println!("no mapping for {owner}");
    }
    Ok(())
}

pub fn install() -> Result<()> {
    ensure(EnsureMode::Interactive)?;
    Ok(())
}

pub fn set_root(root: Option<String>) -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    match root {
        Some(r) if r.is_empty() => cfg.ghq.root = None,
        Some(r) => cfg.ghq.root = Some(r),
        None => cfg.ghq.root = None,
    }
    cfg.save(&path)?;
    match cfg.ghq.root.as_deref() {
        Some(r) => println!("ghq root set to {r}"),
        None => println!("ghq root cleared (will use $GHQ_ROOT or ~/ghq)"),
    }
    Ok(())
}
