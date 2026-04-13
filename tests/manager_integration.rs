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
