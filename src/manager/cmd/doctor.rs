use crate::{paths, shim::real, tool::Tool};
use anyhow::Result;
use std::env;

pub fn run(tool: Tool) -> Result<()> {
    let mut ok = true;
    let bin = paths::bin_dir()?;
    let binary = tool.binary();
    let shim = bin.join(binary);

    println!("shim path: {}", shim.display());
    if !shim.exists() {
        println!("  [FAIL] shim missing or broken");
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

    match real::locate_real(tool, &bin) {
        Ok(p) => println!("  [OK] real {binary} at {}", p.display()),
        Err(e) => {
            println!("  [FAIL] real {binary}: {e}");
            ok = false;
        }
    }

    let expected = std::env::current_exe()?.canonicalize()?;
    match which::which(binary).and_then(|p| {
        p.canonicalize()
            .map_err(|_| which::Error::CannotCanonicalize)
    }) {
        Ok(p) if p == expected => println!("  [OK] PATH resolves {binary} to this shim"),
        _ => {
            println!("  [FAIL] PATH does not resolve {binary} to this shim; run ccdirenv init and put {} first", bin.display());
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
