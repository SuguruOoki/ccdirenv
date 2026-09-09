use crate::{config::Config, paths, profile::resolve, tool::Tool};
use anyhow::Result;

pub fn run(tool: Tool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg = Config::load(&paths::config_file()?)?;
    let profile = resolve::resolve(&cwd, &cfg);
    let dir = tool.config_dir(&profile)?;
    println!("{profile}\t{}", dir.display());
    Ok(())
}
