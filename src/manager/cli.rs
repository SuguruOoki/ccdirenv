use crate::tool::Tool;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ccdirenv",
    version,
    about = "direnv-style Claude Code and Codex CLI account switching"
)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Install the shim and print PATH setup guidance.
    Init {
        /// Preselect discovery mode. Skips the interactive prompt.
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
        /// Skip the prompt; preserve existing discovery settings or default to git.
        #[arg(long)]
        no_prompt: bool,
    },
    /// Create a profile and launch the selected tool's login command.
    Login {
        profile: Option<String>,
        #[arg(long, value_enum, default_value = "claude")]
        tool: Tool,
        /// Arguments to pass to the login command after `--`.
        #[arg(last = true)]
        args: Vec<OsString>,
    },
    /// List profiles with Claude account email or Codex login status.
    List {
        #[arg(long, value_enum, default_value = "claude")]
        tool: Tool,
    },
    /// Print which profile resolves for the current directory.
    Which {
        #[arg(long, value_enum, default_value = "claude")]
        tool: Tool,
    },
    /// Bind the current directory to a profile via a .ccdirenv marker.
    Use { profile: String },
    /// Remove the .ccdirenv marker in the current directory.
    Unuse,
    /// Open ~/.ccdirenv/config.toml in $EDITOR.
    Config,
    /// Diagnostics for the selected tool (shim, PATH order, real binary).
    Doctor {
        #[arg(long, value_enum, default_value = "claude")]
        tool: Tool,
    },
    /// Copy existing tool configuration into the given profile name.
    Import {
        profile: String,
        #[arg(long, value_enum, default_value = "claude")]
        tool: Tool,
        /// Source directory (defaults to ~/.claude or ~/.codex).
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// Manage the shared owner → profile map (used by ghq and git).
    #[command(subcommand)]
    Owners(OwnersCmd),
    /// Manage ghq path-layout detection.
    #[command(subcommand)]
    Ghq(GhqCmd),
    /// Manage git remote-based detection.
    #[command(subcommand)]
    Git(GitCmd),
    /// Set or show the discovery mode (ghq | git | both | off).
    #[command(subcommand)]
    Mode(ModeCmd),
}

#[derive(Debug, Subcommand)]
pub enum OwnersCmd {
    /// List configured owner → profile mappings.
    List,
    /// Map a `<host>/<owner>` (e.g. github.com/Acme) to a profile.
    Map { owner: String, profile: String },
    /// Remove an owner mapping.
    Unmap { owner: String },
}

#[derive(Debug, Subcommand)]
pub enum GhqCmd {
    /// List ghq state and current owner mappings.
    List,
    /// Map a `<host>/<owner>` to a profile (alias of `owners map`).
    Map { owner: String, profile: String },
    /// Remove an owner mapping (alias of `owners unmap`).
    Unmap { owner: String },
    /// Enable ghq path-layout detection.
    Enable,
    /// Disable ghq path-layout detection.
    Disable,
    /// Set or clear the ghq root override (omit value to clear).
    Root {
        /// Path to ghq root. Pass empty string to clear and fall back to $GHQ_ROOT / ~/ghq.
        path: Option<String>,
    },
    /// Install ghq if it is not already on PATH (uses brew or `go install`).
    Install,
}

#[derive(Debug, Subcommand)]
pub enum GitCmd {
    /// Show git detection state.
    Show,
    /// Enable git remote-based detection.
    Enable,
    /// Disable git remote-based detection.
    Disable,
    /// Set the git remote name to inspect (default: origin).
    Remote { name: String },
}

#[derive(Debug, Subcommand)]
pub enum ModeCmd {
    /// Show the current discovery mode.
    Show,
    /// Set the discovery mode (ghq | git | both | off).
    Set { mode: String },
}
