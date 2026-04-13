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
    Init,
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
}
