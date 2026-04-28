//! Profile resolution from a starting directory.

use crate::config::Config;
use crate::paths::MARKER_FILENAME;
use anyhow::Result;
use globset::Glob;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MAX_MARKER_BYTES: u64 = 64 * 1024;

pub fn find_marker_profile(start: &Path) -> Result<Option<String>> {
    let mut dir: PathBuf = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    loop {
        if let Some(name) = read_marker(&dir.join(MARKER_FILENAME))? {
            return Ok(Some(name));
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return Ok(None),
        }
    }
}

fn read_marker(path: &Path) -> Result<Option<String>> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    if !meta.is_file() || meta.len() > MAX_MARKER_BYTES {
        return Ok(None);
    }
    let contents = fs::read_to_string(path).unwrap_or_default();
    let trimmed = contents.lines().next().unwrap_or("").trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

pub fn find_config_profile(cwd: &Path, config: &Config) -> Option<String> {
    let canonical = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    for (pattern, profile) in &config.directories {
        let expanded = shellexpand::full(pattern).ok()?.into_owned();
        let matcher = match Glob::new(&expanded) {
            Ok(g) => g.compile_matcher(),
            Err(_) => continue,
        };
        if matcher.is_match(&canonical) {
            return Some(profile.clone());
        }
    }
    None
}

/// Resolve via the `[ghq.owners]` mapping.
///
/// If `cwd` lives under the ghq root and matches `<root>/<host>/<owner>/<repo>...`,
/// look up `<host>/<owner>` in `config.ghq.owners`.
pub fn find_ghq_profile(cwd: &Path, config: &Config) -> Option<String> {
    if config.ghq.owners.is_empty() {
        return None;
    }
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
    let key = format!("{}/{}", parts[0], parts[1]);
    config.ghq.owners.get(&key).cloned()
}

/// Resolve the ghq root directory.
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

pub fn resolve(cwd: &Path, config: &Config) -> String {
    if let Some(forced) = crate::env::forced_profile() {
        return forced;
    }
    if let Ok(Some(name)) = find_marker_profile(cwd) {
        return name;
    }
    if let Some(name) = find_config_profile(cwd, config) {
        return name;
    }
    if let Some(name) = find_ghq_profile(cwd, config) {
        return name;
    }
    config.default_profile.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn no_marker_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(find_marker_profile(tmp.path()).unwrap(), None);
    }

    #[test]
    fn finds_in_parent() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "work\n").unwrap();
        let sub = tmp.path().join("nested/deep");
        fs::create_dir_all(&sub).unwrap();
        assert_eq!(find_marker_profile(&sub).unwrap(), Some("work".into()));
    }

    #[test]
    fn nearest_wins() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "outer\n").unwrap();
        let inner = tmp.path().join("inner");
        fs::create_dir(&inner).unwrap();
        fs::write(inner.join(".ccdirenv"), "inner-profile\n").unwrap();
        assert_eq!(
            find_marker_profile(&inner).unwrap(),
            Some("inner-profile".into())
        );
    }

    #[test]
    fn empty_marker_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "   \n").unwrap();
        assert_eq!(find_marker_profile(tmp.path()).unwrap(), None);
    }

    #[test]
    fn directory_marker_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".ccdirenv")).unwrap();
        assert_eq!(find_marker_profile(tmp.path()).unwrap(), None);
    }

    #[test]
    fn first_match_wins() {
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let dir = root.join("a").join("b");
        fs::create_dir_all(&dir).unwrap();
        let mut cfg = Config::default();
        cfg.directories
            .insert(format!("{}/**", root.display()), "first".into());
        cfg.directories
            .insert(format!("{}/a/**", root.display()), "second".into());
        assert_eq!(find_config_profile(&dir, &cfg), Some("first".into()));
    }

    #[test]
    fn no_match_returns_none() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config::default();
        assert_eq!(find_config_profile(tmp.path(), &cfg), None);
    }

    #[test]
    #[serial_test::serial]
    fn forced_env_wins() {
        std::env::set_var("CCDIRENV_PROFILE", "forced");
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "marker\n").unwrap();
        let cfg = Config::default();
        assert_eq!(resolve(tmp.path(), &cfg), "forced");
        std::env::remove_var("CCDIRENV_PROFILE");
    }

    #[test]
    #[serial_test::serial]
    fn default_when_nothing() {
        std::env::remove_var("CCDIRENV_PROFILE");
        let tmp = TempDir::new().unwrap();
        let cfg = Config {
            default_profile: "myDefault".into(),
            ..Config::default()
        };
        assert_eq!(resolve(tmp.path(), &cfg), "myDefault");
    }

    #[test]
    #[serial_test::serial]
    fn marker_beats_config() {
        std::env::remove_var("CCDIRENV_PROFILE");
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "marker-wins\n").unwrap();
        let mut cfg = Config::default();
        cfg.directories
            .insert(format!("{}/**", tmp.path().display()), "cfg".into());
        assert_eq!(resolve(tmp.path(), &cfg), "marker-wins");
    }

    #[test]
    #[serial_test::serial]
    fn ghq_owner_resolves() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let repo = root.join("github.com/Acme/widget");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        cfg.ghq
            .owners
            .insert("github.com/Acme".into(), "work".into());
        assert_eq!(find_ghq_profile(&repo, &cfg), Some("work".into()));
        // Subdirectories of the repo also resolve.
        let sub = repo.join("src/lib");
        fs::create_dir_all(&sub).unwrap();
        assert_eq!(find_ghq_profile(&sub, &cfg), Some("work".into()));
    }

    #[test]
    #[serial_test::serial]
    fn ghq_unknown_owner_returns_none() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let repo = root.join("github.com/Stranger/repo");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        cfg.ghq
            .owners
            .insert("github.com/Acme".into(), "work".into());
        assert_eq!(find_ghq_profile(&repo, &cfg), None);
    }

    #[test]
    #[serial_test::serial]
    fn ghq_outside_root_returns_none() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let outside = TempDir::new().unwrap();
        let outside_canonical = fs::canonicalize(outside.path()).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        cfg.ghq
            .owners
            .insert("github.com/Acme".into(), "work".into());
        assert_eq!(find_ghq_profile(&outside_canonical, &cfg), None);
    }

    #[test]
    #[serial_test::serial]
    fn directories_beat_ghq_owner() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let sub = root.join("github.com/Acme/widget/src");
        fs::create_dir_all(&sub).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.root = Some(root.display().to_string());
        cfg.ghq
            .owners
            .insert("github.com/Acme".into(), "ghq-mapped".into());
        // Glob over the repo subtree should beat ghq owner mapping.
        cfg.directories.insert(
            format!("{}/github.com/Acme/widget/**", root.display()),
            "explicit".into(),
        );
        assert_eq!(resolve(&sub, &cfg), "explicit");
    }

    #[test]
    #[serial_test::serial]
    fn ghq_root_uses_env_var_when_unset() {
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let canonical = fs::canonicalize(tmp.path()).unwrap();
        std::env::set_var("GHQ_ROOT", canonical.display().to_string());
        let cfg = Config::default();
        assert_eq!(ghq_root(&cfg), Some(canonical));
        std::env::remove_var("GHQ_ROOT");
    }

    #[test]
    #[serial_test::serial]
    fn ghq_resolves_via_resolve_function() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let repo = root.join("github.com/Acme/widget");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.default_profile = "fallback".into();
        cfg.ghq.root = Some(root.display().to_string());
        cfg.ghq
            .owners
            .insert("github.com/Acme".into(), "work".into());
        assert_eq!(resolve(&repo, &cfg), "work");
    }
}
