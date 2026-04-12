//! Profile resolution from a starting directory.

use crate::paths::MARKER_FILENAME;
use anyhow::Result;
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
}
