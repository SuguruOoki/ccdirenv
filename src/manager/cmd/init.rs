use crate::config::Config;
use crate::manager::cmd::ensure_ghq::{ensure, EnsureMode};
use crate::manager::cmd::mode::Mode;
use crate::paths;
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
#[cfg(unix)]
use std::os::unix::fs as unix_fs;

pub struct InitOptions {
    /// Pre-selected mode. When None and stdin is a TTY, prompt the user.
    pub mode: Option<Mode>,
    /// Skip the interactive prompt entirely (use the default mode).
    pub no_prompt: bool,
}

pub fn run(opts: InitOptions) -> Result<()> {
    let root = paths::root()?;
    let bin = paths::bin_dir()?;
    let default_profile = paths::profile_dir("default")?;

    fs::create_dir_all(&bin).context("creating bin dir")?;
    fs::create_dir_all(&default_profile).context("creating default profile dir")?;

    let cfg_path = paths::config_file()?;
    let mut cfg = if cfg_path.exists() {
        Config::load(&cfg_path)?
    } else {
        Config::default()
    };

    let shim_link = bin.join("claude");
    let self_path = std::env::current_exe().context("resolving current binary")?;
    if shim_link.exists() || shim_link.symlink_metadata().is_ok() {
        fs::remove_file(&shim_link).ok();
    }
    #[cfg(unix)]
    unix_fs::symlink(&self_path, &shim_link).context("creating shim symlink")?;

    println!("ccdirenv initialized at {}", root.display());
    println!();

    let mode = resolve_mode(&opts);
    println!("discovery mode: {}", mode.name());
    mode.apply(&mut cfg);
    cfg.save(&cfg_path)?;

    // Auto-install ghq if any mode that uses it.
    if matches!(mode, Mode::Ghq | Mode::Both) {
        let _ = ensure(EnsureMode::Interactive);
    }

    println!();
    println!("Add to your shell rc:");
    println!("    export PATH=\"{}:$PATH\"", bin.display());
    println!();
    println!("Map owners to profiles, e.g.:");
    println!("    ccdirenv owners map github.com/<your-handle> default");
    println!("    ccdirenv owners map github.com/<work-org>    work");
    if matches!(mode, Mode::Git) {
        println!();
        println!("git mode is active — owner is detected from each repo's `origin` remote.");
        println!("Worktrees and submodules are supported automatically.");
    }
    Ok(())
}

fn resolve_mode(opts: &InitOptions) -> Mode {
    if let Some(m) = opts.mode {
        return m;
    }
    if opts.no_prompt {
        return Mode::Git;
    }
    if !io::stdin().is_terminal() {
        return Mode::Git;
    }
    prompt_interactive().unwrap_or(Mode::Git)
}

fn prompt_interactive() -> Option<Mode> {
    println!("How do you organize Git repositories?");
    println!("  1) ghq   — uses ~/ghq/<host>/<owner>/<repo> layout (ghq priority + git fallback)");
    println!("  2) git   — any repository with a git remote (default)");
    println!("  3) both  — both methods enabled (git priority, ghq fallback)");
    println!("  4) off   — no repo-aware detection");
    print!("Choose [1-4, default 2]: ");
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok()?;
    let trimmed = line.trim();
    let pick = match trimmed {
        "" | "2" | "git" => Mode::Git,
        "1" | "ghq" => Mode::Ghq,
        "3" | "both" => Mode::Both,
        "4" | "off" => Mode::Off,
        other => {
            eprintln!("unknown choice '{other}', defaulting to git");
            Mode::Git
        }
    };
    Some(pick)
}
