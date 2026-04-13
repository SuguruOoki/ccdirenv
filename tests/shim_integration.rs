use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn setup_shim(dir: &std::path::Path) -> PathBuf {
    let bin = assert_cmd::cargo::cargo_bin("ccdirenv");
    let shim_dir = dir.join("shim-bin");
    fs::create_dir_all(&shim_dir).unwrap();
    let shim_path = shim_dir.join("claude");
    symlink(&bin, &shim_path).unwrap();
    shim_path
}

fn setup_fake_claude(dir: &std::path::Path) -> PathBuf {
    let fake_dir = dir.join("real-bin");
    fs::create_dir_all(&fake_dir).unwrap();
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_claude.sh");
    let dst = fake_dir.join("claude");
    fs::copy(&src, &dst).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755)).unwrap();
    fake_dir
}

#[test]
fn passes_profile_config_dir_to_real_claude() {
    let tmp = TempDir::new().unwrap();
    let shim = setup_shim(tmp.path());
    let fake_dir = setup_fake_claude(tmp.path());

    let cwd = tmp.path().join("workdir");
    fs::create_dir_all(&cwd).unwrap();
    fs::write(cwd.join(".ccdirenv"), "work\n").unwrap();

    let ccdirenv_home = tmp.path().join(".ccdirenv");
    let path = format!(
        "{}:{}",
        shim.parent().unwrap().display(),
        fake_dir.display()
    );

    let output = Command::new(&shim)
        .args(["chat", "hi"])
        .current_dir(&cwd)
        .env("PATH", &path)
        .env("CCDIRENV_HOME", &ccdirenv_home)
        .env_remove("CCDIRENV_DISABLE")
        .env_remove("CCDIRENV_PROFILE")
        .output()
        .expect("shim ran");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = ccdirenv_home.join("profiles/work");
    assert!(
        stdout.contains(&format!("CLAUDE_CONFIG_DIR={}", expected.display())),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("ARGS: chat hi"), "stdout: {stdout}");
}

#[test]
fn falls_back_to_default() {
    let tmp = TempDir::new().unwrap();
    let shim = setup_shim(tmp.path());
    let fake_dir = setup_fake_claude(tmp.path());

    let cwd = tmp.path().join("plain");
    fs::create_dir_all(&cwd).unwrap();
    let ccdirenv_home = tmp.path().join(".ccdirenv");
    let path = format!(
        "{}:{}",
        shim.parent().unwrap().display(),
        fake_dir.display()
    );

    let output = Command::new(&shim)
        .current_dir(&cwd)
        .env("PATH", &path)
        .env("CCDIRENV_HOME", &ccdirenv_home)
        .env_remove("CCDIRENV_DISABLE")
        .env_remove("CCDIRENV_PROFILE")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profiles/default"), "stdout: {stdout}");
}

#[test]
fn honors_ccdirenv_disable() {
    let tmp = TempDir::new().unwrap();
    let shim = setup_shim(tmp.path());
    let fake_dir = setup_fake_claude(tmp.path());
    let path = format!(
        "{}:{}",
        shim.parent().unwrap().display(),
        fake_dir.display()
    );

    let output = Command::new(&shim)
        .env("PATH", &path)
        .env("CCDIRENV_DISABLE", "1")
        .env_remove("CLAUDE_CONFIG_DIR")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CLAUDE_CONFIG_DIR=unset"),
        "stdout: {stdout}"
    );
}
