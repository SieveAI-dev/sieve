//! Environment variable overrides for sieve-updater.

/// Runtime overrides read from environment variables.
///
/// Any non-empty value for a boolean env var is treated as `true`.
#[derive(Debug, Clone, Default)]
pub struct EnvOverrides {
    /// `SIEVE_NO_UPDATE` — skip update check entirely when `true`.
    pub no_update: bool,
    /// `SIEVE_NO_TELEMETRY` — omit `uid` from manifest query params when `true`.
    pub no_telemetry: bool,
    /// `SIEVE_UPDATE_URL` — override the manifest URL.
    pub url_override: Option<String>,
}

/// Reads [`EnvOverrides`] from the current process environment.
///
/// Called once at daemon startup; subsequent env changes are
/// not observed (intentional — avoids TOCTOU races).
pub fn from_env() -> EnvOverrides {
    EnvOverrides {
        no_update: env_flag("SIEVE_NO_UPDATE"),
        no_telemetry: env_flag("SIEVE_NO_TELEMETRY"),
        url_override: std::env::var("SIEVE_UPDATE_URL")
            .ok()
            .filter(|s| !s.is_empty()),
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).map(|v| !v.is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn no_update_set() {
        std::env::set_var("SIEVE_NO_UPDATE", "1");
        std::env::remove_var("SIEVE_NO_TELEMETRY");
        std::env::remove_var("SIEVE_UPDATE_URL");
        let o = from_env();
        assert!(o.no_update);
        assert!(!o.no_telemetry);
        assert!(o.url_override.is_none());
        std::env::remove_var("SIEVE_NO_UPDATE");
    }

    #[test]
    #[serial]
    fn no_telemetry_set() {
        std::env::remove_var("SIEVE_NO_UPDATE");
        std::env::set_var("SIEVE_NO_TELEMETRY", "true");
        std::env::remove_var("SIEVE_UPDATE_URL");
        let o = from_env();
        assert!(!o.no_update);
        assert!(o.no_telemetry);
        assert!(o.url_override.is_none());
        std::env::remove_var("SIEVE_NO_TELEMETRY");
    }

    #[test]
    #[serial]
    fn url_override_set() {
        std::env::remove_var("SIEVE_NO_UPDATE");
        std::env::remove_var("SIEVE_NO_TELEMETRY");
        std::env::set_var("SIEVE_UPDATE_URL", "https://example.com/v1/manifest");
        let o = from_env();
        assert!(!o.no_update);
        assert!(!o.no_telemetry);
        assert_eq!(
            o.url_override.as_deref(),
            Some("https://example.com/v1/manifest")
        );
        std::env::remove_var("SIEVE_UPDATE_URL");
    }

    #[test]
    #[serial]
    fn empty_value_treated_as_unset() {
        std::env::set_var("SIEVE_NO_UPDATE", "");
        std::env::set_var("SIEVE_NO_TELEMETRY", "");
        std::env::set_var("SIEVE_UPDATE_URL", "");
        let o = from_env();
        assert!(!o.no_update);
        assert!(!o.no_telemetry);
        assert!(o.url_override.is_none());
        std::env::remove_var("SIEVE_NO_UPDATE");
        std::env::remove_var("SIEVE_NO_TELEMETRY");
        std::env::remove_var("SIEVE_UPDATE_URL");
    }
}
