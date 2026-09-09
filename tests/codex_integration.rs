use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct Fixture {
    tmp: TempDir,
    home: PathBuf,
    real: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("ccdirenv");
        let real = tmp.path().join("real");
        fs::create_dir_all(&real).unwrap();
        for tool in ["claude", "codex"] {
            let path = real.join(tool);
            fs::write(&path, concat!(
                "#!/bin/sh\n",
                "printf 'CODEX_HOME=%s\\nCLAUDE_CONFIG_DIR=%s\\n' \"${CODEX_HOME-unset}\" \"${CLAUDE_CONFIG_DIR-unset}\"\n",
                "for arg in \"$@\"; do printf 'ARG=<%s>\\n' \"$arg\"; done\n",
                "if [ \"$1\" = login ] && [ \"$2\" = status ]; then\n",
                "  printf 'secret-key-fragment\\n' >&2\n",
                "  [ -f \"$CODEX_HOME/logged-in\" ]; exit $?\n",
                "fi\n",
                "exit \"${FAKE_EXIT-0}\"\n",
            )).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let f = Self { tmp, home, real };
        f.manager()
            .args(["init", "--mode", "off"])
            .assert()
            .success();
        f
    }

    fn configure(&self, cmd: &mut Command) {
        cmd.env_clear()
            .env("HOME", self.tmp.path())
            .env("CCDIRENV_HOME", &self.home)
            .env(
                "PATH",
                std::env::join_paths([self.home.join("bin"), self.real.clone()]).unwrap(),
            )
            .current_dir(self.tmp.path());
    }

    fn manager(&self) -> Command {
        let mut cmd = Command::cargo_bin("ccdirenv").unwrap();
        self.configure(&mut cmd);
        cmd
    }

    fn shim(&self, tool: &str) -> Command {
        let mut cmd = Command::new(self.home.join("bin").join(tool));
        self.configure(&mut cmd);
        cmd
    }

    fn marker(&self, profile: &str) {
        fs::write(self.tmp.path().join(".ccdirenv"), profile).unwrap();
    }
}

#[test]
fn init_installs_both_shims_and_is_repeatable() {
    let f = Fixture::new();
    f.manager()
        .args(["init", "--mode", "off"])
        .assert()
        .success();
    for tool in ["claude", "codex"] {
        assert!(fs::symlink_metadata(f.home.join("bin").join(tool))
            .unwrap()
            .is_symlink());
    }
}

#[test]
fn codex_and_claude_share_profile_selection_but_not_config_dirs() {
    let f = Fixture::new();
    f.marker("work");
    let codex = f.home.join("profiles/work/codex");
    f.shim("codex")
        .args(["exec", "hello world", "--", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "CODEX_HOME={}\n",
            codex.display()
        )))
        .stdout(predicate::str::contains("CLAUDE_CONFIG_DIR=unset"))
        .stdout(predicate::str::contains(
            "ARG=<hello world>\nARG=<-->\nARG=<--help>",
        ));
    assert_eq!(
        fs::metadata(&codex).unwrap().permissions().mode() & 0o777,
        0o700
    );
    f.shim("claude")
        .arg("doctor prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "CLAUDE_CONFIG_DIR={}\n",
            f.home.join("profiles/work").display()
        )))
        .stdout(predicate::str::contains("CODEX_HOME=unset"));
}

#[test]
fn codex_default_override_and_inherited_env() {
    let f = Fixture::new();
    f.shim("codex")
        .env("CODEX_HOME", "/inherited")
        .env("CLAUDE_CONFIG_DIR", "/claude-inherited")
        .assert()
        .success()
        .stdout(predicate::str::contains("profiles/default/codex"))
        .stdout(predicate::str::contains(
            "CLAUDE_CONFIG_DIR=/claude-inherited",
        ));
    f.marker("work");
    f.shim("codex")
        .env("CCDIRENV_PROFILE", "personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("profiles/personal/codex"));
}

#[test]
fn bypass_preserves_inherited_home_without_creating_profiles() {
    let f = Fixture::new();
    f.marker("work");
    for args in [vec!["--help"], vec!["--version"]] {
        f.shim("codex")
            .args(args)
            .env("CODEX_HOME", "/inherited")
            .assert()
            .success()
            .stdout(predicate::str::contains("CODEX_HOME=/inherited"));
    }
    f.shim("codex")
        .env("CCDIRENV_DISABLE", "1")
        .env("CODEX_HOME", "/inherited")
        .assert()
        .success()
        .stdout(predicate::str::contains("CODEX_HOME=/inherited"));
    assert!(!f.home.join("profiles/work").exists());
}

#[test]
fn codex_prompt_words_do_not_bypass_resolution_and_exit_code_is_preserved() {
    let f = Fixture::new();
    f.marker("work");
    for args in [
        vec!["doctor"],
        vec!["exec", "--", "--help"],
        vec!["exec", "migrate-installer"],
    ] {
        f.shim("codex")
            .args(args)
            .env("FAKE_EXIT", "42")
            .assert()
            .code(42)
            .stdout(predicate::str::contains("profiles/work/codex"));
    }
}

#[test]
fn codex_login_passes_tool_arguments_and_which_returns_codex_home() {
    let f = Fixture::new();
    f.manager()
        .args(["login", "work", "--tool", "codex", "--", "--device-auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARG=<login>\nARG=<--device-auth>"))
        .stdout(predicate::str::contains("profiles/work/codex"));
    f.marker("work");
    f.manager()
        .args(["which", "--tool", "codex"])
        .assert()
        .success()
        .stdout(format!(
            "work\t{}\n",
            f.home.join("profiles/work/codex").display()
        ));
    f.manager()
        .args(["login", "--tool", "codex"])
        .env("FAKE_EXIT", "3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("codex login` exited"));
}

#[test]
fn codex_list_uses_login_status_without_leaking_tool_output() {
    let f = Fixture::new();
    fs::create_dir_all(f.home.join("profiles/work/codex")).unwrap();
    fs::write(f.home.join("profiles/work/codex/logged-in"), "").unwrap();
    f.manager()
        .args(["list", "--tool", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(logged in)"))
        .stdout(predicate::str::contains("(not configured)"))
        .stdout(predicate::str::contains("secret-key-fragment").not())
        .stderr("");
    fs::remove_file(f.home.join("profiles/work/codex/logged-in")).unwrap();
    f.manager()
        .args(["list", "--tool", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login unavailable"));
    fs::remove_file(f.real.join("codex")).unwrap();
    f.manager()
        .args(["list", "--tool", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("codex not found"));
}

#[test]
fn codex_import_preserves_claude_and_links_and_refuses_overwrite() {
    let f = Fixture::new();
    let source = f.tmp.path().join(".codex");
    fs::create_dir_all(source.join("skills")).unwrap();
    fs::write(source.join("config.toml"), "model = 'example'\n").unwrap();
    fs::write(source.join("AGENTS.md"), "local rules\n").unwrap();
    symlink("../AGENTS.md", source.join("skills/rules")).unwrap();
    let claude = f.home.join("profiles/default/.claude.json");
    fs::write(&claude, "{}").unwrap();
    f.manager()
        .args(["import", "default", "--tool", "codex"])
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&claude).unwrap(), "{}");
    let target = f.home.join("profiles/default/codex");
    assert_eq!(
        fs::read_link(target.join("skills/rules")).unwrap(),
        PathBuf::from("../AGENTS.md")
    );
    assert!(target.join("config.toml").is_file());
    f.manager()
        .args(["import", "default", "--tool", "codex"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
    f.manager()
        .args(["import", "custom", "--tool", "codex", "--from"])
        .arg(source)
        .assert()
        .success();
}

#[test]
fn codex_doctor_works_without_claude_and_detects_shadowing() {
    let f = Fixture::new();
    fs::remove_file(f.real.join("claude")).unwrap();
    f.manager()
        .args(["doctor", "--tool", "codex"])
        .assert()
        .success();
    f.manager()
        .args(["doctor", "--tool", "codex"])
        .env(
            "PATH",
            std::env::join_paths([f.real.clone(), f.home.join("bin")]).unwrap(),
        )
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "PATH does not resolve codex to this shim",
        ));
}

#[test]
fn missing_real_codex_exits_instead_of_recursing_through_symlinks() {
    let f = Fixture::new();
    fs::remove_file(f.real.join("codex")).unwrap();
    symlink(
        assert_cmd::cargo::cargo_bin("ccdirenv"),
        f.real.join("codex"),
    )
    .unwrap();
    f.shim("codex")
        .assert()
        .code(127)
        .stderr(predicate::str::contains("codex` not found"));
}

#[test]
fn invalid_profile_and_unwritable_home_fail_before_launch() {
    let f = Fixture::new();
    f.marker("../escape");
    f.shim("codex")
        .assert()
        .code(127)
        .stderr(predicate::str::contains("invalid profile name"));
    assert!(!f.home.join("escape").exists());
    f.marker("work");
    fs::write(f.home.join("profiles/work"), "not a directory").unwrap();
    f.shim("codex").assert().code(127).stdout("");
}

#[test]
fn shared_owner_mapping_selects_codex_profile_from_git() {
    let f = Fixture::new();
    fs::create_dir_all(f.tmp.path().join(".git")).unwrap();
    fs::write(
        f.tmp.path().join(".git/config"),
        "[remote \"origin\"]\nurl = git@github.com:Acme/widget.git\n",
    )
    .unwrap();
    f.manager().args(["mode", "set", "git"]).assert().success();
    f.manager()
        .args(["owners", "map", "github.com/Acme", "work"])
        .assert()
        .success();
    f.shim("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("profiles/work/codex"));
}

#[test]
fn claude_import_can_follow_codex_import_without_overwriting_codex() {
    let f = Fixture::new();
    let codex = f.home.join("profiles/work/codex");
    fs::create_dir_all(&codex).unwrap();
    fs::write(codex.join("config.toml"), "codex config").unwrap();
    let source = f.tmp.path().join(".claude");
    fs::create_dir(&source).unwrap();
    fs::write(source.join(".claude.json"), "{}").unwrap();
    f.manager().args(["import", "work"]).assert().success();
    assert_eq!(
        fs::read_to_string(codex.join("config.toml")).unwrap(),
        "codex config"
    );
    assert!(f.home.join("profiles/work/.claude.json").is_file());
}

#[test]
fn import_rejects_symlinked_credentials_and_recursive_destination() {
    let f = Fixture::new();
    let source = f.tmp.path().join(".codex");
    fs::create_dir(&source).unwrap();
    let external = f.tmp.path().join("external-auth");
    fs::write(&external, "synthetic test data").unwrap();
    symlink(&external, source.join("auth.json")).unwrap();
    f.manager()
        .args(["import", "work", "--tool", "codex"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("symlinked credentials"));
    assert!(!f.home.join("profiles/work/codex").exists());
    f.manager()
        .args(["import", "work", "--tool", "codex", "--from"])
        .arg(f.tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not be inside"));
}

#[test]
fn shim_finds_real_codex_next_to_ccdirenv_installation() {
    let f = Fixture::new();
    let installed = f.real.join("ccdirenv");
    fs::copy(assert_cmd::cargo::cargo_bin("ccdirenv"), &installed).unwrap();
    let shim = f.home.join("bin/codex");
    fs::remove_file(&shim).unwrap();
    symlink(installed, shim).unwrap();
    f.shim("codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("profiles/default/codex"));
}

#[test]
fn adding_codex_shim_preserves_existing_discovery_settings() {
    let f = Fixture::new();
    let config = f.home.join("config.toml");
    fs::write(
        &config,
        concat!(
            "default_profile = 'work'\ndiscovery_priority = 'ghq'\n",
            "[git]\nenabled = false\nremote = 'upstream'\n",
            "[ghq]\nenabled = false\nroot = '/custom'\n",
            "[owners]\n'github.com/Acme' = 'work'\n",
        ),
    )
    .unwrap();
    let before = ccdirenv::config::Config::load(&config).unwrap();
    fs::remove_file(f.home.join("bin/codex")).unwrap();
    f.manager().args(["init", "--no-prompt"]).assert().success();
    let after = ccdirenv::config::Config::load(&config).unwrap();
    assert_eq!(
        toml::to_string(&before).unwrap(),
        toml::to_string(&after).unwrap()
    );
    assert!(f.home.join("bin/codex").exists());
}

#[test]
fn malformed_config_does_not_silently_launch_default_account() {
    let f = Fixture::new();
    fs::write(f.home.join("config.toml"), "invalid [ TOML").unwrap();
    f.shim("codex").assert().code(127).stdout("");
    f.manager()
        .args(["which", "--tool", "codex"])
        .assert()
        .failure()
        .stdout("");
}
