//! Git remote-based profile detection.
//!
//! Walks up from `cwd` to find a Git working tree, resolves the actual
//! `.git/config` (handling linked worktrees and submodules), parses
//! `[remote "<name>"]` URL, and extracts `<host>/<owner>`.
//!
//! All file I/O is direct (no `git` subprocess) so this stays cheap on the
//! shim hot path.

use crate::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

/// Find the `<host>/<owner>` pair for the git repo containing `cwd`, if any.
///
/// Returns `None` when:
/// - no `.git` is found walking upward
/// - the resolved repo has no remote of the configured name
/// - the remote URL cannot be parsed into `<host>/<owner>`
pub fn detect_owner(cwd: &Path, config: &Config) -> Option<String> {
    let git_dir = find_git_dir(cwd)?;
    let common_dir = resolve_commondir(&git_dir);
    let url = read_remote_url(&common_dir, &config.git.remote)?;
    parse_owner(&url)
}

/// Walk up from `cwd` until a `.git` entry is found. Returns its absolute path.
/// `.git` may be a directory (normal repo) or a file (worktree / submodule).
fn find_git_dir(cwd: &Path) -> Option<PathBuf> {
    let canonical = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let mut dir: &Path = &canonical;
    loop {
        let candidate = dir.join(".git");
        if candidate.symlink_metadata().is_ok() {
            return Some(candidate);
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return None,
        }
    }
}

/// Given a path to `.git` (which may be a file or a directory), return the
/// path to the directory whose `config` we should read.
///
/// - `.git` is a directory → return it as-is.
/// - `.git` is a file (linked worktree / submodule) → read `gitdir: <path>`,
///   resolve relative paths against the parent of `.git`, then look for a
///   `commondir` file inside the gitdir; if present, the actual config lives
///   in `<gitdir>/<commondir>` (typically the main repo's `.git`).
pub fn resolve_commondir(git_path: &Path) -> PathBuf {
    let meta = match fs::symlink_metadata(git_path) {
        Ok(m) => m,
        Err(_) => return git_path.to_path_buf(),
    };
    if meta.is_dir() {
        return git_path.to_path_buf();
    }
    if !meta.is_file() {
        return git_path.to_path_buf();
    }
    let contents = match fs::read_to_string(git_path) {
        Ok(s) => s,
        Err(_) => return git_path.to_path_buf(),
    };
    let gitdir_line = contents
        .lines()
        .find_map(|l| l.strip_prefix("gitdir:").map(str::trim));
    let gitdir = match gitdir_line {
        Some(p) => p,
        None => return git_path.to_path_buf(),
    };
    let parent = git_path.parent().unwrap_or_else(|| Path::new("."));
    let gitdir_abs = if Path::new(gitdir).is_absolute() {
        PathBuf::from(gitdir)
    } else {
        normalize(&parent.join(gitdir))
    };

    // Linked worktrees and submodules expose a `commondir` file pointing at
    // the main repo's `.git` directory (relative to the linked gitdir).
    let commondir_marker = gitdir_abs.join("commondir");
    if let Ok(rel) = fs::read_to_string(&commondir_marker) {
        let rel = rel.trim();
        if !rel.is_empty() {
            let resolved = if Path::new(rel).is_absolute() {
                PathBuf::from(rel)
            } else {
                normalize(&gitdir_abs.join(rel))
            };
            return resolved;
        }
    }
    gitdir_abs
}

/// Read the URL of the named remote from `<git_dir>/config`. Performs a
/// minimal INI walk — we don't try to be a complete git config parser, only
/// to handle the patterns git itself writes.
pub fn read_remote_url(git_dir: &Path, remote: &str) -> Option<String> {
    let config_path = git_dir.join("config");
    let contents = fs::read_to_string(&config_path).ok()?;
    let target_header = format!("[remote \"{remote}\"]");
    let mut in_section = false;
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == target_header;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("url") {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Extract `<host>/<owner>` from a git remote URL.
///
/// Supported shapes:
/// - `git@github.com:Acme/widget.git`
/// - `ssh://git@github.com/Acme/widget.git`
/// - `https://github.com/Acme/widget.git`
/// - `https://user@github.com/Acme/widget`
/// - `git://github.com/Acme/widget.git`
pub fn parse_owner(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // scp-like: user@host:path
    if !url.contains("://") {
        if let Some((host_part, path)) = url.split_once(':') {
            let host = host_part
                .rsplit_once('@')
                .map(|(_, h)| h)
                .unwrap_or(host_part);
            let owner = first_segment(path)?;
            if !host.is_empty() && !owner.is_empty() {
                return Some(format!("{host}/{owner}"));
            }
        }
        return None;
    }

    // scheme://[user@]host[:port]/owner/repo[.git]
    let (_scheme, rest) = url.split_once("://")?;
    let (authority, path) = rest.split_once('/')?;
    let host_with_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host_with_port
        .split_once(':')
        .map(|(h, _)| h)
        .unwrap_or(host_with_port);
    let owner = first_segment(path)?;
    if host.is_empty() || owner.is_empty() {
        return None;
    }
    Some(format!("{host}/{owner}"))
}

fn first_segment(path: &str) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    let mut iter = trimmed.split('/');
    let owner = iter.next()?.trim_end_matches(".git");
    if owner.is_empty() {
        return None;
    }
    Some(owner.to_string())
}

/// Best-effort path normalization that resolves `..` and `.` without touching
/// the filesystem. We use this to combine the parent of a `.git` file with a
/// relative `gitdir:` value.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;

    fn write(path: impl AsRef<Path>, body: &str) {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    #[test]
    fn parse_owner_scp_form() {
        assert_eq!(
            parse_owner("git@github.com:Acme/widget.git").as_deref(),
            Some("github.com/Acme")
        );
        assert_eq!(
            parse_owner("git@github.com:Acme/widget").as_deref(),
            Some("github.com/Acme")
        );
    }

    #[test]
    fn parse_owner_ssh_url() {
        assert_eq!(
            parse_owner("ssh://git@github.com/Acme/widget.git").as_deref(),
            Some("github.com/Acme")
        );
    }

    #[test]
    fn parse_owner_https() {
        assert_eq!(
            parse_owner("https://github.com/Acme/widget.git").as_deref(),
            Some("github.com/Acme")
        );
        assert_eq!(
            parse_owner("https://user@github.com/Acme/widget").as_deref(),
            Some("github.com/Acme")
        );
        assert_eq!(
            parse_owner("https://github.example.com:8443/Org/repo.git").as_deref(),
            Some("github.example.com/Org")
        );
    }

    #[test]
    fn parse_owner_rejects_garbage() {
        assert!(parse_owner("").is_none());
        assert!(parse_owner("not-a-url").is_none());
        assert!(parse_owner("https://no-path-here").is_none());
    }

    #[test]
    fn read_remote_url_finds_origin() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        write(
            git_dir.join("config"),
            r#"[core]
    repositoryformatversion = 0
[remote "origin"]
    url = git@github.com:Acme/widget.git
    fetch = +refs/heads/*:refs/remotes/origin/*
[remote "upstream"]
    url = git@github.com:Original/widget.git
"#,
        );
        assert_eq!(
            read_remote_url(&git_dir, "origin").as_deref(),
            Some("git@github.com:Acme/widget.git")
        );
        assert_eq!(
            read_remote_url(&git_dir, "upstream").as_deref(),
            Some("git@github.com:Original/widget.git")
        );
        assert!(read_remote_url(&git_dir, "missing").is_none());
    }

    #[test]
    fn detect_owner_in_plain_repo() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let git_dir = repo.join(".git");
        write(
            git_dir.join("config"),
            "[remote \"origin\"]\n    url = https://github.com/Acme/widget.git\n",
        );
        let cfg = Config::default();
        assert_eq!(
            detect_owner(&repo, &cfg).as_deref(),
            Some("github.com/Acme")
        );
        // Subdirectory walks up.
        let sub = repo.join("src/lib");
        fs::create_dir_all(&sub).unwrap();
        assert_eq!(detect_owner(&sub, &cfg).as_deref(), Some("github.com/Acme"));
    }

    #[test]
    fn detect_owner_in_linked_worktree() {
        // Layout:
        //   <tmp>/main/.git/config                      (real repo)
        //   <tmp>/main/.git/worktrees/wt/commondir      (contents: "../..")
        //   <tmp>/wt/.git                               (file: "gitdir: ../main/.git/worktrees/wt")
        let tmp = TempDir::new().unwrap();
        let main_repo = tmp.path().join("main");
        let main_git = main_repo.join(".git");
        write(
            main_git.join("config"),
            "[remote \"origin\"]\n    url = git@github.com:Acme/widget.git\n",
        );
        let wt_gitdir = main_git.join("worktrees").join("wt");
        write(wt_gitdir.join("commondir"), "../..\n");
        let wt = tmp.path().join("wt");
        fs::create_dir_all(&wt).unwrap();
        write(wt.join(".git"), "gitdir: ../main/.git/worktrees/wt\n");

        let cfg = Config::default();
        assert_eq!(
            detect_owner(&wt, &cfg).as_deref(),
            Some("github.com/Acme"),
            "worktree must follow .git file → gitdir → commondir back to main config"
        );
    }

    #[test]
    fn detect_owner_in_submodule() {
        // Layout:
        //   <tmp>/parent/.git/modules/sub/config       (real config)
        //   <tmp>/parent/sub/.git                      (file: "gitdir: ../.git/modules/sub")
        let tmp = TempDir::new().unwrap();
        let parent = tmp.path().join("parent");
        let sub_gitdir = parent.join(".git").join("modules").join("sub");
        write(
            sub_gitdir.join("config"),
            "[remote \"origin\"]\n    url = https://github.com/Acme/sub.git\n",
        );
        let sub = parent.join("sub");
        fs::create_dir_all(&sub).unwrap();
        write(sub.join(".git"), "gitdir: ../.git/modules/sub\n");

        let cfg = Config::default();
        assert_eq!(
            detect_owner(&sub, &cfg).as_deref(),
            Some("github.com/Acme"),
            "submodule must follow gitdir into <parent>/.git/modules/<name>"
        );
    }

    #[test]
    fn detect_owner_with_custom_remote() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let git_dir = repo.join(".git");
        write(
            git_dir.join("config"),
            r#"[remote "origin"]
    url = git@github.com:fork/widget.git
[remote "upstream"]
    url = git@github.com:Original/widget.git
"#,
        );
        let mut cfg = Config::default();
        cfg.git.remote = "upstream".into();
        assert_eq!(
            detect_owner(&repo, &cfg).as_deref(),
            Some("github.com/Original")
        );
    }

    #[test]
    fn detect_owner_no_remote_returns_none() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        let git_dir = repo.join(".git");
        write(git_dir.join("config"), "[core]\n    bare = false\n");
        let cfg = Config::default();
        assert!(detect_owner(&repo, &cfg).is_none());
    }

    #[test]
    fn detect_owner_no_git_dir_returns_none() {
        let tmp = TempDir::new().unwrap();
        let cfg = Config::default();
        assert!(detect_owner(tmp.path(), &cfg).is_none());
    }
}
