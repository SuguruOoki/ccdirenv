use crate::tool::Tool;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(profile: &str, tool: Tool, from: Option<PathBuf>) -> Result<()> {
    let source = match from {
        Some(path) => path,
        None => dirs::home_dir()
            .context("no home dir")?
            .join(tool.source_dir_name()),
    };
    if !source.is_dir() {
        bail!("source {} does not exist", source.display());
    }
    let target = tool.config_dir(profile)?;
    if target.exists() {
        for entry in target.read_dir()? {
            let entry = entry?;
            if tool == Tool::Claude && entry.file_name() == "codex" {
                continue;
            }
            bail!(
                "profile dir {} is non-empty; refusing to overwrite",
                target.display()
            );
        }
    }
    if tool == Tool::Claude && source.join("codex").symlink_metadata().is_ok() {
        bail!(
            "source contains reserved codex directory; import Codex separately with --tool codex"
        );
    }
    for filename in ["auth.json", ".credentials.json", ".claude.json"] {
        if source
            .join(filename)
            .symlink_metadata()
            .is_ok_and(|meta| meta.is_symlink())
        {
            bail!("refusing to import symlinked credentials ({filename}); log in to this profile instead");
        }
    }
    tool.ensure_config_dir(profile)?;
    if target.canonicalize()?.starts_with(source.canonicalize()?) {
        bail!("import target must not be inside the source directory");
    }
    copy_tree(&source, &target)?;
    println!("imported {} -> {}", source.display(), target.display());
    if tool == Tool::Codex {
        println!("OS keyring credentials are not copied. Check with `ccdirenv list --tool codex`; log in again if needed.");
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if meta.file_type().is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(&src)?, &dst)?;
        } else if meta.is_dir() {
            copy_tree(&src, &dst)?;
        } else if meta.is_file() {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}
