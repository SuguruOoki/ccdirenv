//! Profile resolution from a starting directory.

use crate::config::Config;
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
    let meta = match fs::symlink_metadata(path) { Ok(m) => m, Err(_) => return Ok(None) };
    if !meta.is_file() || meta.len() > MAX_MARKER_BYTES { return Ok(None); }
    let contents = fs::read_to_string(path).unwrap_or_default();
    let trimmed = contents.lines().next().unwrap_or("").trim();
    if trimmed.is_empty() { Ok(None) } else { Ok(Some(trimmed.to_string())) }
}

pub fn find_config_profile(cwd: &Path, config: &Config) -> Option<String> {
    let canonical = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    for (pattern, profile) in &config.directories {
        let expanded = shellexpand::full(pattern).ok()?.into_owned();
        let matcher = match Glob::new(&expanded) {
            Ok(g) => g.compile_matcher(),
            Err(_) => continue,
        };
        if matcher.is_match(&canonical) { return Some(profile.clone()); }
    }
    None
}

pub fn resolve(cwd: &Path, config: &Config) -> String {
    if let Some(forced) = crate::env::forced_profile() { return forced; }
    if let Ok(Some(name)) = find_marker_profile(cwd) { return name; }
    if let Some(name) = find_config_profile(cwd, config) { return name; }
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
        assert_eq!(find_marker_profile(&inner).unwrap(), Some("inner-profile".into()));
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
        cfg.directories.insert(format!("{}/**", root.display()), "first".into());
        cfg.directories.insert(format!("{}/a/**", root.display()), "second".into());
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
        let mut cfg = Config::default();
        cfg.default_profile = "myDefault".into();
        assert_eq!(resolve(tmp.path(), &cfg), "myDefault");
    }

    #[test]
    #[serial_test::serial]
    fn marker_beats_config() {
        std::env::remove_var("CCDIRENV_PROFILE");
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".ccdirenv"), "marker-wins\n").unwrap();
        let mut cfg = Config::default();
        cfg.directories.insert(format!("{}/**", tmp.path().display()), "cfg".into());
        assert_eq!(resolve(tmp.path(), &cfg), "marker-wins");
    }
}
