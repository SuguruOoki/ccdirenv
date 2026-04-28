//! config.toml schema and loader.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryPriority {
    Git,
    Ghq,
}

impl Default for DiscoveryPriority {
    fn default() -> Self {
        DiscoveryPriority::Git
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
    /// Which discovery method runs first. Default: git.
    #[serde(default)]
    pub discovery_priority: DiscoveryPriority,
    #[serde(default)]
    pub directories: indexmap::IndexMap<String, String>,
    /// Owner → profile map (shared by ghq and git discovery).
    /// Backward compat: legacy `[ghq.owners]` is merged into this at load time.
    #[serde(default, skip_serializing_if = "indexmap::IndexMap::is_empty")]
    pub owners: indexmap::IndexMap<String, String>,
    #[serde(default, skip_serializing_if = "GhqConfig::is_empty_for_serialize")]
    pub ghq: GhqConfig,
    #[serde(default, skip_serializing_if = "GitConfig::is_default")]
    pub git: GitConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GhqConfig {
    /// When true, ghq path-based detection participates in resolution.
    #[serde(default)]
    pub enabled: bool,
    /// Optional override for ghq root. Defaults to $GHQ_ROOT, then `~/ghq`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// Legacy v0.2.x location for owner mappings. Merged into top-level
    /// `[owners]` at load time. Kept for backward compatibility.
    #[serde(default, skip_serializing_if = "indexmap::IndexMap::is_empty")]
    pub owners: indexmap::IndexMap<String, String>,
}

impl GhqConfig {
    pub fn is_empty_for_serialize(&self) -> bool {
        !self.enabled && self.root.is_none() && self.owners.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitConfig {
    /// When true, .git/config-based detection participates in resolution.
    #[serde(default = "default_git_enabled")]
    pub enabled: bool,
    /// Which remote to inspect. Default "origin".
    #[serde(default = "default_git_remote")]
    pub remote: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: default_git_enabled(),
            remote: default_git_remote(),
        }
    }
}

impl GitConfig {
    fn is_default(&self) -> bool {
        self.enabled == default_git_enabled() && self.remote == default_git_remote()
    }
}

fn default_git_enabled() -> bool {
    true
}

fn default_git_remote() -> String {
    "origin".to_string()
}

fn default_profile_name() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_profile: default_profile_name(),
            discovery_priority: DiscoveryPriority::default(),
            directories: indexmap::IndexMap::new(),
            owners: indexmap::IndexMap::new(),
            ghq: GhqConfig::default(),
            git: GitConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let mut cfg: Self = match fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
        };
        let had_legacy_ghq_owners = !cfg.ghq.owners.is_empty();
        cfg.merge_legacy_ghq_owners();
        if had_legacy_ghq_owners {
            // v0.2 configs implicitly meant "ghq detection on". Preserve that
            // unless the user has explicitly disabled it.
            cfg.ghq.enabled = true;
        }
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, s)?;
        Ok(())
    }

    /// Merge legacy `[ghq.owners]` entries into top-level `[owners]`.
    /// Existing top-level keys take precedence.
    fn merge_legacy_ghq_owners(&mut self) {
        if self.ghq.owners.is_empty() {
            return;
        }
        let legacy = std::mem::take(&mut self.ghq.owners);
        for (k, v) in legacy {
            self.owners.entry(k).or_insert(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config::load(&tmp.path().join("nope.toml")).unwrap();
        assert_eq!(cfg.default_profile, "default");
        assert!(cfg.directories.is_empty());
        assert!(cfg.owners.is_empty());
        assert!(!cfg.ghq.enabled);
        assert!(cfg.git.enabled);
        assert_eq!(cfg.git.remote, "origin");
        assert_eq!(cfg.discovery_priority, DiscoveryPriority::Git);
    }

    #[test]
    fn parse_legacy_ghq_owners_merges_into_owners() {
        let src = r#"
default_profile = "default"

[ghq.owners]
"github.com/Acme" = "work"
"github.com/me"   = "personal"
"#;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("c.toml");
        std::fs::write(&path, src).unwrap();
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.owners.get("github.com/Acme").map(String::as_str), Some("work"));
        assert_eq!(cfg.owners.get("github.com/me").map(String::as_str), Some("personal"));
        // Legacy ghq.owners now empty after merge.
        assert!(cfg.ghq.owners.is_empty());
        // v0.2 implied ghq detection on — preserve that for legacy users.
        assert!(cfg.ghq.enabled);
    }

    #[test]
    fn top_level_owners_wins_over_legacy_ghq_owners() {
        let src = r#"
[owners]
"github.com/Acme" = "winner"

[ghq.owners]
"github.com/Acme" = "loser"
"#;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("c.toml");
        std::fs::write(&path, src).unwrap();
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.owners.get("github.com/Acme").map(String::as_str), Some("winner"));
    }

    #[test]
    fn parse_full_v3_schema() {
        let src = r#"
default_profile = "personal"
discovery_priority = "ghq"

[ghq]
enabled = true
root = "~/repos"

[git]
enabled = false
remote = "upstream"

[owners]
"github.com/Acme" = "work"

[directories]
"~/work/**" = "work"
"#;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("c.toml");
        std::fs::write(&path, src).unwrap();
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.default_profile, "personal");
        assert_eq!(cfg.discovery_priority, DiscoveryPriority::Ghq);
        assert!(cfg.ghq.enabled);
        assert_eq!(cfg.ghq.root.as_deref(), Some("~/repos"));
        assert!(!cfg.git.enabled);
        assert_eq!(cfg.git.remote, "upstream");
        assert_eq!(cfg.owners.get("github.com/Acme").map(String::as_str), Some("work"));
    }

    #[test]
    fn directories_preserve_insertion_order() {
        let src = r#"
[directories]
"~/a/**" = "one"
"~/b/**" = "two"
"~/c/**" = "three"
"#;
        let cfg: Config = toml::from_str(src).unwrap();
        let keys: Vec<&str> = cfg.directories.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["~/a/**", "~/b/**", "~/c/**"]);
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut cfg = Config {
            default_profile: "personal".into(),
            ..Config::default()
        };
        cfg.directories.insert("~/work/**".into(), "work".into());
        cfg.owners
            .insert("github.com/Acme".into(), "work".into());
        cfg.ghq.enabled = true;
        cfg.discovery_priority = DiscoveryPriority::Ghq;
        cfg.save(&path).unwrap();
        let reloaded = Config::load(&path).unwrap();
        assert_eq!(reloaded.default_profile, "personal");
        assert_eq!(reloaded.directories.get("~/work/**"), Some(&"work".into()));
        assert_eq!(reloaded.owners.get("github.com/Acme"), Some(&"work".into()));
        assert!(reloaded.ghq.enabled);
        assert_eq!(reloaded.discovery_priority, DiscoveryPriority::Ghq);
    }
}
