//! Detect argv shapes that can skip profile resolution.

use std::ffi::OsString;

pub fn is_fast_path(args: &[OsString]) -> bool {
    is_fast_path_for(crate::tool::Tool::Claude, args)
}

pub fn is_fast_path_for(tool: crate::tool::Tool, args: &[OsString]) -> bool {
    // Never interpret prompt text or option values as shim control flags.
    args.len() == 2
        && args[1].to_str().is_some_and(|tok| {
            matches!(tok, "--version" | "-V" | "--help" | "-h")
                || (tool == crate::tool::Tool::Claude
                    && matches!(tok, "doctor" | "migrate-installer"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(tail: &[&str]) -> Vec<OsString> {
        std::iter::once("claude")
            .chain(tail.iter().copied())
            .map(OsString::from)
            .collect()
    }

    #[test]
    fn empty_is_not_fast() {
        assert!(!is_fast_path(&mk(&[])));
    }
    #[test]
    fn version_is_fast() {
        assert!(is_fast_path(&mk(&["--version"])));
        assert!(is_fast_path(&mk(&["-V"])));
    }
    #[test]
    fn help_is_fast() {
        assert!(is_fast_path(&mk(&["--help"])));
        assert!(is_fast_path(&mk(&["-h"])));
    }
    #[test]
    fn doctor_is_fast() {
        assert!(is_fast_path(&mk(&["doctor"])));
    }
    #[test]
    fn chat_is_not_fast() {
        assert!(!is_fast_path(&mk(&["chat", "hello"])));
    }
}
