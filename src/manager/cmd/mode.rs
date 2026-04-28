//! `ccdirenv mode` subcommand: set / show the discovery mode.
//!
//! A "mode" is shorthand for a coordinated set of `[ghq] enabled`,
//! `[git] enabled`, and `discovery_priority` values:
//!
//! | mode  | ghq.enabled | git.enabled | discovery_priority |
//! |-------|-------------|-------------|--------------------|
//! | ghq   | true        | true        | ghq                |
//! | git   | false       | true        | git                |
//! | both  | true        | true        | git                |
//! | off   | false       | false       | git                |

use crate::config::{Config, DiscoveryPriority};
use crate::paths::config_file;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Ghq,
    Git,
    Both,
    Off,
}

impl Mode {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "ghq" => Ok(Mode::Ghq),
            "git" => Ok(Mode::Git),
            "both" => Ok(Mode::Both),
            "off" | "none" | "skip" => Ok(Mode::Off),
            other => Err(anyhow!(
                "unknown mode '{other}' (expected: ghq, git, both, off)"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Ghq => "ghq",
            Mode::Git => "git",
            Mode::Both => "both",
            Mode::Off => "off",
        }
    }

    pub fn apply(self, cfg: &mut Config) {
        match self {
            Mode::Ghq => {
                cfg.ghq.enabled = true;
                cfg.git.enabled = true;
                cfg.discovery_priority = DiscoveryPriority::Ghq;
            }
            Mode::Git => {
                cfg.ghq.enabled = false;
                cfg.git.enabled = true;
                cfg.discovery_priority = DiscoveryPriority::Git;
            }
            Mode::Both => {
                cfg.ghq.enabled = true;
                cfg.git.enabled = true;
                cfg.discovery_priority = DiscoveryPriority::Git;
            }
            Mode::Off => {
                cfg.ghq.enabled = false;
                cfg.git.enabled = false;
            }
        }
    }

    /// Best-effort inference for `mode show`: which named mode does the
    /// current config most closely resemble?
    pub fn from_config(cfg: &Config) -> Self {
        match (cfg.ghq.enabled, cfg.git.enabled, cfg.discovery_priority) {
            (false, false, _) => Mode::Off,
            (false, true, _) => Mode::Git,
            (true, true, DiscoveryPriority::Ghq) => Mode::Ghq,
            (true, true, DiscoveryPriority::Git) => Mode::Both,
            (true, false, _) => Mode::Ghq,
        }
    }
}

pub fn set(mode: &str) -> Result<()> {
    let parsed = Mode::parse(mode)?;
    let path = config_file()?;
    let mut cfg = Config::load(&path)?;
    parsed.apply(&mut cfg);
    cfg.save(&path)?;
    println!("discovery mode set to {}", parsed.name());
    println!("  ghq.enabled = {}", cfg.ghq.enabled);
    println!("  git.enabled = {}", cfg.git.enabled);
    println!(
        "  discovery_priority = {}",
        match cfg.discovery_priority {
            DiscoveryPriority::Git => "git",
            DiscoveryPriority::Ghq => "ghq",
        }
    );
    Ok(())
}

pub fn show() -> Result<()> {
    let cfg = Config::load(&config_file()?)?;
    let mode = Mode::from_config(&cfg);
    println!("mode: {}", mode.name());
    println!("  ghq.enabled = {}", cfg.ghq.enabled);
    println!("  git.enabled = {}", cfg.git.enabled);
    println!(
        "  discovery_priority = {}",
        match cfg.discovery_priority {
            DiscoveryPriority::Git => "git",
            DiscoveryPriority::Ghq => "ghq",
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_known_aliases() {
        assert_eq!(Mode::parse("ghq").unwrap(), Mode::Ghq);
        assert_eq!(Mode::parse("GIT").unwrap(), Mode::Git);
        assert_eq!(Mode::parse("Both").unwrap(), Mode::Both);
        assert_eq!(Mode::parse("off").unwrap(), Mode::Off);
        assert_eq!(Mode::parse("none").unwrap(), Mode::Off);
        assert_eq!(Mode::parse("skip").unwrap(), Mode::Off);
        assert!(Mode::parse("unknown").is_err());
    }

    #[test]
    fn apply_sets_expected_combo() {
        let mut cfg = Config::default();
        Mode::Ghq.apply(&mut cfg);
        assert!(cfg.ghq.enabled);
        assert!(cfg.git.enabled);
        assert_eq!(cfg.discovery_priority, DiscoveryPriority::Ghq);

        Mode::Git.apply(&mut cfg);
        assert!(!cfg.ghq.enabled);
        assert!(cfg.git.enabled);
        assert_eq!(cfg.discovery_priority, DiscoveryPriority::Git);

        Mode::Both.apply(&mut cfg);
        assert!(cfg.ghq.enabled);
        assert!(cfg.git.enabled);
        assert_eq!(cfg.discovery_priority, DiscoveryPriority::Git);

        Mode::Off.apply(&mut cfg);
        assert!(!cfg.ghq.enabled);
        assert!(!cfg.git.enabled);
    }

    #[test]
    fn from_config_round_trips() {
        for mode in [Mode::Ghq, Mode::Git, Mode::Both, Mode::Off] {
            let mut cfg = Config::default();
            mode.apply(&mut cfg);
            assert_eq!(Mode::from_config(&cfg), mode);
        }
    }
}
