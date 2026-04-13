//! Locate real `claude` in PATH excluding the shim directory.

use anyhow::{anyhow, Context, Result};
use std::env;
use std::path::{Path, PathBuf};

pub fn path_without(skip: &Path) -> String {
    let skip_canonical = std::fs::canonicalize(skip).ok();
    let entries: Vec<PathBuf> = env::var_os("PATH")
        .map(|p| env::split_paths(&p).collect())
        .unwrap_or_default();
    let filtered: Vec<PathBuf> = entries
        .into_iter()
        .filter(|entry| {
            if entry == skip {
                return false;
            }
            !matches!(std::fs::canonicalize(entry), Ok(c) if Some(&c) == skip_canonical.as_ref())
        })
        .collect();
    env::join_paths(filtered)
        .ok()
        .and_then(|s| s.into_string().ok())
        .unwrap_or_default()
}

pub fn locate_real_claude(shim_dir: &Path) -> Result<PathBuf> {
    let cleaned = path_without(shim_dir);
    let me_canonical = env::current_exe()
        .ok()
        .and_then(|p| std::fs::canonicalize(&p).ok());

    let candidates = which::which_in_all("claude", Some(&cleaned), env::current_dir()?)
        .context("PATH search failed")?;

    for candidate in candidates {
        if candidate == shim_dir.join("claude") {
            continue;
        }
        if let Ok(canon) = std::fs::canonicalize(&candidate) {
            if Some(&canon) == me_canonical.as_ref() {
                continue;
            }
        }
        return Ok(candidate);
    }
    Err(anyhow!(
        "`claude` not found in PATH (after skipping shim and self-matches)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn removes_exact_match() {
        let tmp = TempDir::new().unwrap();
        let shim = tmp.path().join("shim");
        let other = tmp.path().join("other");
        std::fs::create_dir_all(&shim).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        env::set_var("PATH", env::join_paths([&shim, &other]).unwrap());
        let out = path_without(&shim);
        assert!(!out.contains(shim.to_str().unwrap()));
        assert!(out.contains(other.to_str().unwrap()));
    }

    #[test]
    #[serial]
    fn errors_when_only_shim_has_claude() {
        let tmp = TempDir::new().unwrap();
        let shim = tmp.path().join("bin");
        std::fs::create_dir_all(&shim).unwrap();
        let fake = shim.join("claude");
        std::fs::write(&fake, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        env::set_var("PATH", shim.to_str().unwrap());
        assert!(locate_real_claude(&shim).is_err());
    }
}
