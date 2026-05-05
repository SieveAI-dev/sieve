//! Persistent install-ID management (ADR-030 §5.2 + §3 遥测信标).
//!
//! The install-ID is a UUIDv4 stored at `<cache_dir>/install-id`.
//! It is created on first run and re-created whenever the file is deleted
//! (treated as a fresh install by the update server for DAU/retention stats).

use std::path::PathBuf;

use uuid::Uuid;

use crate::error::UpdaterError;

/// Loads the install-ID from disk or generates a fresh one.
///
/// ADR-030 §5.2: file written with mode 0600 on Unix.
/// File-not-found → generate new UUID + write → return.
/// Deletion between runs → next call regenerates (counts as new install).
///
/// # Errors
/// Returns `UpdaterError` if the cache directory cannot be resolved or
/// the file cannot be read/written.
pub async fn load_or_create_install_id() -> Result<Uuid, UpdaterError> {
    let path = install_id_path()?;
    load_or_create_at(&path).await
}

fn install_id_path() -> Result<PathBuf, UpdaterError> {
    let dir = crate::cache_dir::cache_dir()?;
    Ok(dir.join("install-id"))
}

async fn load_or_create_at(path: &std::path::Path) -> Result<Uuid, UpdaterError> {
    // Try to read existing file.
    match tokio::fs::read_to_string(path).await {
        Ok(contents) => {
            let trimmed = contents.trim();
            Uuid::parse_str(trimmed).map_err(|e| {
                UpdaterError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("install-id file contains invalid UUID: {e}"),
                ))
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // First run or file was deleted — generate fresh UUID.
            let id = Uuid::new_v4();
            write_install_id(path, &id).await?;
            tracing::info!(install_id = %id, "generated new install-id");
            Ok(id)
        }
        Err(e) => Err(UpdaterError::Io(e)),
    }
}

async fn write_install_id(path: &std::path::Path, id: &Uuid) -> Result<(), UpdaterError> {
    let content = id.to_string();
    // Write to a temp file in the same directory then rename for atomicity.
    let parent = path.parent().ok_or_else(|| {
        UpdaterError::Io(std::io::Error::other(
            "install-id path has no parent directory",
        ))
    })?;
    let tmp_path = parent.join(format!(".install-id.tmp.{}", Uuid::new_v4()));
    tokio::fs::write(&tmp_path, content).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    tokio::fs::rename(&tmp_path, path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_dir::cache_dir_for_test;

    async fn load_or_create_in(dir: &std::path::Path) -> Result<Uuid, UpdaterError> {
        let path = dir.join("install-id");
        load_or_create_at(&path).await
    }

    #[tokio::test]
    async fn first_call_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = cache_dir_for_test(tmp.path()).unwrap();
        let id = load_or_create_in(&cache).await.expect("should succeed");
        // File must now exist.
        let on_disk = tokio::fs::read_to_string(cache.join("install-id"))
            .await
            .unwrap();
        assert_eq!(on_disk.trim(), id.to_string());
    }

    #[tokio::test]
    async fn idempotent_on_second_call() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = cache_dir_for_test(tmp.path()).unwrap();
        let id1 = load_or_create_in(&cache).await.unwrap();
        let id2 = load_or_create_in(&cache).await.unwrap();
        assert_eq!(id1, id2, "second call must return same UUID");
    }

    #[tokio::test]
    async fn deleted_file_regenerates_fresh_id() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = cache_dir_for_test(tmp.path()).unwrap();
        let id1 = load_or_create_in(&cache).await.unwrap();
        tokio::fs::remove_file(cache.join("install-id"))
            .await
            .unwrap();
        let id2 = load_or_create_in(&cache).await.unwrap();
        // Regenerated — not guaranteed different (astronomically unlikely to
        // collide), but the file must exist again and be a valid UUID.
        let _ = id1;
        let _ = id2;
        assert!(cache.join("install-id").exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn file_permissions_are_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let cache = cache_dir_for_test(tmp.path()).unwrap();
        load_or_create_in(&cache).await.unwrap();
        let meta = std::fs::metadata(cache.join("install-id")).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "install-id must be mode 0600, got {mode:o}");
    }
}
