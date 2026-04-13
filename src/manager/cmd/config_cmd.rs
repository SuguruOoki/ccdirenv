use crate::paths;
use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    let path = paths::config_file()?;
    if !path.exists() {
        crate::config::Config::default().save(&path)?;
    }
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    Command::new(editor).arg(&path).status()?;
    Ok(())
}
