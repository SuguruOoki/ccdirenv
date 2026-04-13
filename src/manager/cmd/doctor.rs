use crate::{paths, shim::real};
use anyhow::Result;
use std::env;

pub fn run() -> Result<()> {
    let mut ok = true;
    let bin = paths::bin_dir()?;
    let shim = bin.join("claude");

    println!("shim path: {}", shim.display());
    if !shim.exists() && shim.symlink_metadata().is_err() {
        println!("  [FAIL] shim not installed");
        ok = false;
    } else {
        println!("  [OK] shim present");
    }

    let path = env::var("PATH").unwrap_or_default();
    if env::split_paths(&path).any(|p| p == bin) {
        println!("  [OK] PATH includes {}", bin.display());
    } else {
        println!("  [FAIL] PATH does not include {}", bin.display());
        ok = false;
    }

    match real::locate_real_claude(&bin) {
        Ok(p) => println!("  [OK] real claude at {}", p.display()),
        Err(e) => {
            println!("  [FAIL] real claude: {e}");
            ok = false;
        }
    }

    let cfg = paths::config_file()?;
    if cfg.exists() {
        println!("  [OK] config at {}", cfg.display());
    } else {
        println!("  [info] no config.toml (defaults active)");
    }

    if ok {
        println!("\nall checks passed.");
        Ok(())
    } else {
        anyhow::bail!("one or more checks failed")
    }
}
