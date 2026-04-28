use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ccdirenv",
    version,
    about = "direnv-style Claude Code account switching"
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
        /// Skip the interactive prompt; use the default mode (git).
        #[arg(long)]
        no_prompt: bool,
    },
    /// Create a profile and run `claude /login` inside it.
    Login { profile: Option<String> },
    /// List all profiles with their active account email.
    List,
    /// Print which profile resolves for the current directory.
    Which,
    /// Bind the current directory to a profile via a .ccdirenv marker.
    Use { profile: String },
    /// Remove the .ccdirenv marker in the current directory.
    Unuse,
    /// Open ~/.ccdirenv/config.toml in $EDITOR.
    Config,
    /// Diagnostics (PATH order, real claude resolvability, permissions).
    Doctor,
    /// Copy existing ~/.claude/ into the given profile name.
    Import { profile: String },
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
