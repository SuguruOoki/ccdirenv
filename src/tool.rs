//! Tool-specific commands and isolated configuration locations.

use crate::paths;
use anyhow::Result;
use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Tool {
    #[default]
    Claude,
    Codex,
}

impl Tool {
    pub const ALL: [Self; 2] = [Self::Claude, Self::Codex];

    pub fn binary(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    pub fn config_env(self) -> &'static str {
        match self {
            Self::Claude => "CLAUDE_CONFIG_DIR",
            Self::Codex => "CODEX_HOME",
        }
    }

    pub fn config_dir(self, profile: &str) -> Result<PathBuf> {
        let root = paths::profile_dir(profile)?;
        Ok(match self {
            Self::Claude => root,
            Self::Codex => root.join("codex"),
        })
    }

    pub fn login_arg(self) -> &'static str {
        match self {
            Self::Claude => "/login",
            Self::Codex => "login",
        }
    }

    pub fn ensure_config_dir(self, profile: &str) -> Result<PathBuf> {
        use std::os::unix::fs::DirBuilderExt;
        let dir = self.config_dir(profile)?;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)?;
        Ok(dir)
    }

    pub fn source_dir_name(self) -> &'static str {
        match self {
            Self::Claude => ".claude",
            Self::Codex => ".codex",
        }
    }
}
