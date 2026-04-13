use crate::{config::Config, paths, profile::resolve};
use anyhow::Result;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg = Config::load(&paths::config_file()?).unwrap_or_default();
    let profile = resolve::resolve(&cwd, &cfg);
    let dir = paths::profile_dir(&profile)?;
    println!("{profile}\t{}", dir.display());
    Ok(())
}
