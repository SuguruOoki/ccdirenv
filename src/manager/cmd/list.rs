use crate::{paths, shim::real, tool::Tool};
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
struct ClaudeJson {
    #[serde(rename = "oauthAccount")]
    oauth_account: Option<Oauth>,
}

#[derive(Debug, Deserialize)]
struct Oauth {
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

pub fn run(tool: Tool) -> Result<()> {
    let dir = paths::profiles_dir()?;
    if !dir.is_dir() {
        println!("(no profiles — run `ccdirenv init` first)");
        return Ok(());
    }

    let mut names: Vec<_> = fs::read_dir(&dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();

    let codex = if tool == Tool::Codex {
        real::locate_real(tool, &paths::bin_dir()?).ok()
    } else {
        None
    };
    for name in names {
        if tool == Tool::Codex {
            let config_dir = tool.config_dir(&name)?;
            let status = if !config_dir.is_dir() {
                "(not configured)"
            } else if let Some(binary) = &codex {
                // Ask Codex so file and OS keyring storage both work. Do not
                // display its output, which can contain API key fragments.
                match Command::new(binary)
                    .args(["login", "status"])
                    .env(tool.config_env(), &config_dir)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                {
                    Ok(s) if s.success() => "(logged in)",
                    Ok(_) => "(login unavailable; run codex login status for details)",
                    Err(_) => "(status unavailable)",
                }
            } else {
                "(status unavailable: codex not found)"
            };
            println!("{name:20}{status}");
            continue;
        }
        let path = dir.join(&name).join(".claude.json");
        let email = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<ClaudeJson>(&s).ok())
            .and_then(|j| j.oauth_account.and_then(|a| a.email_address))
            .unwrap_or_else(|| "(not logged in)".to_string());
        println!("{name:20}{email}");
    }
    Ok(())
}
