#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use include_dir::{Dir, DirEntry, include_dir};
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "linux")]
use tempfile::TempDir;
#[cfg(target_os = "linux")]
use fs2::FileExt;

#[cfg(target_os = "linux")]
static XKB_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../../built/minimal-assets/x11/xkb");

#[cfg(target_os = "linux")]
static X11_NLS_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../../built/minimal-assets/x11/nls");

#[cfg(target_os = "linux")]
static XKB_TEMP_DIR: OnceCell<(TempDir, std::fs::File)> = OnceCell::new();

#[cfg(target_os = "linux")]
static X11_NLS_TEMP_DIR: OnceCell<(TempDir, std::fs::File)> = OnceCell::new();

#[cfg(target_os = "linux")]
static CLEANUP_ONCE: std::sync::Once = std::sync::Once::new();

#[cfg(target_os = "linux")]
fn clean_orphaned_temp_dirs() -> Result<()> {
    use std::fs;

    let temp_dir = std::env::temp_dir();
    if !temp_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(&temp_dir).context("read temp dir")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if file_name.starts_with("ctoolbox-xkb-") || file_name.starts_with("ctoolbox-x11-nls-") {
            let lock_file_path = path.join(".lock");
            if lock_file_path.is_file() {
                if let Ok(file) = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&lock_file_path)
                {
                    if file.try_lock_exclusive().is_ok() {
                        drop(file);
                        if let Err(e) = fs::remove_dir_all(&path) {
                            warn_fmt!(
                                "Failed to clean up orphaned temp dir {}: {:?}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            } else {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > std::time::Duration::from_secs(300) {
                                if let Err(e) = fs::remove_dir_all(&path) {
                                    warn_fmt!(
                                        "Failed to clean up old temp dir {}: {:?}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn maybe_cleanup() {
    CLEANUP_ONCE.call_once(|| {
        if let Err(e) = clean_orphaned_temp_dirs() {
            warn_fmt!("Failed to clean up orphaned temp dirs: {:?}", e);
        }
    });
}

/// Ensure libxkbcommon can resolve XKB data without relying on system
/// installation paths.
///
/// On Linux, this extracts embedded `xkeyboard-config` data to a temporary
/// directory and sets the `XKB_CONFIG_ROOT` environment variable to point at
/// it. The temp directory is kept alive for the lifetime of the process.
///
/// If `XKB_CONFIG_ROOT` is already set to an existing directory, this function
/// does nothing.
///
/// # Errors
///
/// Returns an error if extraction fails.
#[cfg(target_os = "linux")]
pub fn ensure_xkb_config_root() -> Result<()> {
    if let Some(existing) = std::env::var_os("XKB_CONFIG_ROOT") {
        let existing = PathBuf::from(existing);
        if existing.is_dir() {
            return Ok(());
        }
    }

    maybe_cleanup();

    let temp = XKB_TEMP_DIR.get_or_try_init::<_, anyhow::Error>(|| {
        let dir = tempfile::Builder::new()
            .prefix("ctoolbox-xkb-")
            .tempdir()
            .context("create temp dir for XKB data")?;

        let lock_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(dir.path().join(".lock"))
            .context("create lock file for XKB temp dir")?;

        lock_file
            .try_lock_exclusive()
            .context("lock XKB temp dir")?;

        Ok((dir, lock_file))
    })?;

    extract_embedded_xkb(temp.0.path())?;
    #[allow(unsafe_code, reason = "XKB_CONFIG_ROOT environment variable must be set globally")]
    // SAFETY: setting environment variable globally is required for xkb config initialization
    unsafe {
        // Safety: unknown, but seems fairly unlikely to cause issues in practice.
        // FIXME: looks like they're working on a safe API for this in glibc:
        // https://doc.rust-lang.org/std/env/fn.set_var.html
        // https://sourceware.org/bugzilla/show_bug.cgi?id=15607
        std::env::set_var("XKB_CONFIG_ROOT", temp.0.path());
    }

    Ok(())
}

/// Ensure bundled X11 locale and Compose data is available for runtime
/// consumers such as the Rust XKB compose loader.
#[cfg(target_os = "linux")]
pub fn ensure_x11_locale_root() -> Result<()> {
    if let Some(existing) = std::env::var_os("XLOCALEDIR") {
        let existing = PathBuf::from(existing);
        if existing.is_dir() {
            return Ok(());
        }
    }

    maybe_cleanup();

    let temp = X11_NLS_TEMP_DIR.get_or_try_init::<_, anyhow::Error>(|| {
        let dir = tempfile::Builder::new()
            .prefix("ctoolbox-x11-nls-")
            .tempdir()
            .context("create temp dir for X11 locale data")?;

        let lock_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(dir.path().join(".lock"))
            .context("create lock file for X11 locale temp dir")?;

        lock_file
            .try_lock_exclusive()
            .context("lock X11 locale temp dir")?;

        Ok((dir, lock_file))
    })?;

    extract_embedded_x11_nls(temp.0.path())?;
    #[allow(unsafe_code, reason = "XLOCALEDIR environment variable must be set globally")]
    // SAFETY: setting environment variable globally is required for locale directory configuration
    unsafe {
        std::env::set_var("XLOCALEDIR", temp.0.path());
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn extract_embedded_xkb(dst_root: &Path) -> Result<()> {
    // Fast path: if it already looks extracted.
    if dst_root.join("rules").is_dir() && dst_root.join("symbols").is_dir() {
        return Ok(());
    }

    let entries = XKB_DATA_DIR
        .find("**/*")
        .context("list embedded XKB files")?;

    for entry in entries {
        match entry {
            DirEntry::Dir(dir) => {
                let out_dir = dst_root.join(dir.path());
                std::fs::create_dir_all(&out_dir).with_context(|| {
                    format!("create embedded XKB dir {}", out_dir.display())
                })?;
            }
            DirEntry::File(file) => {
                let out_path = dst_root.join(file.path());
                let Some(parent) = out_path.parent() else {
                    bail!("invalid embedded XKB path {}", out_path.display());
                };
                std::fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "create embedded XKB parent dir {}",
                        parent.display()
                    )
                })?;
                std::fs::write(&out_path, file.contents()).with_context(
                    || {
                        format!(
                            "write embedded XKB file {}",
                            out_path.display()
                        )
                    },
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn extract_embedded_x11_nls(dst_root: &Path) -> Result<()> {
    if dst_root.join("compose.dir").is_file()
        && dst_root.join("locale.alias").is_file()
    {
        return Ok(());
    }

    let entries = X11_NLS_DIR
        .find("**/*")
        .context("list embedded X11 locale files")?;

    for entry in entries {
        match entry {
            DirEntry::Dir(dir) => {
                let out_dir = dst_root.join(dir.path());
                std::fs::create_dir_all(&out_dir).with_context(|| {
                    format!(
                        "create embedded X11 locale dir {}",
                        out_dir.display()
                    )
                })?;
            }
            DirEntry::File(file) => {
                let out_path = dst_root.join(file.path());
                let Some(parent) = out_path.parent() else {
                    bail!(
                        "invalid embedded X11 locale path {}",
                        out_path.display()
                    );
                };
                std::fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "create embedded X11 locale parent dir {}",
                        parent.display()
                    )
                })?;
                std::fs::write(&out_path, file.contents()).with_context(
                    || {
                        format!(
                            "write embedded X11 locale file {}",
                            out_path.display()
                        )
                    },
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn ensure_xkb_config_root() -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn ensure_x11_locale_root() -> Result<()> {
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::fs;

    #[crate::ctb_test]
    fn test_temp_dir_cleanup() {
        use fs2::FileExt;

        let temp_dir = std::env::temp_dir();

        // 1. Create a dummy active temp directory (locked)
        let active_dir = tempfile::Builder::new()
            .prefix("ctoolbox-xkb-active-")
            .tempdir()
            .unwrap();
        let active_lock_path = active_dir.path().join(".lock");
        let active_lock_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&active_lock_path)
            .unwrap();
        active_lock_file.try_lock_exclusive().unwrap();

        // 2. Create a dummy orphaned temp directory (unlocked)
        let orphaned_dir_path = temp_dir.join("ctoolbox-xkb-orphaned-test");
        if orphaned_dir_path.exists() {
            let _ = fs::remove_dir_all(&orphaned_dir_path);
        }
        fs::create_dir_all(&orphaned_dir_path).unwrap();
        let orphaned_lock_path = orphaned_dir_path.join(".lock");
        let orphaned_lock_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&orphaned_lock_path)
            .unwrap();
        // Do not lock it (mocking a dead process where lock is released)
        drop(orphaned_lock_file);

        // Run the cleanup function
        clean_orphaned_temp_dirs().unwrap();

        // The active directory should still exist
        assert!(active_dir.path().exists(), "Active locked directory should not be deleted");

        // The orphaned directory should be deleted
        assert!(!orphaned_dir_path.exists(), "Orphaned unlocked directory should be deleted");

        // Clean up the active lock file so it doesn't leak
        drop(active_lock_file);
    }
}
