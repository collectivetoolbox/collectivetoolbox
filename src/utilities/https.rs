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

//! Utilities for working with HTTP.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use rustls::ClientConfig;
use rustls::RootCertStore;

use crate::branding;
use crate::invocation_settings::{self, TlsValidatorOverride};
use crate::pc_settings::{PcSettingBoolKey, get_bool_setting};

pub mod crlite;

pub const BASE_RETRY_DELAY_MS: u64 = 500;
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const STARTUP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn get_https_data(key: &str) -> Option<Vec<u8>> {
    // Reason for fallback: strip optional fixtures/ prefix, defaulting to full relative key
    let clean = key.strip_prefix("fixtures/").unwrap_or(key);
    crate::load_manifest_fixture(
        &format!("{}/https", env!("CARGO_MANIFEST_DIR")),
        clean,
    )
}

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub connect_timeout: Option<Duration>,
    pub read_timeout: Option<Duration>,
    pub timeout: Option<Duration>,
    pub user_agent: Option<String>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            read_timeout: None,
            timeout: None,
            user_agent: None,
        }
    }
}

impl ClientOptions {
    pub fn startup() -> Self {
        Self {
            connect_timeout: Some(STARTUP_CONNECT_TIMEOUT),
            read_timeout: None,
            timeout: Some(STARTUP_REQUEST_TIMEOUT),
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TlsCertificateSource {
    OperatingSystem,
    WebPki,
}

#[derive(Debug, Clone)]
pub struct AsyncClient {
    pub inner: reqwest::Client,
    skip_crlite_ready_check: bool,
    timeout: Option<Duration>,
}

pub struct AsyncResponse {
    pub inner: reqwest::Response,
}

const BLOCKING_STREAM_BUFFER_CHUNKS: usize = 8;

enum StreamEvent {
    Chunk(Vec<u8>, tokio::sync::OwnedSemaphorePermit),
    Eof,
    Error(std::io::Error),
}

enum BodyAction {
    Bytes(std::sync::mpsc::Sender<Result<Vec<u8>>>),
    Text(std::sync::mpsc::Sender<Result<String>>),
    Stream {
        chunk_tx: std::sync::mpsc::Sender<StreamEvent>,
        semaphore: Arc<tokio::sync::Semaphore>,
        cancel: Arc<tokio::sync::Notify>,
    },
}

struct ResponseHead {
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    content_length: Option<u64>,
    action_tx: tokio::sync::oneshot::Sender<BodyAction>,
}

enum WorkerSendError {
    Transport(reqwest::Error),
    Other(anyhow::Error),
}

struct WorkerRequest {
    url: String,
    headers: Option<reqwest::header::HeaderMap>,
    timeout: Option<Duration>,
    test_name: Option<String>,
    test_storage_dir: Option<std::path::PathBuf>,
    reply_tx: std::sync::mpsc::Sender<
        std::result::Result<ResponseHead, WorkerSendError>,
    >,
}

struct BlockingWorker {
    tx: tokio::sync::mpsc::UnboundedSender<WorkerRequest>,
}

impl std::fmt::Debug for BlockingWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockingWorker").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct BlockingClient {
    worker: Arc<BlockingWorker>,
    skip_crlite_ready_check: bool,
    timeout: Option<Duration>,
    #[cfg(test)]
    bypass_test_network_guard: bool,
}

struct ActiveBodyStream {
    chunk_rx: std::sync::mpsc::Receiver<StreamEvent>,
    cancel: Arc<tokio::sync::Notify>,
    current: Option<(Vec<u8>, usize, tokio::sync::OwnedSemaphorePermit)>,
    eof: bool,
}

impl Drop for ActiveBodyStream {
    fn drop(&mut self) {
        self.cancel.notify_one();
    }
}

enum BlockingBodyState {
    Idle(tokio::sync::oneshot::Sender<BodyAction>),
    Streaming(ActiveBodyStream),
    Consumed,
}

pub struct BlockingResponse {
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    content_length: Option<u64>,
    body: BlockingBodyState,
    _worker: Arc<BlockingWorker>,
}

pub fn tls_certificate_source() -> TlsCertificateSource {
    if let Some(override_source) =
        invocation_settings::get_settings().tls_validator_override
    {
        return match override_source {
            TlsValidatorOverride::Bundled => TlsCertificateSource::WebPki,
            TlsValidatorOverride::System => {
                TlsCertificateSource::OperatingSystem
            }
        };
    }

    if get_bool_setting(PcSettingBoolKey::UseOsCaCertificates) {
        return TlsCertificateSource::OperatingSystem;
    }
    TlsCertificateSource::WebPki
}

pub fn async_client(options: ClientOptions) -> Result<AsyncClient> {
    let skip_crlite =
        invocation_settings::get_settings().insecure_skip_crlite_check;
    let timeout = options.timeout;
    Ok(AsyncClient {
        inner: build_async_client(options)?,
        skip_crlite_ready_check: skip_crlite,
        timeout,
    })
}

pub fn blocking_client(options: ClientOptions) -> Result<BlockingClient> {
    let skip_crlite =
        invocation_settings::get_settings().insecure_skip_crlite_check;
    let timeout = options.timeout;
    let builder = configure_async_builder(reqwest::Client::builder(), options)?;
    let worker = spawn_blocking_worker_from_builder(builder)?;
    Ok(BlockingClient {
        worker,
        skip_crlite_ready_check: skip_crlite,
        timeout,
        #[cfg(test)]
        bypass_test_network_guard: false,
    })
}

pub fn blocking_client_no_crlite() -> Result<BlockingClient> {
    blocking_client_no_crlite_with_timeout(None)
}

pub fn blocking_client_no_crlite_with_timeout(
    timeout: Option<Duration>,
) -> Result<BlockingClient> {
    static CACHE: std::sync::OnceLock<Result<BlockingClient, String>> =
        std::sync::OnceLock::new();
    let client_res = CACHE.get_or_init(|| {
        let tls = build_rustls_client_config_no_crlite().map_err(|e| {
            format!("Failed to build TLS config without CRLite: {e:#}")
        })?;
        let builder = reqwest::Client::builder()
            .connect_timeout(STARTUP_CONNECT_TIMEOUT)
            .use_preconfigured_tls(tls);
        let worker =
            spawn_blocking_worker_from_builder(builder).map_err(|e| {
                format!("Failed to build blocking HTTP client: {e:#}")
            })?;
        Ok(BlockingClient {
            worker,
            skip_crlite_ready_check: true,
            timeout: None,
            #[cfg(test)]
            bypass_test_network_guard: false,
        })
    });

    match client_res {
        Ok(client) => {
            let mut cloned = client.clone();
            cloned.timeout = timeout;
            Ok(cloned)
        }
        Err(e) => Err(anyhow::anyhow!("{e}")),
    }
}

pub async fn get(url: &str) -> Result<Vec<u8>> {
    let response = async_client(ClientOptions::default())?.get(url).await?;
    response.bytes().await
}

pub fn blocking_get(url: &str) -> Result<Vec<u8>> {
    let response = blocking_client(ClientOptions::default())?.get(url)?;
    response.bytes()
}

pub fn blocking_get_response(url: &str) -> Result<BlockingResponse> {
    blocking_client(ClientOptions::default())?.get(url)
}

pub fn blocking_get_response_with_headers(
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
) -> Result<BlockingResponse> {
    blocking_client(ClientOptions::default())?.get_with_headers(url, headers)
}

impl AsyncClient {
    async fn ensure_crlite_ready(
        &self,
        url: &str,
        start: std::time::Instant,
    ) -> Result<()> {
        if self.skip_crlite_ready_check {
            return Ok(());
        }
        // This is string slicing, not downloading.
        if !url
            .get(0..8)
            .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
        {
            return Ok(());
        }
        let remaining = match self.timeout {
            Some(timeout) => {
                let rem = timeout.saturating_sub(start.elapsed());
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
                Some(rem)
            }
            None => Some(STARTUP_REQUEST_TIMEOUT),
        };
        crate::spawn_blocking_with_current_test_name(move || {
            crlite::ensure_crlite_cache_ready_with_timeout(remaining)
        })
        .await
        .context("CRLite ready check task panicked")?
        .context("Failed to ensure CRLite cache is ready")?;
        Ok(())
    }

    pub async fn get(&self, url: &str) -> Result<AsyncResponse> {
        let start = std::time::Instant::now();
        self.ensure_crlite_ready(url, start).await?;
        ensure_network_allowed_in_tests(url)?;
        let retry_limit =
            invocation_settings::get_settings().retry_on_host_error;
        let mut attempt = 0;
        loop {
            let elapsed = start.elapsed();
            let remaining = self.timeout.map(|t| t.saturating_sub(elapsed));
            if let Some(rem) = remaining {
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
            }

            let send_fut = self.inner.get(url).send();
            let res = match remaining {
                Some(rem) => match tokio::time::timeout(rem, send_fut).await {
                    Ok(send_res) => send_res,
                    Err(_) => {
                        bail!(
                            "Request to {url} timed out after {:?}",
                            self.timeout
                        );
                    }
                },
                None => send_fut.await,
            };

            match res {
                Ok(resp) => return Ok(AsyncResponse { inner: resp }),
                Err(err) => {
                    if is_transient_error(&err) && attempt < retry_limit {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Err(anyhow::Error::from(err)
                        .context(format!("Failed to GET {url}")));
                }
            }
        }
    }

    pub async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<AsyncResponse> {
        let start = std::time::Instant::now();
        self.ensure_crlite_ready(url, start).await?;
        ensure_network_allowed_in_tests(url)?;
        let retry_limit =
            invocation_settings::get_settings().retry_on_host_error;
        let mut attempt = 0;
        loop {
            let elapsed = start.elapsed();
            let remaining = self.timeout.map(|t| t.saturating_sub(elapsed));
            if let Some(rem) = remaining {
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
            }

            let mut builder = self.inner.post(url).body(body.clone());
            if let Some(ref h) = headers {
                builder = builder.headers(h.clone());
            }
            let send_fut = builder.send();
            let res = match remaining {
                Some(rem) => match tokio::time::timeout(rem, send_fut).await {
                    Ok(send_res) => send_res,
                    Err(_) => {
                        bail!(
                            "Request to {url} timed out after {:?}",
                            self.timeout
                        );
                    }
                },
                None => send_fut.await,
            };

            match res {
                Ok(resp) => return Ok(AsyncResponse { inner: resp }),
                Err(err) => {
                    if is_transient_error(&err) && attempt < retry_limit {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Err(anyhow::Error::from(err)
                        .context(format!("Failed to POST {url}")));
                }
            }
        }
    }

    pub async fn get_with_backoff(
        &self,
        url: &str,
        retry_count: usize,
    ) -> Result<AsyncResponse> {
        let start = std::time::Instant::now();
        self.ensure_crlite_ready(url, start).await?;
        ensure_network_allowed_in_tests(url)?;
        let host_retry_limit =
            invocation_settings::get_settings().retry_on_host_error;
        let max_host_retries = std::cmp::max(retry_count, host_retry_limit);
        let mut attempt = 0;
        loop {
            let elapsed = start.elapsed();
            let remaining = self.timeout.map(|t| t.saturating_sub(elapsed));
            if let Some(rem) = remaining {
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
            }

            let send_fut = self.inner.get(url).send();
            let res = match remaining {
                Some(rem) => match tokio::time::timeout(rem, send_fut).await {
                    Ok(send_res) => send_res,
                    Err(_) => {
                        bail!(
                            "Request to {url} timed out after {:?}",
                            self.timeout
                        );
                    }
                },
                None => send_fut.await,
            };

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if (status.is_server_error() || status.as_u16() == 429)
                        && attempt < retry_count
                    {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Ok(AsyncResponse { inner: resp });
                }
                Err(err) => {
                    if is_transient_error(&err) && attempt < max_host_retries {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Err(anyhow::Error::from(err)
                        .context(format!("Failed to GET {url}")));
                }
            }
        }
    }
}

impl AsyncResponse {
    pub fn status_code(&self) -> u16 {
        self.inner.status().as_u16()
    }

    pub fn is_success(&self) -> bool {
        self.inner.status().is_success()
    }

    pub async fn bytes(self) -> Result<Vec<u8>> {
        self.inner
            .bytes()
            .await
            .context("Failed to read HTTP response body")
            .map(|body| body.to_vec())
    }

    pub async fn text(self) -> Result<String> {
        self.inner
            .text()
            .await
            .context("Failed to read HTTP response body as text")
    }
}

trait AsyncResponseBody: Send + 'static {
    fn into_bytes(
        self: Box<Self>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<u8>>> + Send>,
    >;
    fn into_text(
        self: Box<Self>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String>> + Send>,
    >;
    fn next_chunk(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::io::Result<Option<Vec<u8>>>>
                + Send
                + '_,
        >,
    >;
}

impl AsyncResponseBody for reqwest::Response {
    fn into_bytes(
        self: Box<Self>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<u8>>> + Send>,
    > {
        Box::pin(async move {
            (*self)
                .bytes()
                .await
                .context("Failed to read HTTP response body")
                .map(|body| body.to_vec())
        })
    }

    fn into_text(
        self: Box<Self>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String>> + Send>,
    > {
        Box::pin(async move {
            (*self)
                .text()
                .await
                .context("Failed to read HTTP response body as text")
        })
    }

    fn next_chunk(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::io::Result<Option<Vec<u8>>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            match self.chunk().await {
                Ok(Some(chunk)) => Ok(Some(Vec::from(chunk))),
                Ok(None) => Ok(None),
                Err(err) => {
                    let kind = if err.is_timeout() {
                        std::io::ErrorKind::TimedOut
                    } else {
                        std::io::ErrorKind::Other
                    };
                    Err(std::io::Error::new(kind, err))
                }
            }
        })
    }
}

async fn drive_response_body(
    mut body: Box<dyn AsyncResponseBody>,
    action_rx: tokio::sync::oneshot::Receiver<BodyAction>,
) {
    let Ok(action) = action_rx.await else {
        return;
    };
    match action {
        BodyAction::Bytes(tx) => {
            let _ = tx.send(body.into_bytes().await);
        }
        BodyAction::Text(tx) => {
            let _ = tx.send(body.into_text().await);
        }
        BodyAction::Stream {
            chunk_tx,
            semaphore,
            cancel,
        } => {
            let cancelled = cancel.notified();
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
                    res = body.next_chunk() => res,
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
        }
    }
}

fn spawn_blocking_worker_from_builder(
    builder: reqwest::ClientBuilder,
) -> Result<Arc<BlockingWorker>> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WorkerRequest>();
    let (startup_tx, startup_rx) = std::sync::mpsc::channel::<Result<()>>();

    std::thread::Builder::new()
        .name("ctb-https-blocking".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Failed to create runtime for blocking HTTP client")
            {
                Ok(rt) => rt,
                Err(err) => {
                    let _ = startup_tx.send(Err(err));
                    return;
                }
            };

            rt.block_on(async move {
                let client = match builder
                    .build()
                    .context("Failed to build blocking HTTP client")
                {
                    Ok(client) => client,
                    Err(err) => {
                        let _ = startup_tx.send(Err(err));
                        return;
                    }
                };
                if startup_tx.send(Ok(())).is_err() {
                    return;
                }

                while let Some(req) = rx.recv().await {
                    let client = client.clone();
                    tokio::spawn(crate::testing::scope_optional_test_context(
                        req.test_name,
                        req.test_storage_dir,
                        async move {
                            let mut builder = client.get(&req.url);
                            if let Some(headers) = req.headers {
                                builder = builder.headers(headers);
                            }
                            if let Some(timeout) = req.timeout {
                                builder = builder.timeout(timeout);
                            }
                            match builder.send().await {
                                Ok(resp) => {
                                    let status = resp.status();
                                    let headers = resp.headers().clone();
                                    let content_length = resp.content_length();
                                    let (action_tx, action_rx) =
                                        tokio::sync::oneshot::channel();
                                    let head = ResponseHead {
                                        status,
                                        headers,
                                        content_length,
                                        action_tx,
                                    };
                                    if req.reply_tx.send(Ok(head)).is_ok() {
                                        drive_response_body(
                                            Box::new(resp),
                                            action_rx,
                                        )
                                        .await;
                                    }
                                }
                                Err(err) => {
                                    let _ = req.reply_tx.send(Err(
                                        WorkerSendError::Transport(err),
                                    ));
                                }
                            }
                        },
                    ));
                }
            });
        })
        .context("Failed to spawn blocking HTTP worker thread")?;

    match startup_rx.recv() {
        Ok(Ok(())) => Ok(Arc::new(BlockingWorker { tx })),
        Ok(Err(err)) => Err(err),
        Err(_) => bail!("Blocking HTTP worker thread panicked during startup"),
    }
}

impl BlockingClient {
    fn ensure_crlite_ready(
        &self,
        url: &str,
        start: std::time::Instant,
    ) -> Result<()> {
        if self.skip_crlite_ready_check {
            return Ok(());
        }
        // This is string slicing, not downloading.
        if !url
            .get(0..8)
            .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
        {
            return Ok(());
        }
        let remaining = match self.timeout {
            Some(timeout) => {
                let rem = timeout.saturating_sub(start.elapsed());
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
                Some(rem)
            }
            None => Some(STARTUP_REQUEST_TIMEOUT),
        };
        crlite::ensure_crlite_cache_ready_with_timeout(remaining)
            .context("Failed to ensure CRLite cache is ready")?;
        Ok(())
    }

    fn send_get(
        &self,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
        status_retry_count: usize,
        retry_on_server_error: bool,
    ) -> Result<BlockingResponse> {
        let start = std::time::Instant::now();
        self.ensure_crlite_ready(url, start)?;
        #[cfg(test)]
        if !self.bypass_test_network_guard {
            ensure_network_allowed_in_tests(url)?;
        }
        #[cfg(not(test))]
        ensure_network_allowed_in_tests(url)?;

        let host_retry_limit =
            invocation_settings::get_settings().retry_on_host_error;
        let max_host_retries = if retry_on_server_error {
            std::cmp::max(status_retry_count, host_retry_limit)
        } else {
            host_retry_limit
        };
        let test_name = crate::testing::try_get_current_test_name();
        let test_storage_dir = crate::testing::try_get_test_storage_dir();
        let mut attempt = 0;
        loop {
            let elapsed = start.elapsed();
            let remaining = self.timeout.map(|t| t.saturating_sub(elapsed));
            if let Some(rem) = remaining {
                if rem.is_zero() {
                    bail!(
                        "Request to {url} timed out after {:?}",
                        self.timeout
                    );
                }
            }

            let (reply_tx, reply_rx) = std::sync::mpsc::channel();
            let worker_req = WorkerRequest {
                url: url.to_string(),
                headers: headers.clone(),
                timeout: remaining,
                test_name: test_name.clone(),
                test_storage_dir: test_storage_dir.clone(),
                reply_tx,
            };
            if self.worker.tx.send(worker_req).is_err() {
                bail!("Blocking HTTP worker stopped before sending GET {url}");
            }

            let recv_res = reply_rx.recv().map_err(|_| {
                anyhow::anyhow!(
                    "Blocking HTTP worker exited while awaiting GET {url}"
                )
            })?;

            match recv_res {
                Ok(head) => {
                    if retry_on_server_error
                        && head.status.is_server_error()
                        && attempt < status_retry_count
                    {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            drop(head);
                            std::thread::sleep(delay);
                            continue;
                        }
                    }
                    return Ok(BlockingResponse {
                        status: head.status,
                        headers: head.headers,
                        content_length: head.content_length,
                        body: BlockingBodyState::Idle(head.action_tx),
                        _worker: Arc::clone(&self.worker),
                    });
                }
                Err(WorkerSendError::Transport(err)) => {
                    if is_transient_error(&err) && attempt < max_host_retries {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            std::thread::sleep(delay);
                            continue;
                        }
                    }
                    return Err(anyhow::Error::from(err)
                        .context(format!("Failed to GET {url}")));
                }
                Err(WorkerSendError::Other(err)) => {
                    return Err(err.context(format!("Failed to GET {url}")));
                }
            }
        }
    }

    pub fn get(&self, url: &str) -> Result<BlockingResponse> {
        self.send_get(url, None, 0, false)
    }

    pub fn get_with_backoff(
        &self,
        url: &str,
        retry_count: usize,
    ) -> Result<BlockingResponse> {
        self.send_get(url, None, retry_count, true)
    }

    pub fn get_with_headers(
        &self,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<BlockingResponse> {
        self.send_get(url, headers, 0, false)
    }

    pub fn get_with_headers_and_backoff(
        &self,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
        retry_count: usize,
    ) -> Result<BlockingResponse> {
        self.send_get(url, headers, retry_count, true)
    }
}

impl BlockingResponse {
    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn status(&self) -> reqwest::StatusCode {
        self.status
    }

    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        &self.headers
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn bytes(mut self) -> Result<Vec<u8>> {
        match std::mem::replace(&mut self.body, BlockingBodyState::Consumed) {
            BlockingBodyState::Idle(action_tx) => {
                let (tx, rx) = std::sync::mpsc::channel();
                if action_tx.send(BodyAction::Bytes(tx)).is_err() {
                    bail!(
                        "Blocking HTTP response driver exited before reading body"
                    );
                }
                rx.recv().map_err(|_| {
                    anyhow::anyhow!(
                        "Blocking HTTP response driver exited while reading body"
                    )
                })?
            }
            BlockingBodyState::Streaming(stream) => {
                self.body = BlockingBodyState::Streaming(stream);
                let mut out = Vec::new();
                std::io::Read::read_to_end(&mut self, &mut out)
                    .context("Failed to read HTTP response body")?;
                Ok(out)
            }
            BlockingBodyState::Consumed => Ok(Vec::new()),
        }
    }

    pub fn text(mut self) -> Result<String> {
        match std::mem::replace(&mut self.body, BlockingBodyState::Consumed) {
            BlockingBodyState::Idle(action_tx) => {
                let (tx, rx) = std::sync::mpsc::channel();
                if action_tx.send(BodyAction::Text(tx)).is_err() {
                    bail!(
                        "Blocking HTTP response driver exited before reading text body"
                    );
                }
                rx.recv().map_err(|_| {
                    anyhow::anyhow!(
                        "Blocking HTTP response driver exited while reading text body"
                    )
                })?
            }
            BlockingBodyState::Streaming(stream) => {
                self.body = BlockingBodyState::Streaming(stream);
                let raw = self.bytes()?;
                Ok(String::from_utf8_lossy(&raw).into_owned())
            }
            BlockingBodyState::Consumed => Ok(String::new()),
        }
    }

    pub fn copy_to<W: std::io::Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<u64> {
        std::io::copy(self, writer)
            .context("Failed to stream HTTP response body")
    }
}

impl std::io::Read for BlockingResponse {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if matches!(self.body, BlockingBodyState::Idle(_)) {
            let prev =
                std::mem::replace(&mut self.body, BlockingBodyState::Consumed);
            if let BlockingBodyState::Idle(action_tx) = prev {
                let (chunk_tx, chunk_rx) = std::sync::mpsc::channel();
                let semaphore = Arc::new(tokio::sync::Semaphore::new(
                    BLOCKING_STREAM_BUFFER_CHUNKS,
                ));
                let cancel = Arc::new(tokio::sync::Notify::new());
                if action_tx
                    .send(BodyAction::Stream {
                        chunk_tx,
                        semaphore,
                        cancel: Arc::clone(&cancel),
                    })
                    .is_err()
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "Blocking HTTP response driver exited before streaming body",
                    ));
                }
                self.body = BlockingBodyState::Streaming(ActiveBodyStream {
                    chunk_rx,
                    cancel,
                    current: None,
                    eof: false,
                });
            }
        }

        let BlockingBodyState::Streaming(ref mut stream) = self.body else {
            return Ok(0);
        };

        if stream.eof {
            return Ok(0);
        }

        loop {
            if stream.current.is_none() {
                match stream.chunk_rx.recv() {
                    Ok(StreamEvent::Chunk(chunk, permit)) => {
                        if chunk.is_empty() {
                            continue;
                        }
                        stream.current = Some((chunk, 0, permit));
                    }
                    Ok(StreamEvent::Eof) => {
                        stream.eof = true;
                        return Ok(0);
                    }
                    Ok(StreamEvent::Error(err)) => {
                        stream.eof = true;
                        return Err(err);
                    }
                    Err(_) => {
                        stream.eof = true;
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Blocking HTTP stream terminated before EOF",
                        ));
                    }
                }
            }

            let Some((ref chunk, ref mut offset, _)) = stream.current else {
                continue;
            };
            let Some(remaining_slice) = chunk.get(*offset..) else {
                stream.current = None;
                continue;
            };
            if remaining_slice.is_empty() {
                stream.current = None;
                continue;
            }

            let to_copy = std::cmp::min(buf.len(), remaining_slice.len());
            let Some(src) = remaining_slice.get(..to_copy) else {
                return Err(std::io::Error::other(
                    "Invalid source slice bounds while reading HTTP body",
                ));
            };
            let Some(dst) = buf.get_mut(..to_copy) else {
                return Err(std::io::Error::other(
                    "Invalid destination buffer bounds while reading HTTP body",
                ));
            };
            dst.copy_from_slice(src);
            *offset = offset.saturating_add(to_copy);
            if *offset >= chunk.len() {
                stream.current = None;
            }
            return Ok(to_copy);
        }
    }
}

fn ensure_network_allowed_in_tests(url: &str) -> Result<()> {
    anyhow::ensure!(
        !crate::testing::is_in_test(),
        "Real HTTP requests are forbidden in tests; inject a mock HTTP client for {url}"
    );
    Ok(())
}

fn build_async_client(options: ClientOptions) -> Result<reqwest::Client> {
    let builder = configure_async_builder(reqwest::Client::builder(), options)?;
    builder.build().context("Failed to build async HTTP client")
}

fn configure_async_builder(
    mut builder: reqwest::ClientBuilder,
    options: ClientOptions,
) -> Result<reqwest::ClientBuilder> {
    if let Some(connect_timeout) = options.connect_timeout {
        builder = builder.connect_timeout(connect_timeout);
    }
    if let Some(read_timeout) = options.read_timeout {
        builder = builder.read_timeout(read_timeout);
    }
    if let Some(timeout) = options.timeout {
        builder = builder.timeout(timeout);
    }
    // Reason for fallback: caller omitted custom User-Agent in ClientOptions; format application branding header
    let user_agent = options.user_agent.unwrap_or_else(|| {
        let name = branding::user_agent_name();
        format!("{}/{}", name, crate::environment::ctb_version())
    });
    builder = builder.user_agent(user_agent);
    configure_async_tls(builder)
}

fn configure_async_tls(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::ClientBuilder> {
    let tls = build_rustls_client_config()?;
    Ok(builder.use_preconfigured_tls(tls))
}

fn build_rustls_client_config() -> Result<ClientConfig> {
    if invocation_settings::get_settings().insecure_skip_crlite_check {
        return build_rustls_client_config_no_crlite();
    }
    match tls_certificate_source() {
        TlsCertificateSource::OperatingSystem => {
            build_platform_rustls_client_config()
        }
        TlsCertificateSource::WebPki => build_webpki_rustls_client_config(),
    }
}

fn build_platform_rustls_client_config() -> Result<ClientConfig> {
    let verifier = Arc::new(crlite::CRLiteVerifier::with_platform_verifier()?);
    Ok(ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth())
}

fn build_webpki_rustls_client_config() -> Result<ClientConfig> {
    let verifier = Arc::new(crlite::CRLiteVerifier::with_webpki_roots(
        webpki_root_store(),
    )?);
    Ok(ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth())
}

fn build_rustls_client_config_no_crlite() -> Result<ClientConfig> {
    match tls_certificate_source() {
        TlsCertificateSource::OperatingSystem => {
            // Reason for fallback: default rustls process-global CryptoProvider falls back to aws_lc_rs provider
            let provider = rustls::crypto::CryptoProvider::get_default()
                .cloned()
                .unwrap_or_else(|| {
                    Arc::new(rustls::crypto::aws_lc_rs::default_provider())
                });
            let verifier = Arc::new(
                rustls_platform_verifier::Verifier::new(provider).map_err(
                    |error| {
                        anyhow::anyhow!(
                            "Failed to build platform verifier: {error:?}"
                        )
                    },
                )?,
            );
            Ok(ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth())
        }
        TlsCertificateSource::WebPki => {
            let verifier = rustls::client::WebPkiServerVerifier::builder(
                Arc::new(webpki_root_store()),
            )
            .build()
            .map_err(|e| {
                anyhow::anyhow!("Failed to build WebPkiServerVerifier: {e:?}")
            })?;
            Ok(ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth())
        }
    }
}

fn webpki_root_store() -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    let (_valid, _invalid) = root_store.add_parsable_certificates(
        webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
    );
    root_store
}

pub async fn get_success(url: &str) -> Result<Vec<u8>> {
    let response = async_client(ClientOptions::default())?.get(url).await?;
    if !response.is_success() {
        bail!("HTTP GET {url} failed: {}", response.status_code());
    }
    response.bytes().await
}

pub fn blocking_get_success(url: &str) -> Result<Vec<u8>> {
    let response = blocking_client(ClientOptions::default())?.get(url)?;
    if !response.is_success() {
        bail!("HTTP GET {url} failed: {}", response.status_code());
    }
    response.bytes()
}

pub async fn get_with_backoff(
    url: &str,
    retry_count: usize,
) -> Result<Vec<u8>> {
    let client = async_client(ClientOptions::default())?;
    let response = client.get_with_backoff(url, retry_count).await?;
    response.bytes().await
}

pub fn blocking_get_with_backoff(
    url: &str,
    retry_count: usize,
) -> Result<Vec<u8>> {
    let client = blocking_client(ClientOptions::default())?;
    let response = client.get_with_backoff(url, retry_count)?;
    response.bytes()
}

pub async fn get_success_with_backoff(
    url: &str,
    retry_count: usize,
) -> Result<Vec<u8>> {
    let client = async_client(ClientOptions::default())?;
    let response = client.get_with_backoff(url, retry_count).await?;
    if !response.is_success() {
        bail!("HTTP GET {url} failed: {}", response.status_code());
    }
    response.bytes().await
}

pub fn blocking_get_success_with_backoff(
    url: &str,
    retry_count: usize,
) -> Result<Vec<u8>> {
    let client = blocking_client(ClientOptions::default())?;
    let response = client.get_with_backoff(url, retry_count)?;
    if !response.is_success() {
        bail!("HTTP GET {url} failed: {}", response.status_code());
    }
    response.bytes()
}

fn is_transient_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_body()
}

fn backoff_delay(attempt: usize) -> Duration {
    // Reason for fallback: attempt counter overflow saturates to maximum exponent for delay calculation
    let exponent = u32::try_from(attempt).unwrap_or(u32::MAX).saturating_sub(1);
    // 2^exponent fits in u64 only if exponent < 64.
    // Since delay is capped at 10 seconds, and BASE_RETRY_DELAY_MS is 500,
    // 500 * 2^5 = 16000 (16 seconds), which already exceeds 10 seconds.
    // So capping exponent at 6 is perfectly fine and avoids overflow panics.
    let exponent = std::cmp::min(exponent, 6);
    // Reason for fallback: power computation overflow defaults to 10,000ms max exponential multiplier
    let delay_ms = BASE_RETRY_DELAY_MS
        .saturating_mul(2u64.checked_pow(exponent).unwrap_or(10000));
    let jitter_ms = u64::from(rand::random::<u8>().rem_euclid(100));
    let delay = Duration::from_millis(delay_ms.saturating_add(jitter_ms));
    std::cmp::min(delay, Duration::from_secs(10))
}

fn sleep_delay_bounded(
    attempt: usize,
    start: std::time::Instant,
    timeout: Option<Duration>,
) -> Option<Duration> {
    let delay = backoff_delay(attempt);
    if let Some(timeout) = timeout {
        let elapsed = start.elapsed();
        let remaining = timeout.saturating_sub(elapsed);
        if remaining.is_zero() {
            return None;
        }
        Some(std::cmp::min(delay, remaining))
    } else {
        Some(delay)
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
    use super::*;

    #[crate::ctb_test("tokio")]
    async fn test_async_requests_require_mocks() -> Result<()> {
        let mut client = async_client(ClientOptions::default())?;
        for skip_crlite in [false, true] {
            client.skip_crlite_ready_check = skip_crlite;
            for url in [
                "http://unmocked.invalid/test",
                "https://unmocked.invalid/test",
            ] {
                let results = [
                    client.get(url).await,
                    client.get_with_backoff(url, 10).await,
                    client.post(url, Vec::new(), None).await,
                ];
                for result in results {
                    let error = result
                        .err()
                        .context("Unmocked HTTP request succeeded")?;
                    anyhow::ensure!(
                        format!("{error:#}").contains(
                            "Real HTTP requests are forbidden in tests"
                        ),
                        "Unexpected error: {error:#}"
                    );
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_blocking_requests_require_mocks() -> Result<()> {
        let client = blocking_client_no_crlite()?;
        for url in [
            "http://unmocked.invalid/test",
            "https://unmocked.invalid/test",
        ] {
            let results = [
                client.get(url),
                client.get_with_backoff(url, 10),
                client.get_with_headers(url, None),
                client.get_with_headers_and_backoff(url, None, 10),
            ];
            for result in results {
                let error =
                    result.err().context("Unmocked HTTP request succeeded")?;
                anyhow::ensure!(
                    error
                        .to_string()
                        .contains("Real HTTP requests are forbidden in tests"),
                    "Unexpected error: {error:#}"
                );
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_network_guard_applies_to_unnamed_worker_threads() -> Result<()> {
        let error = std::thread::spawn(|| {
            assert!(crate::testing::try_get_current_test_name().is_none());
            ensure_network_allowed_in_tests("http://unmocked.invalid/test")
        })
        .join()
        .map_err(|_| anyhow::anyhow!("Network guard test worker panicked"))?
        .expect_err("Unnamed test worker must not access the network");
        anyhow::ensure!(
            error
                .to_string()
                .contains("Real HTTP requests are forbidden in tests"),
            "Unexpected error: {error:#}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_sleep_delay_bounded_no_timeout() {
        let start = std::time::Instant::now();
        let delay = sleep_delay_bounded(1, start, None);
        assert!(delay.is_some());
        // Reason for fallback / expect: unwrap_or on infallible Some check
        assert!(delay.unwrap_or_default() >= Duration::from_millis(500));
    }

    #[crate::ctb_test]
    fn test_sleep_delay_bounded_expired() {
        let start = std::time::Instant::now();
        let delay = sleep_delay_bounded(1, start, Some(Duration::ZERO));
        assert_eq!(delay, None);
    }

    #[crate::ctb_test]
    fn test_sleep_delay_bounded_clamps_to_remaining() {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(50);
        let delay = sleep_delay_bounded(1, start, Some(timeout));
        assert!(delay.is_some());
        // Reason for fallback / expect: unwrap_or on infallible Some check
        assert!(delay.unwrap_or_default() <= timeout);
    }

    struct MockAsyncBody {
        chunks: std::collections::VecDeque<std::io::Result<Vec<u8>>>,
    }

    impl AsyncResponseBody for MockAsyncBody {
        fn into_bytes(
            mut self: Box<Self>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Vec<u8>>> + Send>,
        > {
            Box::pin(async move {
                let mut out = Vec::new();
                while let Some(next) = self.next_chunk().await? {
                    out.extend_from_slice(&next);
                }
                Ok(out)
            })
        }

        fn into_text(
            self: Box<Self>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<String>> + Send>,
        > {
            Box::pin(async move {
                let bytes = self.into_bytes().await?;
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            })
        }

        fn next_chunk(
            &mut self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = std::io::Result<Option<Vec<u8>>>,
                    > + Send
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

    fn build_mock_blocking_client<F>(handler: F) -> BlockingClient
    where
        F: Fn(
                &str,
                Option<&reqwest::header::HeaderMap>,
            ) -> Result<(
                reqwest::StatusCode,
                reqwest::header::HeaderMap,
                Option<u64>,
                Vec<std::io::Result<Vec<u8>>>,
            )> + Send
            + Sync
            + 'static,
    {
        let handler = Arc::new(handler);
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<WorkerRequest>();
        let (startup_tx, startup_rx) = std::sync::mpsc::channel::<()>();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let _ = startup_tx.send(());
                while let Some(req) = rx.recv().await {
                    let handler = Arc::clone(&handler);
                    tokio::spawn(async move {
                        let (ytx, yrx) = tokio::sync::oneshot::channel();
                        let _ = ytx.send(());
                        let _ = yrx.await;
                        match handler(&req.url, req.headers.as_ref()) {
                            Ok((status, headers, content_length, chunks)) => {
                                let (action_tx, action_rx) =
                                    tokio::sync::oneshot::channel();
                                let head = ResponseHead {
                                    status,
                                    headers,
                                    content_length,
                                    action_tx,
                                };
                                if req.reply_tx.send(Ok(head)).is_ok() {
                                    let body = MockAsyncBody {
                                        chunks: chunks.into_iter().collect(),
                                    };
                                    drive_response_body(
                                        Box::new(body),
                                        action_rx,
                                    )
                                    .await;
                                }
                            }
                            Err(err) => {
                                let _ = req
                                    .reply_tx
                                    .send(Err(WorkerSendError::Other(err)));
                            }
                        }
                    });
                }
            });
        });
        startup_rx.recv().unwrap();

        BlockingClient {
            worker: Arc::new(BlockingWorker { tx }),
            skip_crlite_ready_check: true,
            timeout: None,
            bypass_test_network_guard: true,
        }
    }

    #[crate::ctb_test]
    fn test_blocking_client_inside_multi_thread_block_on_exceeds_coop_budget() {
        // Reproduces the exact #[tokio::main] CachedParkThread::block_on
        // environment where reqwest::blocking panicked in debug builds and
        // entered a 100%-CPU zero-syscall spin loop after 128 oneshot polls
        // (~64 requests = 528 MB of 8 MiB chunks) in release builds.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let shared_client = build_mock_blocking_client(|url, _headers| {
                let payload = format!("payload:{url}").into_bytes();
                let len = u64::try_from(payload.len()).unwrap();
                Ok((
                    reqwest::StatusCode::PARTIAL_CONTENT,
                    reqwest::header::HeaderMap::new(),
                    Some(len),
                    vec![Ok(payload[..4].to_vec()), Ok(payload[4..].to_vec())],
                ))
            });

            for i in 0..300_usize {
                let url = format!("https://mock.invalid/chunk/{i}");
                let mut resp = shared_client.get(&url).unwrap();
                assert_eq!(resp.status(), reqwest::StatusCode::PARTIAL_CONTENT);
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut resp, &mut buf).unwrap();
                assert_eq!(buf, format!("payload:{url}").into_bytes());
            }

            // Also verify per-iteration client construction + bytes()/text()
            // well past the 128-poll cooperative scheduling budget.
            for i in 0..160_usize {
                let one_shot_client =
                    build_mock_blocking_client(|url, _headers| {
                        Ok((
                            reqwest::StatusCode::OK,
                            reqwest::header::HeaderMap::new(),
                            None,
                            vec![Ok(url.as_bytes().to_vec())],
                        ))
                    });
                let url = format!("https://mock.invalid/item/{i}");
                if i % 2 == 0 {
                    let bytes =
                        one_shot_client.get(&url).unwrap().bytes().unwrap();
                    assert_eq!(bytes, url.as_bytes());
                } else {
                    let text =
                        one_shot_client.get(&url).unwrap().text().unwrap();
                    assert_eq!(text, url);
                }
            }
        });
    }

    #[crate::ctb_test("tokio")]
    async fn test_blocking_response_streaming_backpressure_and_errors() {
        let client = build_mock_blocking_client(|url, _headers| {
            if url.ends_with("/error") {
                return Ok((
                    reqwest::StatusCode::OK,
                    reqwest::header::HeaderMap::new(),
                    None,
                    vec![
                        Ok(b"partial".to_vec()),
                        Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "simulated read timeout",
                        )),
                    ],
                ));
            }

            // 24 chunks (> BLOCKING_STREAM_BUFFER_CHUNKS) to exercise bounded
            // permit backpressure across small incremental reads.
            let mut chunks = Vec::new();
            for idx in 0..24_u8 {
                chunks.push(Ok(vec![idx; 17]));
            }
            Ok((
                reqwest::StatusCode::OK,
                reqwest::header::HeaderMap::new(),
                Some(24 * 17),
                chunks,
            ))
        });

        let mut resp = client.get("https://mock.invalid/stream").unwrap();
        let mut collected = Vec::new();
        let mut tiny = [0_u8; 5];
        loop {
            let n = std::io::Read::read(&mut resp, &mut tiny).unwrap();
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&tiny[..n]);
        }
        assert_eq!(collected.len(), 24 * 17);
        for idx in 0..24_usize {
            assert!(
                collected[idx * 17..(idx + 1) * 17]
                    .iter()
                    .all(|&b| usize::from(b) == idx)
            );
        }

        // Drop mid-stream and before reading to verify clean cancellation.
        let mut mid_drop = client.get("https://mock.invalid/stream").unwrap();
        let n = std::io::Read::read(&mut mid_drop, &mut tiny).unwrap();
        assert!(n > 0);
        drop(mid_drop);

        let unread_drop = client.get("https://mock.invalid/stream").unwrap();
        drop(unread_drop);

        // Mid-stream error must propagate rather than truncating silently.
        let mut err_resp = client.get("https://mock.invalid/error").unwrap();
        let mut out = Vec::new();
        let err = std::io::Read::read_to_end(&mut err_resp, &mut out)
            .expect_err("mid-stream error must fail read_to_end");
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    }

    // #[crate::ctb_test("tokio")]
    // async fn test_async_client_zero_retry_on_invalid_host() {
    //     let mut settings = invocation_settings::get_settings();
    //     settings.retry_on_host_error = 0;
    //     invocation_settings::set_settings(settings);

    //     let options = ClientOptions {
    //         connect_timeout: Some(Duration::from_millis(200)),
    //         timeout: Some(Duration::from_millis(500)),
    //         user_agent: None,
    //     };
    //     let Ok(client) = async_client(options) else {
    //         return;
    //     };
    //     let start = std::time::Instant::now();
    //     let res = client
    //         .get("http://invalid.invalid-dns-test-domain.ctb-local")
    //         .await;
    //     assert!(res.is_err());
    //     assert!(start.elapsed() < Duration::from_secs(4));
    // }
}
