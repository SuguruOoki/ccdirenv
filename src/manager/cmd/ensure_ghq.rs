//! Ensure that the `ghq` binary is available on PATH, installing it if not.
//!
//! ccdirenv is built around the ghq layout (`<root>/<host>/<owner>/<repo>`),
//! so during `ccdirenv init` we make sure the user actually has ghq.
//!
//! Strategy (first available wins):
//!   1. `which ghq` — if present, do nothing.
//!   2. `brew install ghq` — if Homebrew is available.
//!   3. `go install github.com/x-motemen/ghq@latest` — if Go is available.
//!   4. Otherwise print installation guidance and continue.

use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsureMode {
    /// Run `ccdirenv init` style: print progress, run installs.
    Interactive,
    /// Quiet check (used by tests / scripts) — only report.
    #[allow(dead_code)]
    QuietCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureResult {
    AlreadyInstalled,
    InstalledViaBrew,
    InstalledViaGo,
    Skipped { reason: String },
}

pub fn ensure(mode: EnsureMode) -> Result<EnsureResult> {
    if which::which("ghq").is_ok() {
        if mode == EnsureMode::Interactive {
            println!("ghq: already installed");
        }
        return Ok(EnsureResult::AlreadyInstalled);
    }

    // Allow CI / scripted runs to bypass auto-install side effects.
    if matches!(
        std::env::var("CCDIRENV_SKIP_GHQ_AUTOINSTALL").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    ) {
        let reason = String::from("auto-install skipped via CCDIRENV_SKIP_GHQ_AUTOINSTALL");
        if mode == EnsureMode::Interactive {
            eprintln!("ghq: {reason}");
        }
        return Ok(EnsureResult::Skipped { reason });
    }

    if mode == EnsureMode::Interactive {
        println!("ghq: not found on PATH, attempting auto-install...");
    }

    if which::which("brew").is_ok() {
        if mode == EnsureMode::Interactive {
            println!("ghq: installing via Homebrew (`brew install ghq`)...");
        }
        let status = Command::new("brew").args(["install", "ghq"]).status();
        match status {
            Ok(s) if s.success() => {
                if mode == EnsureMode::Interactive {
                    println!("ghq: installed via Homebrew");
                }
                return Ok(EnsureResult::InstalledViaBrew);
            }
            Ok(s) => {
                if mode == EnsureMode::Interactive {
                    eprintln!("ghq: brew install failed (exit {}), trying next option", s);
                }
            }
            Err(e) => {
                if mode == EnsureMode::Interactive {
                    eprintln!("ghq: failed to launch brew ({e}), trying next option");
                }
            }
        }
    }

    if which::which("go").is_ok() {
        if mode == EnsureMode::Interactive {
            println!("ghq: installing via Go (`go install github.com/x-motemen/ghq@latest`)...");
        }
        let status = Command::new("go")
            .args(["install", "github.com/x-motemen/ghq@latest"])
            .status();
        match status {
            Ok(s) if s.success() => {
                if mode == EnsureMode::Interactive {
                    println!("ghq: installed via `go install`");
                    println!("ghq: ensure $(go env GOPATH)/bin is on your PATH");
                }
                return Ok(EnsureResult::InstalledViaGo);
            }
            Ok(s) => {
                if mode == EnsureMode::Interactive {
                    eprintln!("ghq: go install failed (exit {})", s);
                }
            }
            Err(e) => {
                if mode == EnsureMode::Interactive {
                    eprintln!("ghq: failed to launch go ({e})");
                }
            }
        }
    }

    let reason = String::from(
        "no installer available (need brew or go on PATH). \
         Install ghq manually: https://github.com/x-motemen/ghq#installation",
    );
    if mode == EnsureMode::Interactive {
        eprintln!("ghq: {reason}");
        eprintln!("ghq: ccdirenv will still work, but ghq-aware resolution requires ghq.");
    }
    Ok(EnsureResult::Skipped { reason })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn skip_env_short_circuits_when_ghq_missing() {
        // We can't easily reproduce "ghq missing" on a host that has it, but the
        // skip flag must be honoured before any install attempt happens. Here we
        // assert it returns Skipped when the flag is set AND ghq isn't found by
        // hijacking PATH to an empty directory.
        let tmp = tempfile::TempDir::new().unwrap();
        let original_path = std::env::var("PATH").ok();
        std::env::set_var("PATH", tmp.path());
        std::env::set_var("CCDIRENV_SKIP_GHQ_AUTOINSTALL", "1");
        let result = ensure(EnsureMode::QuietCheck).unwrap();
        std::env::remove_var("CCDIRENV_SKIP_GHQ_AUTOINSTALL");
        if let Some(p) = original_path {
            std::env::set_var("PATH", p);
        } else {
            std::env::remove_var("PATH");
        }
        assert!(matches!(result, EnsureResult::Skipped { .. }));
    }
}
