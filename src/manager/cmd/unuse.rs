use crate::paths::MARKER_FILENAME;
use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let marker = cwd.join(MARKER_FILENAME);
    if marker.exists() {
        fs::remove_file(&marker)?;
        println!("removed {}", marker.display());
    } else {
        println!("no marker here");
    }
    Ok(())
}
