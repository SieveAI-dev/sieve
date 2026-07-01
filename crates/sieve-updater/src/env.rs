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
        url_override: resolve_url_override(),
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).map(|v| !v.is_empty()).unwrap_or(false)
}

/// Reads and **host-whitelists** the `SIEVE_UPDATE_URL` override.
///
/// A hijacked env var must not silently repoint the updater at an attacker
/// server (which could feed a MITM manifest or serve an older-but-signed pack
/// for a downgrade). Only URLs whose host is `sieveai.dev` or a subdomain of it
/// (covering `updates.` / `cdn.`) are honoured. A non-whitelisted (or
/// unparseable) host is **ignored** — `url_override` becomes `None` and the
/// caller falls back to the default URL — unless the operator explicitly opts
/// out via `SIEVE_UPDATE_URL_ALLOW_UNSAFE=1` (escape hatch for self-hosted
/// mirrors / testing).
///
/// Empty `SIEVE_UPDATE_URL` is treated as unset (returns `None`, no warning).
fn resolve_url_override() -> Option<String> {
    let raw = std::env::var("SIEVE_UPDATE_URL")
        .ok()
        .filter(|s| !s.is_empty())?;

    if url_host_is_trusted(&raw) || env_truthy("SIEVE_UPDATE_URL_ALLOW_UNSAFE") {
        Some(raw)
    } else {
        tracing::warn!(
            url = %raw,
            "SIEVE_UPDATE_URL host 不在白名单，已忽略；如需自定义设 SIEVE_UPDATE_URL_ALLOW_UNSAFE=1"
        );
        None
    }
}

/// Returns `true` iff `url`'s host is exactly `sieveai.dev` or a subdomain of
/// it (`*.sieveai.dev`).
///
/// Parses via [`http::Uri`] (already a dependency) to extract the authority
/// host, then matches an exact `sieveai.dev` or a `.sieveai.dev` **suffix** — a
/// suffix match (not `contains`) so spoofers like `notsieveai.dev` or
/// `sieveai.dev.evil.com` are rejected. A URL that fails to parse or carries no
/// host is treated as untrusted.
fn url_host_is_trusted(url: &str) -> bool {
    let Ok(uri) = url.parse::<http::Uri>() else {
        return false;
    };
    match uri.host() {
        Some(host) => host == "sieveai.dev" || host.ends_with(".sieveai.dev"),
        None => false,
    }
}

/// A stricter truthiness check than [`env_flag`]: only `1` or `true`
/// (case-insensitive) enable the flag. Used for the `ALLOW_UNSAFE` escape hatch
/// so a stray `SIEVE_UPDATE_URL_ALLOW_UNSAFE=0` does not accidentally open it.
fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
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

    /// 清掉本组测试关心的两个 URL 相关 env var，避免用例间串扰。
    fn clear_url_env() {
        std::env::remove_var("SIEVE_UPDATE_URL");
        std::env::remove_var("SIEVE_UPDATE_URL_ALLOW_UNSAFE");
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

    // ── F2: SIEVE_UPDATE_URL host 白名单（防上游 URL 劫持）─────────────────────────

    /// updates.sieveai.dev 子域 → 保留（覆盖默认 manifest 端点）。
    #[test]
    #[serial]
    fn url_whitelist_keeps_updates_subdomain() {
        clear_url_env();
        std::env::set_var(
            "SIEVE_UPDATE_URL",
            "https://updates.sieveai.dev/v1/manifest",
        );
        assert_eq!(
            from_env().url_override.as_deref(),
            Some("https://updates.sieveai.dev/v1/manifest")
        );
        clear_url_env();
    }

    /// cdn.sieveai.dev 子域 → 保留。
    #[test]
    #[serial]
    fn url_whitelist_keeps_cdn_subdomain() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "https://cdn.sieveai.dev/v1/manifest");
        assert_eq!(
            from_env().url_override.as_deref(),
            Some("https://cdn.sieveai.dev/v1/manifest")
        );
        clear_url_env();
    }

    /// 恰好 "sieveai.dev"（apex 域，无子域）→ 保留。
    #[test]
    #[serial]
    fn url_whitelist_keeps_apex_domain() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "https://sieveai.dev/v1/manifest");
        assert_eq!(
            from_env().url_override.as_deref(),
            Some("https://sieveai.dev/v1/manifest")
        );
        clear_url_env();
    }

    /// 非白名单 host（evil.com）→ 忽略（url_override = None，落地回退默认）。
    #[test]
    #[serial]
    fn url_whitelist_ignores_non_whitelisted_host() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "https://evil.com/v1/manifest");
        assert!(
            from_env().url_override.is_none(),
            "non-whitelisted host must be ignored"
        );
        clear_url_env();
    }

    /// 后缀伪装（notsieveai.dev）不能通过——必须用 ".sieveai.dev" 后缀 / 完整相等，
    /// 不是 contains。
    #[test]
    #[serial]
    fn url_whitelist_rejects_suffix_spoof() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "https://notsieveai.dev/v1/manifest");
        assert!(
            from_env().url_override.is_none(),
            "suffix-spoofing host must be rejected"
        );
        clear_url_env();
    }

    /// 非白名单 host + SIEVE_UPDATE_URL_ALLOW_UNSAFE=1 → 放行（自建镜像逃生阀）。
    #[test]
    #[serial]
    fn url_allow_unsafe_permits_non_whitelisted_host() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "https://evil.com/v1/manifest");
        std::env::set_var("SIEVE_UPDATE_URL_ALLOW_UNSAFE", "1");
        assert_eq!(
            from_env().url_override.as_deref(),
            Some("https://evil.com/v1/manifest")
        );
        clear_url_env();
    }

    /// 空串 SIEVE_UPDATE_URL 仍视为未设（None），即便 host 校验存在也不误报。
    #[test]
    #[serial]
    fn url_empty_string_still_unset() {
        clear_url_env();
        std::env::set_var("SIEVE_UPDATE_URL", "");
        assert!(from_env().url_override.is_none());
        clear_url_env();
    }
}
