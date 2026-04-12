//! config.toml schema and loader.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
    #[serde(default)]
    pub directories: indexmap::IndexMap<String, String>,
}

fn default_profile_name() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_profile: default_profile_name(),
            directories: indexmap::IndexMap::new(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).with_context(|| format!("parsing {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, s)?;
        Ok(())
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
    }

    #[test]
    fn parse_full_example() {
        let src = r#"
default_profile = "personal"
[directories]
"~/work/**" = "work"
"~/oss/**" = "personal"
"#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.default_profile, "personal");
        assert_eq!(cfg.directories.get("~/work/**"), Some(&"work".to_string()));
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
        cfg.save(&path).unwrap();
        let reloaded = Config::load(&path).unwrap();
        assert_eq!(reloaded.default_profile, "personal");
        assert_eq!(reloaded.directories.get("~/work/**"), Some(&"work".into()));
    }
}
