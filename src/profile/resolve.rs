//! Profile resolution from a starting directory.

use crate::config::{Config, DiscoveryPriority};
use crate::paths::MARKER_FILENAME;
use anyhow::Result;
use globset::Glob;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Look up an `<host>/<owner>` key in the shared owners map.
fn lookup_owner(owner: &str, config: &Config) -> Option<String> {
    config.owners.get(owner).cloned()
}

/// Run discovery in priority order. The first method that finds an owner
/// AND has it mapped wins.
pub fn find_discovery_profile(cwd: &Path, config: &Config) -> Option<String> {
    let try_ghq = || -> Option<String> {
        if !config.ghq.enabled {
            return None;
        }
        let owner = super::ghq::detect_owner(cwd, config)?;
        lookup_owner(&owner, config)
    };
    let try_git = || -> Option<String> {
        if !config.git.enabled {
            return None;
        }
        let owner = super::git::detect_owner(cwd, config)?;
        lookup_owner(&owner, config)
    };

    match config.discovery_priority {
        DiscoveryPriority::Ghq => try_ghq().or_else(try_git),
        DiscoveryPriority::Git => try_git().or_else(try_ghq),
    }
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
    if let Some(name) = find_discovery_profile(cwd, config) {
        return name;
    }
    config.default_profile.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn write(path: impl AsRef<Path>, body: &str) {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

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
    fn first_glob_match_wins() {
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
    #[serial]
    fn forced_env_wins() {
        std::env::set_var("CCDIRENV_PROFILE", "forced");
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "marker\n").unwrap();
        let cfg = Config::default();
        assert_eq!(resolve(tmp.path(), &cfg), "forced");
        std::env::remove_var("CCDIRENV_PROFILE");
    }

    #[test]
    #[serial]
    fn marker_beats_directories() {
        std::env::remove_var("CCDIRENV_PROFILE");
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "marker-wins\n").unwrap();
        let mut cfg = Config::default();
        cfg.directories
            .insert(format!("{}/**", tmp.path().display()), "cfg".into());
        assert_eq!(resolve(tmp.path(), &cfg), "marker-wins");
    }

    #[test]
    #[serial]
    fn directories_beat_discovery() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let sub = root.join("github.com/Acme/widget/src");
        fs::create_dir_all(&sub).unwrap();
        let mut cfg = Config::default();
        cfg.ghq.enabled = true;
        cfg.ghq.root = Some(root.display().to_string());
        cfg.owners
            .insert("github.com/Acme".into(), "ghq-mapped".into());
        cfg.directories.insert(
            format!("{}/github.com/Acme/widget/**", root.display()),
            "explicit".into(),
        );
        assert_eq!(resolve(&sub, &cfg), "explicit");
    }

    #[test]
    #[serial]
    fn ghq_priority_runs_ghq_first() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let ghq_root = fs::canonicalize(tmp.path()).unwrap();
        let repo = ghq_root.join("github.com/AcmeGhq/widget");
        let git_dir = repo.join(".git");
        write(
            git_dir.join("config"),
            "[remote \"origin\"]\n    url = git@github.com:AcmeGit/widget.git\n",
        );
        let mut cfg = Config::default();
        cfg.discovery_priority = DiscoveryPriority::Ghq;
        cfg.ghq.enabled = true;
        cfg.ghq.root = Some(ghq_root.display().to_string());
        cfg.git.enabled = true;
        cfg.owners
            .insert("github.com/AcmeGhq".into(), "by-ghq".into());
        cfg.owners
            .insert("github.com/AcmeGit".into(), "by-git".into());
        assert_eq!(resolve(&repo, &cfg), "by-ghq");
    }

    #[test]
    #[serial]
    fn git_priority_runs_git_first() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let ghq_root = fs::canonicalize(tmp.path()).unwrap();
        let repo = ghq_root.join("github.com/AcmeGhq/widget");
        let git_dir = repo.join(".git");
        write(
            git_dir.join("config"),
            "[remote \"origin\"]\n    url = git@github.com:AcmeGit/widget.git\n",
        );
        let mut cfg = Config::default();
        cfg.discovery_priority = DiscoveryPriority::Git;
        cfg.ghq.enabled = true;
        cfg.ghq.root = Some(ghq_root.display().to_string());
        cfg.git.enabled = true;
        cfg.owners
            .insert("github.com/AcmeGhq".into(), "by-ghq".into());
        cfg.owners
            .insert("github.com/AcmeGit".into(), "by-git".into());
        assert_eq!(resolve(&repo, &cfg), "by-git");
    }

    #[test]
    #[serial]
    fn falls_back_to_other_method_when_first_misses() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        // Path is under ghq root, but no .git → git detection misses.
        // git priority means git tried first, then ghq.
        let tmp = TempDir::new().unwrap();
        let ghq_root = fs::canonicalize(tmp.path()).unwrap();
        let repo = ghq_root.join("github.com/Acme/widget");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.discovery_priority = DiscoveryPriority::Git;
        cfg.ghq.enabled = true;
        cfg.ghq.root = Some(ghq_root.display().to_string());
        cfg.git.enabled = true;
        cfg.owners
            .insert("github.com/Acme".into(), "ghq-fallback".into());
        assert_eq!(resolve(&repo, &cfg), "ghq-fallback");
    }

    #[test]
    #[serial]
    fn discovery_disabled_falls_through_to_default() {
        std::env::remove_var("CCDIRENV_PROFILE");
        std::env::remove_var("GHQ_ROOT");
        let tmp = TempDir::new().unwrap();
        let ghq_root = fs::canonicalize(tmp.path()).unwrap();
        let repo = ghq_root.join("github.com/Acme/widget");
        fs::create_dir_all(&repo).unwrap();
        let mut cfg = Config::default();
        cfg.default_profile = "fallback".into();
        cfg.discovery_priority = DiscoveryPriority::Git;
        cfg.ghq.enabled = false;
        cfg.git.enabled = false;
        cfg.ghq.root = Some(ghq_root.display().to_string());
        cfg.owners
            .insert("github.com/Acme".into(), "would-have-matched".into());
        assert_eq!(resolve(&repo, &cfg), "fallback");
    }
}
