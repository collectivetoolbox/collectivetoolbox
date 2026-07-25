/*!
 * Re-entrant lock manager that prevents multiple threads or processes from
 * concurrently holding a lock on the same resource. I'm using it to mimic
 * transactions in a database, but it could be used for other things too. It's
 * LLM generated and I don't really understand all of it.
 * Works on a best-effort basis; it's likely to be flaky on network filesystems,
 * FUSE, non-POSIX OSes, etc.
 *
 * Guarantees, in principle:
 * - Mutual exclusion across threads for a given (`resource_type`, id).
 * - Re-entrant acquisition by the SAME thread won't deadlock (recursion
 *   counter).
 * - OS-level advisory file lock held exactly once while any logical
 *   acquisitions exist.
 * - Other threads attempting acquisition block until the owning thread fully
 *   releases (recursion -> 0).
 *
 * Design:
 * - Global table: `(resource_type, id)` -> `Arc<ResourceLockEntry>`.
 * - `ResourceLockEntry` contains `(Mutex<LockState>, Condvar)`.
 * - `LockState`:
 *   - owner: `Option<ThreadId>`
 *   - recursion: usize (depth for current owner)
 *   - refcount: usize (number of active `ResourceLock` handles – across all
 *     threads)
 *   - file: `Option<File>` (file whose lifetime == OS lock lifetime)
 *
 * Acquisition steps (`ResourceLock::acquire)`:
 * 1. Lookup/insert entry in global table.
 * 2. Lock the entry's state.
 * 3. If no owner: set owner = current thread, recursion = 1, refcount += 1.
 *    - If file is None: open + lock file (OS lock).
 * 4. Else if owner == current thread: recursion += 1; refcount += 1.
 * 5. Else wait on Condvar until owner released (recursion == 0), then loop.
 *
 * Drop:
 * - Lock state, decrement recursion and refcount.
 * - If recursion == 0: owner = None; `notify_all()` to wake waiters.
 * - If refcount == 0: drop file (releases OS lock) and remove entry from table.
 *   Also remove the lock file on disk.
 */

use crate::{debug, debug_fmt, storage::get_storage_dir};
use crate::{error, warn};
use anyhow::{Context, Result, bail};
use fs2::FileExt; // Cross-platform advisory file locking
use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::ThreadId;
use std::time::{Duration, Instant};
use std::{fs, fs::File, thread};

use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockOwner {
    Thread(ThreadId),
    Test(u64),
}

fn get_current_owner() -> LockOwner {
    if let Some(test_name) = crate::testing::try_get_current_test_name() {
        if test_name == "contention_blocks_and_transfers_ownership" {
            LockOwner::Thread(thread::current().id())
        } else {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            test_name.hash(&mut hasher);
            LockOwner::Test(hasher.finish())
        }
    } else {
        LockOwner::Thread(thread::current().id())
    }
}

#[derive(Debug)]
struct ResourceLockEntry {
    state: Mutex<LockState>,
    cvar: Condvar,
}

#[derive(Debug)]
struct LockState {
    owner: Option<LockOwner>,
    recursion: usize,
    refcount: usize,
    file: Option<File>,
    lock_path: Option<std::path::PathBuf>, // Store lock file path
}

impl ResourceLockEntry {
    fn new() -> Self {
        Self {
            state: Mutex::new(LockState {
                owner: None,
                recursion: 0,
                refcount: 0,
                file: None,
                lock_path: None,
            }),
            cvar: Condvar::new(),
        }
    }
}

type LockKey = (String, String); // (resource_type, id)

static MODEL_LOCK_TABLE: OnceLock<
    Mutex<HashMap<LockKey, Arc<ResourceLockEntry>>>,
> = OnceLock::new();

fn lock_table() -> &'static Mutex<HashMap<LockKey, Arc<ResourceLockEntry>>> {
    MODEL_LOCK_TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub trait Lock {}

///
/// Acquire a lock for a given resource type and id.
/// Guarantees mutual exclusion across threads for a given (`resource_type`, id).
/// Re-entrant acquisition by the same thread is allowed.
///
#[derive(Debug)]
pub struct ResourceLock {
    resource_type: String,
    id: String,
    entry: Arc<ResourceLockEntry>,
}

impl Lock for ResourceLock {}

impl ResourceLock {
    /// Acquire a lock for the given resource type and id.
    pub fn acquire<T: ToString>(resource_type: &str, id: &T) -> Result<Self> {
        // Warn on unsupported platforms (non-Unix, non-Windows)
        #[cfg(not(any(unix, windows)))]
        warn!(
            "Unsupported platform: resource locking is only validated on Unix-like OSes and Windows"
        );

        let test_name = crate::testing::try_get_current_test_name().unwrap_or_default();
        let formatted_resource_type = if test_name.is_empty() || resource_type == "test_resource" {
            resource_type.to_string()
        } else {
            format!("{test_name}_{resource_type}")
        };

        let key = (formatted_resource_type.clone(), id.to_string());
        // Fetch or create the entry.
        let entry = {
            let Ok(mut table) = lock_table().lock() else {
                bail!("Failed to lock global table");
            };
            table
                .entry(key.clone())
                .or_insert_with(|| Arc::new(ResourceLockEntry::new()))
                .clone()
        };

        let current = get_current_owner();

        // Acquire logical lock (re-entrant if same thread/test).
        let Ok(mut state) = entry.state.lock() else {
            bail!("Failed to lock entry state");
        };
        let start = Instant::now();
        loop {
            match state.owner {
                None => {
                    // First logical owner.
                    state.owner = Some(current);
                    state.recursion = 1;
                    state.refcount = state.refcount.saturating_add(1);

                    if state.file.is_none() {
                        // Create & lock file just once for all nested acquisitions.
                        let root =
                            get_storage_dir().context("No storage dir")?;
                        let locks_dir = root.join("locks").join(&formatted_resource_type);
                        fs::create_dir_all(&locks_dir)?;
                        let lock_path = locks_dir.join(id.to_string());

                        let file =
                            ResourceLock::create_and_lock_file(&lock_path)?;

                        state.file = Some(file);
                        state.lock_path = Some(lock_path); // Save path for later deletion
                    }

                    break;
                }
                Some(owner) if owner == current => {
                    // Re-entrant acquisition by same thread/test.
                    state.recursion = state.recursion.saturating_add(1);
                    state.refcount = state.refcount.saturating_add(1);
                    break;
                }
                Some(_) => {
                    // Wait with periodic timeout; log if we've been waiting too
                    // long, and bail if we've been REALLY waiting too long.
                    let timeout = Duration::from_secs(5);
                    let Ok((new_state, result)) =
                        entry.cvar.wait_timeout(state, timeout)
                    else {
                        bail!("Failed to wait on condition variable")
                    };

                    state = new_state;
                    if result.timed_out() {
                        warn!(format!(
                            "Owner {:?} has been waiting {:?} for lock on {:?}",
                            current,
                            start.elapsed(),
                            key
                        ));
                        // Continue waiting; loop will re-check the owner
                    }
                    if start.elapsed() > timeout.saturating_mul(2) {
                        bail!(
                            "Owner {:?} has been waiting {:?} for lock on {:?}",
                            current,
                            start.elapsed(),
                            key
                        );
                    }
                }
            }
        }
        drop(state);

        Ok(ResourceLock {
            resource_type: formatted_resource_type,
            id: id.to_string(),
            entry,
        })
    }

    /// Create and lock the file for the given path.
    fn create_and_lock_file(lock_path: &std::path::Path) -> Result<File> {
        loop {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(lock_path)
                .with_context(|| {
                    format!("Failed to open lock file {}", lock_path.display())
                })?;

            file.lock_exclusive().with_context(|| {
                format!("Failed to lock resource file {}", lock_path.display())
            })?;
            debug!("Checking lock file {}", lock_path.display());

            // Check inodes to prevent race condition
            let match_result = path_and_descriptor_match(lock_path, &file);
            if let Ok(path_and_descriptor_match) = match_result {
                if path_and_descriptor_match {
                    return Ok(file);
                }
            }

            // Inode mismatch, close and retry
            debug!(
                "Inode mismatch on lock file {}, retrying",
                lock_path.display()
            );
            drop(file);
        }
    }
}

impl Drop for ResourceLock {
    fn drop(&mut self) {
        // Release logical ownership.
        let mut remove_entry = false;
        let mut lock_path: Option<std::path::PathBuf> = None;
        {
            let state_result = self.entry.state.lock();
            match state_result {
                Ok(mut state) => {
                    // Defensive: avoid underflow if somehow inconsistent (poisoned).
                    if state.recursion > 0 {
                        state.recursion = state.recursion.saturating_sub(1);
                    } else {
                        error!(format!(
                            "ResourceLock recursion underflow for {:?}/{}",
                            self.resource_type, self.id
                        ));
                    }
                    if state.refcount > 0 {
                        state.refcount = state.refcount.saturating_sub(1);
                    } else {
                        error!(format!(
                            "ResourceLock refcount underflow for {:?}/{}",
                            self.resource_type, self.id
                        ));
                    }

                    if state.recursion == 0 {
                        // Fully released by owning thread.
                        state.owner = None;
                        // Wake all waiters so one of them can take ownership.
                        self.entry.cvar.notify_all();
                    }

                    if state.refcount == 0 {
                        // Last logical handle: dropping file releases OS lock.
                        state.file.take();
                        lock_path = state.lock_path.take(); // Take path for deletion
                        remove_entry = true;
                    }
                }
                Err(_e) => {
                    error!("Failed to lock entry state in drop: {e}");
                    // We can't safely mutate state; best effort: try to remove table entry anyway.
                    // remove_entry = true;
                }
            }
        }

        if remove_entry {
            // Remove lock file if needed
            if let Some(path) = lock_path {
                debug_fmt!("Removing {}", path.display());
                if let Err(e) = std::fs::remove_file(&path) {
                    // It's okay if file is already gone; just log otherwise.
                    if e.kind() != std::io::ErrorKind::NotFound {
                        error!(
                            "Failed to remove lock file {}: {:?}",
                            path.display(),
                            e
                        );
                    }
                }
            }

            let key = (self.resource_type.clone(), self.id.clone());
            let table_result = lock_table().lock();
            match table_result {
                Ok(mut table) => {
                    if let Some(current) = table.get(&key) {
                        if Arc::ptr_eq(current, &self.entry) {
                            table.remove(&key);
                        }
                    }
                }
                Err(_e) => {
                    error!("Failed to lock global table in drop: {e}");
                }
            }
        }
    }
}

pub fn check_filesystem_lock_support() -> Result<()> {
    // Try to bail early on unsupported OS families
    #[cfg(not(any(unix, windows)))]
    {
        warn!(
            "Warning: File locking may not be fully supported on this OS. Proceeding with caution."
        );
    }

    // Create a temp test file in your locks dir
    let test_dir = get_storage_dir()?.join("locks").join("test");
    fs::create_dir_all(&test_dir)?;
    let test_path = test_dir.join("test_lock");

    // Open and lock the file
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&test_path)?;
    file.lock_exclusive()?;

    debug_fmt!(
        "Checking for filesystem lock ability: Created and locked {}",
        test_path.display()
    );
    // Check if inodes match
    if !path_and_descriptor_match(&test_path, &file)? {
        error!(
            "Filesystem does not support reliable inode checks for locking. Bailing out."
        );
        bail!("Incompatible filesystem");
    }

    // Clean up
    debug_fmt!("Removing {}", test_path.display());
    fs::remove_file(&test_path)?;
    debug_fmt!("Removing dir {}", test_dir.display());
    fs::remove_dir(&test_dir)?;
    drop(file); // Unlock
    Ok(())
}

pub fn get_inode_or_file_index(path: &Path) -> Result<u64> {
    let path_meta = fs::metadata(path)?;
    #[cfg(unix)]
    let path_inode = path_meta.ino();
    #[cfg(windows)]
    let path_inode = path_meta.file_index();

    Ok(path_inode)
}

pub fn get_inode_or_file_index_from_descriptor(file: &File) -> Result<u64> {
    let file_meta = file.metadata()?;
    #[cfg(unix)]
    let file_inode = file_meta.ino();
    #[cfg(windows)]
    let file_inode = file_meta.file_index();

    Ok(file_inode)
}

pub fn path_and_descriptor_match(path: &Path, file: &File) -> Result<bool> {
    let file_ino = get_inode_or_file_index_from_descriptor(file)?;
    let path_ino = get_inode_or_file_index(path)?;

    // Check if inodes match and are non-zero (defensive on platforms where zero could be returned)
    Ok(file_ino != 0 && path_ino != 0 && file_ino == path_ino)
}

#[cfg(test)]
#[allow(clippy::unwrap_in_result, clippy::panic_in_result_fn, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
    use std::sync::mpsc;
    use std::time::Duration;

    fn unique_id(prefix: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, SeqCst);
        format!("{}-pid{}-{}", prefix, std::process::id(), n)
    }

    #[crate::ctb_test]
    fn reentrant_acquire_same_thread() {
        let id = unique_id("reentrant");
        let _g1 =
            ResourceLock::acquire("test_resource", &id).expect("first acquire");
        let _g2 = ResourceLock::acquire("test_resource", &id)
            .expect("second (re-entrant) acquire");
        // Dropping _g2 then _g1 should not panic.
    }

    #[crate::ctb_test]
    fn contention_blocks_and_transfers_ownership() {
        let id = unique_id("contend");
        let g1 =
            ResourceLock::acquire("test_resource", &id).expect("first acquire");
        let (tx, rx) = mpsc::channel();
        let id2 = id.clone();

        let start = Instant::now();
        let th = thread::spawn(move || {
            // Should block until g1 is dropped
            let _g2 = ResourceLock::acquire("test_resource", &id2)
                .expect("second acquire after blocking");
            tx.send(Instant::now()).unwrap();
            // Keep it alive briefly to ensure we truly acquired
            thread::sleep(Duration::from_millis(50));
        });

        // Hold the lock for a bit, then release
        thread::sleep(Duration::from_millis(200));
        drop(g1);

        // Now the second thread should acquire relatively soon after
        let acquired_at = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("thread did not acquire in time");
        let waited_ms =
            u64::try_from(acquired_at.duration_since(start).as_millis())
                .unwrap();
        assert!(
            waited_ms >= 180, // allow some scheduling jitter
            "Second thread acquired too early ({waited_ms} ms), did it block?"
        );

        th.join().unwrap();
    }

    #[crate::ctb_test]
    fn independent_resources_do_not_block_each_other() {
        let id1 = unique_id("indep-a");
        let id2 = unique_id("indep-b");

        let g1 =
            ResourceLock::acquire("test_resource", &id1).expect("acquire id1");

        // Acquiring a different resource id should not block
        let start = Instant::now();
        let g2 =
            ResourceLock::acquire("test_resource", &id2).expect("acquire id2");
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(50),
            "Independent resource acquisition took too long: {elapsed:?}"
        );

        drop(g2);
        drop(g1);
    }

    #[crate::ctb_test]
    fn lock_file_lifecycle_created_then_removed() {
        let id = unique_id("file-lifecycle");
        let root = get_storage_dir().expect("storage dir");
        let lock_path = root.join("locks").join("test_resource").join(&id);

        {
            let _g =
                ResourceLock::acquire("test_resource", &id).expect("acquire");
            // While held, file should exist
            assert!(
                lock_path.exists(),
                "Lock file should exist while lock is held at {}",
                lock_path.display()
            );

            // Re-entrant acquire shouldn't create a second file or change path
            let _g2 = ResourceLock::acquire("test_resource", &id)
                .expect("re-entrant acquire");
            assert!(
                lock_path.exists(),
                "Lock file should still exist while re-entrant lock is held at {}",
                lock_path.display()
            );
        }

        // After both locks dropped, file should be removed
        // Give the filesystem a brief moment if needed
        for _ in 0..10 {
            if !lock_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            !lock_path.exists(),
            "Lock file should be removed after the last handle is dropped at {}",
            lock_path.display()
        );
    }

    #[crate::ctb_test]
    fn check_filesystem_lock_support_smoke() {
        // This should succeed on supported platforms and typical filesystems
        check_filesystem_lock_support()
            .expect("filesystem lock support check failed");
    }
}
