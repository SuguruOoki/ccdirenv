use crate::paths;
use anyhow::Result;
use serde::Deserialize;
use std::fs;

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

pub fn run() -> Result<()> {
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

    for name in names {
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
