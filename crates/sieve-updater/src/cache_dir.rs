//! Platform-specific cache directory resolution (ADR-030 §5.1).

use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::UpdaterError;

/// Env var that overrides the platform-default cache directory.
///
/// When set to a non-empty value, its value is used verbatim as the cache
/// directory (the equivalent of `$HOME/Library/Caches/sieve` on macOS),
/// bypassing platform detection entirely. Primarily for hermetic test
/// isolation (so the updater/install-id never touches the real user cache) and
/// custom/enterprise deployments. Signature verification and all other
/// security behaviour are unaffected — this only controls *where* files land.
pub const CACHE_DIR_ENV: &str = "SIEVE_CACHE_DIR";

/// Returns the sieve updater cache directory for the current platform.
///
/// ADR-030 §5.1: directory is created (mode 0700 on Unix) if it does not
/// already exist.
///
/// | Source | Path |
/// |--------|------|
/// | `SIEVE_CACHE_DIR` (any platform, if non-empty) | the value verbatim |
/// | macOS    | `$HOME/Library/Caches/sieve/` |
/// | Linux    | `$XDG_CACHE_HOME/sieve/` or `$HOME/.cache/sieve/` |
/// | Windows  | `%LOCALAPPDATA%\sieve\Cache\` |
/// | other    | returns `UpdaterError::UnsupportedPlatform` |
pub fn cache_dir() -> Result<PathBuf, UpdaterError> {
    let dir = resolve_cache_dir_with_override(std::env::var_os(CACHE_DIR_ENV))?;
    create_dir_secure(&dir)?;
    Ok(dir)
}

/// Resolves the cache dir given an explicit `SIEVE_CACHE_DIR` value.
///
/// Split out from [`cache_dir`] so the override precedence can be unit-tested
/// without mutating the process-global environment (which races under cargo's
/// parallel test runner).
fn resolve_cache_dir_with_override(
    override_val: Option<OsString>,
) -> Result<PathBuf, UpdaterError> {
    if let Some(v) = override_val {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    resolve_cache_dir()
}

#[cfg(target_os = "macos")]
fn resolve_cache_dir() -> Result<PathBuf, UpdaterError> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        UpdaterError::UnsupportedPlatform("HOME env var not set on macOS".to_owned())
    })?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Caches")
        .join("sieve"))
}

#[cfg(target_os = "linux")]
fn resolve_cache_dir() -> Result<PathBuf, UpdaterError> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("sieve"));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| {
        UpdaterError::UnsupportedPlatform("HOME env var not set on Linux".to_owned())
    })?;
    Ok(PathBuf::from(home).join(".cache").join("sieve"))
}

#[cfg(target_os = "windows")]
fn resolve_cache_dir() -> Result<PathBuf, UpdaterError> {
    let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        UpdaterError::UnsupportedPlatform("LOCALAPPDATA env var not set on Windows".to_owned())
    })?;
    Ok(PathBuf::from(local_app_data).join("sieve").join("Cache"))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn resolve_cache_dir() -> Result<PathBuf, UpdaterError> {
    Err(UpdaterError::UnsupportedPlatform(format!(
        "platform {} is not supported",
        std::env::consts::OS
    )))
}

/// Creates the directory with mode 0700 on Unix, or default ACL on Windows.
fn create_dir_secure(dir: &std::path::Path) -> Result<(), UpdaterError> {
    if dir.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Test helper: returns cache dir rooted at `base` instead of the system default.
///
/// Used in unit tests to avoid touching the real cache directory.
#[cfg(test)]
pub(crate) fn cache_dir_for_test(base: &std::path::Path) -> Result<PathBuf, UpdaterError> {
    let dir = base.join("sieve");
    create_dir_secure(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_path_contains_library_caches_sieve() {
        // Skip if HOME is not set (unlikely in normal env, but guard anyway).
        if std::env::var_os("HOME").is_none() {
            return;
        }
        let dir = cache_dir().expect("cache_dir must succeed on macOS with HOME set");
        assert!(
            dir.to_string_lossy().contains("Library/Caches/sieve"),
            "macOS cache dir should contain Library/Caches/sieve, got: {dir:?}"
        );
    }

    #[test]
    fn test_helper_creates_sieve_subdir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = cache_dir_for_test(tmp.path()).expect("helper must succeed");
        assert!(dir.exists());
        assert_eq!(dir.file_name().unwrap(), "sieve");
    }

    #[test]
    fn override_is_used_verbatim() {
        let want = PathBuf::from("/tmp/sieve-cache-override-xyz");
        let got = resolve_cache_dir_with_override(Some(want.clone().into_os_string()))
            .expect("override must resolve");
        assert_eq!(got, want);
    }

    #[test]
    fn empty_override_falls_through_to_platform_default() {
        // Empty value must be ignored (treated as unset), not used as a path.
        let got = resolve_cache_dir_with_override(Some(OsString::new()));
        // On supported platforms this resolves to the platform default; on
        // unsupported ones it errors. Either way it must NOT be the empty path.
        if let Ok(dir) = got {
            assert_ne!(dir.as_os_str(), OsString::new());
            assert!(dir.to_string_lossy().contains("sieve"));
        }
    }

    #[test]
    fn none_override_falls_through_to_platform_default() {
        // Mirrors the empty case: no override → platform resolution path.
        let got = resolve_cache_dir_with_override(None);
        if let Ok(dir) = got {
            assert!(dir.to_string_lossy().contains("sieve"));
        }
    }
}
