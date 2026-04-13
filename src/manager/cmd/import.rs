use crate::paths;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn run(profile: &str) -> Result<()> {
    let source = dirs::home_dir().context("no home dir")?.join(".claude");
    if !source.is_dir() {
        bail!("source {} does not exist", source.display());
    }
    let target = paths::profile_dir(profile)?;
    if target.exists() && target.read_dir()?.next().is_some() {
        bail!(
            "profile dir {} is non-empty; refusing to overwrite",
            target.display()
        );
    }
    fs::create_dir_all(&target)?;
    copy_tree(&source, &target)?;
    println!("imported {} -> {}", source.display(), target.display());
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if meta.is_dir() {
            copy_tree(&src, &dst)?;
        } else if meta.is_file() {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}
