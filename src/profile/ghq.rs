//! ghq path-layout based profile detection.
//!
//! Given a ghq root (`~/ghq` by default), repos live at
//! `<root>/<host>/<owner>/<repo>`. We extract `<host>/<owner>` from the
//! current path and use it as the lookup key against the shared owners map.

use crate::config::Config;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Resolve `<host>/<owner>` for `cwd` if it lives under the ghq root.
pub fn detect_owner(cwd: &Path, config: &Config) -> Option<String> {
    let root = ghq_root(config)?;
    let canonical = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let canonical_root = fs::canonicalize(&root).unwrap_or(root);
    let rel = canonical.strip_prefix(&canonical_root).ok()?;
    let parts: Vec<&str> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    if parts.len() < 2 {
        return None;
    }
    Some(format!("{}/{}", parts[0], parts[1]))
}

/// Resolve the ghq root.
///
/// Priority:
/// 1. `[ghq] root = "..."` in config.toml
/// 2. `$GHQ_ROOT` environment variable
/// 3. `~/ghq` (the default ghq layout)
pub fn ghq_root(config: &Config) -> Option<PathBuf> {
    if let Some(r) = config.ghq.root.as_ref() {
        if let Ok(expanded) = shellexpand::full(r) {
            return Some(PathBuf::from(expanded.into_owned()));
        }
    }
    if let Ok(r) = std::env::var("GHQ_ROOT") {
        if !r.is_empty() {
            return Some(PathBuf::from(r));
        }
    }
    let home = dirs::home_dir()?;
    Some(home.join("ghq"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn detects_owner_under_root() {
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let repo = root.join("github.com/Acme/widget");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        assert_eq!(detect_owner(&repo, &cfg).as_deref(), Some("github.com/Acme"));
        // Subdir works too.
        let sub = repo.join("src/lib");
        fs::create_dir_all(&sub).unwrap();
        assert_eq!(detect_owner(&sub, &cfg).as_deref(), Some("github.com/Acme"));
    }

    #[test]
    #[serial]
    fn outside_root_returns_none() {
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let outside = TempDir::new().unwrap();
        let outside_canonical = fs::canonicalize(outside.path()).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        assert!(detect_owner(&outside_canonical, &cfg).is_none());
    }

    #[test]
    #[serial]
    fn shallow_path_returns_none() {
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let host_only = root.join("github.com");
        fs::create_dir_all(&host_only).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        assert!(detect_owner(&host_only, &cfg).is_none());
    }

    #[test]
    #[serial]
    fn ghq_root_uses_env_var() {
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let canonical = fs::canonicalize(tmp.path()).unwrap();
        std::env::set_var("GHQ_ROOT", canonical.display().to_string());
        let cfg = Config::default();
        assert_eq!(ghq_root(&cfg), Some(canonical));
        std::env::remove_var("GHQ_ROOT");
    }
}
