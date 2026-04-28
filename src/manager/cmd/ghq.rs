//! `ccdirenv ghq` subcommands.
//!
//! As of v0.3.0, the canonical owner-management commands live under
//! `ccdirenv owners ...`, but `ccdirenv ghq map/unmap/list` are kept as
//! aliases for v0.2.x users. This module also owns ghq-specific knobs:
//! enable/disable, root override, install.

use crate::config::Config;
use crate::manager::cmd::ensure_ghq::{ensure, EnsureMode};
use crate::manager::cmd::owners;
use crate::paths::config_file;
use anyhow::Result;

/// `ccdirenv ghq list` — alias of `ccdirenv owners list` plus ghq context.
pub fn list() -> Result<()> {
    let cfg = Config::load(&config_file()?)?;
    println!("ghq enabled: {}", cfg.ghq.enabled);
    if let Some(root) = cfg.ghq.root.as_ref() {
        println!("ghq root: {root}");
    } else if let Ok(env_root) = std::env::var("GHQ_ROOT") {
        if !env_root.is_empty() {
            println!("ghq root: {env_root} (from $GHQ_ROOT)");
        } else {
            println!("ghq root: ~/ghq (default)");
        }
    } else {
        println!("ghq root: ~/ghq (default)");
    }
    println!();
    owners::list()
}

pub fn map(owner: &str, profile: &str) -> Result<()> {
    owners::map(owner, profile)
}

pub fn unmap(owner: &str) -> Result<()> {
    owners::unmap(owner)
}

pub fn enable() -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.ghq.enabled = true;
    cfg.save(&path)?;
    println!("ghq detection enabled");
    Ok(())
}

pub fn disable() -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.ghq.enabled = false;
    cfg.save(&path)?;
    println!("ghq detection disabled");
    Ok(())
}

pub fn install() -> Result<()> {
    ensure(EnsureMode::Interactive)?;
    Ok(())
}

pub fn set_root(root: Option<String>) -> Result<()> {
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    cfg.ghq.root = match root {
        Some(r) if r.is_empty() => None,
        Some(r) => Some(r),
        None => None,
    };
    cfg.save(&path)?;
    match cfg.ghq.root.as_deref() {
        Some(r) => println!("ghq root set to {r}"),
        None => println!("ghq root cleared (will use $GHQ_ROOT or ~/ghq)"),
    }
    Ok(())
}
