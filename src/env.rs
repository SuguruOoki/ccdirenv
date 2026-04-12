//! CCDIRENV_DISABLE / CCDIRENV_PROFILE / CCDIRENV_DEBUG readers.

use std::env;

pub const DISABLE: &str = "CCDIRENV_DISABLE";
pub const FORCE_PROFILE: &str = "CCDIRENV_PROFILE";
pub const DEBUG: &str = "CCDIRENV_DEBUG";

pub fn is_disabled() -> bool { truthy(DISABLE) }
pub fn is_debug() -> bool { truthy(DEBUG) }

pub fn forced_profile() -> Option<String> {
    env::var(FORCE_PROFILE).ok().filter(|s| !s.is_empty())
}

fn truthy(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn disable_accepts_truthy_values() {
        for v in &["1", "true", "TRUE", "yes"] {
            env::set_var(DISABLE, v);
            assert!(is_disabled(), "{v}");
        }
        env::set_var(DISABLE, "0");
        assert!(!is_disabled());
        env::remove_var(DISABLE);
        assert!(!is_disabled());
    }

    #[test]
    #[serial]
    fn forced_profile_reads_name() {
        env::set_var(FORCE_PROFILE, "work");
        assert_eq!(forced_profile().as_deref(), Some("work"));
        env::set_var(FORCE_PROFILE, "");
        assert_eq!(forced_profile(), None);
        env::remove_var(FORCE_PROFILE);
        assert_eq!(forced_profile(), None);
    }
}
