// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Synchronous bridges over asynchronous futures, byte streams, and pools.
//!
//! Isolates Tokio runtime management, cooperative-scheduling boundaries,
//! bounded async-to-sync stream backpressure, and lazy keyed resource pooling
//! so callers can write straightforward synchronous code without worrying
//! about whether an outer async runtime is active on the calling thread.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
#[allow(
    unused_imports,
    reason = "Disambiguates std::panic inside tokio::select! expansion when crate::utilities::* is in scope"
)]
use std::panic;

use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

const DEFAULT_STREAM_BUFFER_CHUNKS: usize = 8;

/// Run a non-`'static` async future to completion from synchronous code.
///
/// Hopefully makes futures simpler to work with.
///
/// Safe to call whether or not a Tokio runtime is active on the current thread:
/// when a runtime is already entered, runs `fut` on a scoped OS thread with its
/// own `current_thread` Tokio runtime so it neither panics nor consumes the
/// caller's cooperative scheduling budget.
pub fn unasync<R>(fut: impl Future<Output = R> + Send) -> Result<R>
where
    R: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        let test_name = crate::testing::try_get_current_test_name();
        let test_storage_dir = crate::testing::try_get_test_storage_dir();

        let result = std::thread::scope(|s| {
            let thread_handle = s.spawn(move || {
                let _guard = crate::testing::push_current_test_name(test_name);
                if let Some(dir) = test_storage_dir {
                    crate::testing::TEST_STORAGE_DIR_SYNC
                        .with(|c| *c.borrow_mut() = Some(dir));
                }
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("failed to create Tokio runtime")?;
                Ok::<R, anyhow::Error>(rt.block_on(fut))
            });
            thread_handle.join()
        });

        let res = result
            .map_err(|e| anyhow::anyhow!("unasync thread panicked: {e:?}"))??;
        Ok(res)
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create Tokio runtime")?;
        Ok(rt.block_on(fut))
    }
}

type BoxTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

struct ScheduledTask {
    test_name: Option<String>,
    test_storage_dir: Option<PathBuf>,
    future: BoxTask,
}

/// Persistent background async executor isolated from the calling thread.
///
/// Communicates with synchronous callers strictly over `std::sync::mpsc` so
/// blocking waits never panic inside `#[tokio::main]` and never touch Tokio's
/// thread-local cooperative scheduling budget.
#[derive(Clone)]
pub struct UnasyncExecutor {
    tx: tokio::sync::mpsc::UnboundedSender<ScheduledTask>,
}

impl std::fmt::Debug for UnasyncExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnasyncExecutor").finish_non_exhaustive()
    }
}

impl UnasyncExecutor {
    /// Spawns a new dedicated background OS thread driving an async runtime.
    pub fn new(thread_name: &str) -> Result<Self> {
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<ScheduledTask>();
        let (startup_tx, startup_rx) = std::sync::mpsc::channel::<Result<()>>();

        std::thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("Failed to create background unasync runtime")
                {
                    Ok(rt) => rt,
                    Err(err) => {
                        let _ = startup_tx.send(Err(err));
                        return;
                    }
                };

                if startup_tx.send(Ok(())).is_err() {
                    return;
                }

                rt.block_on(async move {
                    while let Some(task) = rx.recv().await {
                        tokio::spawn(
                            crate::testing::scope_optional_test_context(
                                task.test_name,
                                task.test_storage_dir,
                                task.future,
                            ),
                        );
                    }
                });
            })
            .context("Failed to spawn background unasync worker thread")?;

        match startup_rx.recv() {
            Ok(Ok(())) => Ok(Self { tx }),
            Ok(Err(err)) => Err(err),
            Err(_) => {
                bail!("Background unasync worker thread panicked on startup")
            }
        }
    }

    /// Spawns a fire-and-forget `'static` async task on the background executor,
    /// propagating any active test context from the calling thread.
    pub fn spawn<F>(&self, fut: F) -> Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = ScheduledTask {
            test_name: crate::testing::try_get_current_test_name(),
            test_storage_dir: crate::testing::try_get_test_storage_dir(),
            future: Box::pin(fut),
        };
        self.tx
            .send(task)
            .map_err(|_| anyhow::anyhow!("Background unasync executor stopped"))
    }

    /// Runs a `'static` async future on the background executor and blocks the
    /// calling thread until its result is ready.
    pub fn block_on<F, T>(&self, fut: F) -> Result<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel::<T>();
        self.spawn(async move {
            let out = fut.await;
            let _ = reply_tx.send(out);
        })?;
        reply_rx.recv().map_err(|_| {
            anyhow::anyhow!(
                "Background unasync task exited without producing a result"
            )
        })
    }
}

/// Returns the process-wide shared [`UnasyncExecutor`].
pub fn shared_executor() -> Result<&'static UnasyncExecutor> {
    static SHARED: OnceLock<std::result::Result<UnasyncExecutor, String>> =
        OnceLock::new();
    let res = SHARED.get_or_init(|| {
        UnasyncExecutor::new("ctb-unasync-worker").map_err(|e| format!("{e:#}"))
    });
    match res {
        Ok(exec) => Ok(exec),
        Err(msg) => bail!("{msg}"),
    }
}

/// Asynchronous source of byte chunks bridged into a [`SyncByteStream`].
pub trait AsyncChunkSource: Send + 'static {
    fn next_chunk(
        &mut self,
    ) -> Pin<
        Box<dyn Future<Output = std::io::Result<Option<Vec<u8>>>> + Send + '_>,
    >;
}

enum StreamEvent {
    Chunk(Vec<u8>, tokio::sync::OwnedSemaphorePermit),
    Eof,
    Error(std::io::Error),
}

/// Synchronous `std::io::Read` adapter over an [`AsyncChunkSource`].
///
/// Uses permit-bounded backpressure so fast async producers never buffer
/// unbounded data in memory, and cancels the async producer on `Drop`.
pub struct SyncByteStream {
    chunk_rx: std::sync::mpsc::Receiver<StreamEvent>,
    cancel: Arc<tokio::sync::Notify>,
    current: Option<(Vec<u8>, usize, tokio::sync::OwnedSemaphorePermit)>,
    eof: bool,
}

impl Drop for SyncByteStream {
    fn drop(&mut self) {
        self.cancel.notify_one();
    }
}

impl SyncByteStream {
    /// Starts streaming `source` on `executor` with bounded chunk buffering.
    pub fn new<S: AsyncChunkSource>(
        executor: &UnasyncExecutor,
        mut source: S,
    ) -> Result<Self> {
        let (chunk_tx, chunk_rx) = std::sync::mpsc::channel();
        let semaphore =
            Arc::new(tokio::sync::Semaphore::new(DEFAULT_STREAM_BUFFER_CHUNKS));
        let cancel = Arc::new(tokio::sync::Notify::new());
        let cancel_worker = Arc::clone(&cancel);

        executor.spawn(async move {
            let cancelled = cancel_worker.notified();
            tokio::pin!(cancelled);
            loop {
                let permit = tokio::select! {
                    biased;
                    () = &mut cancelled => return,
                    res = Arc::clone(&semaphore).acquire_owned() => match res {
                        Ok(permit) => permit,
                        Err(_) => return,
                    },
                };
                let next = tokio::select! {
                    biased;
                    () = &mut cancelled => return,
                    res = source.next_chunk() => res,
                };
                match next {
                    Ok(Some(chunk)) => {
                        if chunk.is_empty() {
                            continue;
                        }
                        if chunk_tx
                            .send(StreamEvent::Chunk(chunk, permit))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Ok(None) => {
                        drop(permit);
                        let _ = chunk_tx.send(StreamEvent::Eof);
                        return;
                    }
                    Err(err) => {
                        drop(permit);
                        let _ = chunk_tx.send(StreamEvent::Error(err));
                        return;
                    }
                }
            }
        })?;

        Ok(Self {
            chunk_rx,
            cancel,
            current: None,
            eof: false,
        })
    }
}

impl std::io::Read for SyncByteStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() || self.eof {
            return Ok(0);
        }

        loop {
            if self.current.is_none() {
                match self.chunk_rx.recv() {
                    Ok(StreamEvent::Chunk(chunk, permit)) => {
                        if chunk.is_empty() {
                            continue;
                        }
                        self.current = Some((chunk, 0, permit));
                    }
                    Ok(StreamEvent::Eof) => {
                        self.eof = true;
                        return Ok(0);
                    }
                    Ok(StreamEvent::Error(err)) => {
                        self.eof = true;
                        return Err(err);
                    }
                    Err(_) => {
                        self.eof = true;
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Async byte stream terminated before EOF",
                        ));
                    }
                }
            }

            let Some((ref chunk, ref mut offset, _)) = self.current else {
                continue;
            };
            let Some(remaining_slice) = chunk.get(*offset..) else {
                self.current = None;
                continue;
            };
            if remaining_slice.is_empty() {
                self.current = None;
                continue;
            }

            let to_copy = std::cmp::min(buf.len(), remaining_slice.len());
            let Some(src) = remaining_slice.get(..to_copy) else {
                return Err(std::io::Error::other(
                    "Invalid source slice bounds while reading byte stream",
                ));
            };
            let Some(dst) = buf.get_mut(..to_copy) else {
                return Err(std::io::Error::other(
                    "Invalid destination buffer bounds while reading byte stream",
                ));
            };
            dst.copy_from_slice(src);
            *offset = offset.saturating_add(to_copy);
            if *offset >= chunk.len() {
                self.current = None;
            }
            return Ok(to_copy);
        }
    }
}

/// Thread-safe lazy pool sharing reusable resources by configuration key.
pub struct SharedPool<K, V> {
    entries: OnceLock<Mutex<HashMap<K, Arc<V>>>>,
}

impl<K, V> Default for SharedPool<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SharedPool<K, V> {
    pub const fn new() -> Self {
        Self {
            entries: OnceLock::new(),
        }
    }

    /// Returns a shared `Arc<V>` for `key`, constructing and caching a new
    /// instance via `init` only when no entry for `key` exists yet.
    pub fn get_or_try_init<F>(&self, key: K, init: F) -> Result<Arc<V>>
    where
        K: Eq + Hash,
        F: FnOnce() -> Result<V>,
    {
        let map = self.entries.get_or_init(|| Mutex::new(HashMap::new()));
        if let Ok(guard) = map.lock()
            && let Some(existing) = guard.get(&key)
        {
            return Ok(Arc::clone(existing));
        }

        let created = Arc::new(init()?);
        if let Ok(mut guard) = map.lock() {
            let entry =
                guard.entry(key).or_insert_with(|| Arc::clone(&created));
            return Ok(Arc::clone(entry));
        }
        Ok(created)
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::{
        Arc, AsyncChunkSource, Future, Pin, SharedPool, SyncByteStream,
        shared_executor,
    };
    use std::io::Read;

    struct VecChunkSource {
        chunks: std::collections::VecDeque<std::io::Result<Vec<u8>>>,
    }

    impl AsyncChunkSource for VecChunkSource {
        fn next_chunk(
            &mut self,
        ) -> Pin<
            Box<
                dyn Future<Output = std::io::Result<Option<Vec<u8>>>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(());
                let _ = rx.await;
                self.chunks.pop_front().transpose()
            })
        }
    }

    #[crate::ctb_test]
    fn test_executor_inside_multi_thread_block_on_exceeds_coop_budget() {
        // Reproduces the exact #[tokio::main] CachedParkThread::block_on
        // environment where polling tokio::sync::oneshot on the caller thread
        // panicked in debug builds and spun at 100% CPU after 128 polls in
        // release builds.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let exec = shared_executor().unwrap();
            for i in 0..350_usize {
                let val = exec
                    .block_on(async move {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        let _ = tx.send(i * 2);
                        rx.await.unwrap()
                    })
                    .unwrap();
                assert_eq!(val, i * 2);

                let mut stream = SyncByteStream::new(
                    exec,
                    VecChunkSource {
                        chunks: std::collections::VecDeque::from([
                            Ok(b"chunk-".to_vec()),
                            Ok(i.to_string().into_bytes()),
                        ]),
                    },
                )
                .unwrap();
                let mut out = Vec::new();
                stream.read_to_end(&mut out).unwrap();
                assert_eq!(out, format!("chunk-{i}").into_bytes());
            }
        });
    }

    #[crate::ctb_test("tokio")]
    async fn test_sync_byte_stream_backpressure_cancellation_and_errors() {
        let exec = shared_executor().unwrap();

        let mut chunks = std::collections::VecDeque::new();
        for idx in 0..24_u8 {
            chunks.push_back(Ok(vec![idx; 19]));
        }
        let mut reader =
            SyncByteStream::new(exec, VecChunkSource { chunks }).unwrap();

        let mut collected = Vec::new();
        let mut small = [0_u8; 5];
        loop {
            let n = reader.read(&mut small).unwrap();
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&small[..n]);
        }
        assert_eq!(collected.len(), 24 * 19);

        // Early drop mid-stream cancels cleanly without deadlock.
        let mut mid = SyncByteStream::new(
            exec,
            VecChunkSource {
                chunks: std::collections::VecDeque::from([
                    Ok(b"first".to_vec()),
                    Ok(b"second".to_vec()),
                ]),
            },
        )
        .unwrap();
        let _ = mid.read(&mut small).unwrap();
        drop(mid);

        // Mid-stream error propagates to caller.
        let mut failing = SyncByteStream::new(
            exec,
            VecChunkSource {
                chunks: std::collections::VecDeque::from([
                    Ok(b"part".to_vec()),
                    Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "stalled",
                    )),
                ]),
            },
        )
        .unwrap();
        let mut buf = Vec::new();
        let err = failing.read_to_end(&mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }

    #[crate::ctb_test]
    fn test_shared_pool_reuses_entries_by_key() {
        let pool: SharedPool<u32, usize> = SharedPool::new();
        let builds = std::sync::atomic::AtomicUsize::new(0);
        let a1 = pool
            .get_or_try_init(1, || {
                Ok(builds.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 10)
            })
            .unwrap();
        let a2 = pool
            .get_or_try_init(1, || {
                Ok(builds.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 99)
            })
            .unwrap();
        let b1 = pool
            .get_or_try_init(2, || {
                Ok(builds.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 20)
            })
            .unwrap();
        assert!(Arc::ptr_eq(&a1, &a2));
        assert_eq!(*a1, 10);
        assert_eq!(*b1, 21);
        assert_eq!(builds.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
