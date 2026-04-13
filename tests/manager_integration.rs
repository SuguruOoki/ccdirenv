use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn init_creates_layout() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join(".ccdirenv");
    let output = Command::cargo_bin("ccdirenv")
        .unwrap()
        .arg("init")
        .env("CCDIRENV_HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(home.join("bin/claude").symlink_metadata().is_ok());
    assert!(home.join("profiles/default").is_dir());
    assert!(home.join("config.toml").is_file());
}

#[test]
fn which_prints_marker_profile() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join(".ccdirenv");
    let work = tmp.path().join("project");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join(".ccdirenv"), "work\n").unwrap();
    let output = Command::cargo_bin("ccdirenv")
        .unwrap()
        .arg("which")
        .current_dir(&work)
        .env("CCDIRENV_HOME", &home)
        .env_remove("CCDIRENV_PROFILE")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("work\t"));
}
