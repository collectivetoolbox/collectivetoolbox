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

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use rustls::ClientConfig;
use rustls::RootCertStore;

use crate::branding;
use crate::invocation_settings::{self, TlsValidatorOverride};
use crate::pc_settings::{PcSettingBoolKey, get_bool_setting};
use crate::unasync::{
    AsyncChunkSource, SharedPool, SyncByteStream, shared_executor,
};

pub mod crlite;

pub const BASE_RETRY_DELAY_MS: u64 = 500;
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(120);
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
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            timeout: None,
            user_agent: None,
        }
    }
}

impl ClientOptions {
    pub fn startup() -> Self {
        Self {
            connect_timeout: Some(STARTUP_CONNECT_TIMEOUT),
            read_timeout: Some(STARTUP_REQUEST_TIMEOUT),
            timeout: Some(STARTUP_REQUEST_TIMEOUT),
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TlsCertificateSource {
    OperatingSystem,
    WebPki,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClientPoolKey {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    user_agent: String,
    tls_source: TlsCertificateSource,
    skip_crlite: bool,
}

static CLIENT_POOL: SharedPool<ClientPoolKey, reqwest::Client> =
    SharedPool::new();

#[derive(Debug, Clone)]
pub struct AsyncClient {
    pub inner: reqwest::Client,
    skip_crlite_ready_check: bool,
    timeout: Option<Duration>,
}

pub struct AsyncResponse {
    pub inner: reqwest::Response,
}

#[derive(Debug, Clone)]
pub struct BlockingClient {
    inner: reqwest::Client,
    skip_crlite_ready_check: bool,
    timeout: Option<Duration>,
}

enum BlockingResponseBody {
    Unread(reqwest::Response),
    Streaming(SyncByteStream),
    Consumed,
}

pub struct BlockingResponse {
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    content_length: Option<u64>,
    body: BlockingResponseBody,
}

struct ReqwestChunkSource {
    response: reqwest::Response,
}

impl AsyncChunkSource for ReqwestChunkSource {
    fn next_chunk(
        &mut self,
    ) -> Pin<
        Box<dyn Future<Output = std::io::Result<Option<Vec<u8>>>> + Send + '_>,
    > {
        Box::pin(async move {
            match self.response.chunk().await {
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

fn default_user_agent() -> String {
    let name = branding::user_agent_name();
    format!("{}/{}", name, crate::environment::ctb_version())
}

fn get_or_build_shared_reqwest_client(
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    user_agent: Option<String>,
    skip_crlite: bool,
) -> Result<reqwest::Client> {
    let tls_source = tls_certificate_source();
    // Reason for fallback: caller omitted custom User-Agent in ClientOptions; format application branding header
    let resolved_ua = user_agent.unwrap_or_else(default_user_agent);
    let key = ClientPoolKey {
        connect_timeout,
        read_timeout,
        user_agent: resolved_ua.clone(),
        tls_source,
        skip_crlite,
    };

    let shared = CLIENT_POOL.get_or_try_init(key, || {
        let tls = if skip_crlite {
            build_rustls_client_config_no_crlite()?
        } else {
            match tls_source {
                TlsCertificateSource::OperatingSystem => {
                    build_platform_rustls_client_config()?
                }
                TlsCertificateSource::WebPki => {
                    build_webpki_rustls_client_config()?
                }
            }
        };
        let mut builder = reqwest::Client::builder()
            .user_agent(resolved_ua)
            .use_preconfigured_tls(tls);
        if let Some(ct) = connect_timeout {
            builder = builder.connect_timeout(ct);
        }
        if let Some(rt) = read_timeout {
            builder = builder.read_timeout(rt);
        }
        builder.build().context("Failed to build HTTP client")
    })?;
    Ok((*shared).clone())
}

pub fn async_client(options: ClientOptions) -> Result<AsyncClient> {
    let skip_crlite =
        invocation_settings::get_settings().insecure_skip_crlite_check;
    let timeout = options.timeout;
    let inner = get_or_build_shared_reqwest_client(
        options.connect_timeout,
        options.read_timeout,
        options.user_agent,
        skip_crlite,
    )?;
    Ok(AsyncClient {
        inner,
        skip_crlite_ready_check: skip_crlite,
        timeout,
    })
}

pub fn blocking_client(options: ClientOptions) -> Result<BlockingClient> {
    let _ = shared_executor()?;
    let skip_crlite =
        invocation_settings::get_settings().insecure_skip_crlite_check;
    let timeout = options.timeout;
    let inner = get_or_build_shared_reqwest_client(
        options.connect_timeout,
        options.read_timeout,
        options.user_agent,
        skip_crlite,
    )?;
    Ok(BlockingClient {
        inner,
        skip_crlite_ready_check: skip_crlite,
        timeout,
    })
}

pub fn blocking_client_no_crlite() -> Result<BlockingClient> {
    blocking_client_no_crlite_with_timeout(None)
}

pub fn blocking_client_no_crlite_with_timeout(
    timeout: Option<Duration>,
) -> Result<BlockingClient> {
    let _ = shared_executor()?;
    let inner = get_or_build_shared_reqwest_client(
        Some(STARTUP_CONNECT_TIMEOUT),
        Some(DEFAULT_READ_TIMEOUT),
        None,
        true,
    )?;
    Ok(BlockingClient {
        inner,
        skip_crlite_ready_check: true,
        timeout,
    })
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

pub fn blocking_get_response_with_backoff(
    url: &str,
    retry_count: usize,
) -> Result<BlockingResponse> {
    blocking_client(ClientOptions::default())?
        .get_with_backoff(url, retry_count)
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
            if let Some(rem) = remaining
                && rem.is_zero()
            {
                bail!("Request to {url} timed out after {:?}", self.timeout);
            }

            let mut req = self.inner.get(url);
            if let Some(rem) = remaining {
                req = req.timeout(rem);
            }
            let send_fut = req.send();
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
            if let Some(rem) = remaining
                && rem.is_zero()
            {
                bail!("Request to {url} timed out after {:?}", self.timeout);
            }

            let mut builder = self.inner.post(url).body(body.clone());
            if let Some(ref h) = headers {
                builder = builder.headers(h.clone());
            }
            if let Some(rem) = remaining {
                builder = builder.timeout(rem);
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
            if let Some(rem) = remaining
                && rem.is_zero()
            {
                bail!("Request to {url} timed out after {:?}", self.timeout);
            }

            let mut req = self.inner.get(url);
            if let Some(rem) = remaining {
                req = req.timeout(rem);
            }
            let send_fut = req.send();
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
        ensure_network_allowed_in_tests(url)?;

        let exec = shared_executor()?;
        let host_retry_limit =
            invocation_settings::get_settings().retry_on_host_error;
        let max_host_retries = if retry_on_server_error {
            std::cmp::max(status_retry_count, host_retry_limit)
        } else {
            host_retry_limit
        };
        let mut attempt = 0;
        loop {
            let elapsed = start.elapsed();
            let remaining = self.timeout.map(|t| t.saturating_sub(elapsed));
            if let Some(rem) = remaining
                && rem.is_zero()
            {
                bail!("Request to {url} timed out after {:?}", self.timeout);
            }

            let client = self.inner.clone();
            let url_owned = url.to_string();
            let headers_cloned = headers.clone();
            let res = exec.block_on(async move {
                let mut req = client.get(&url_owned);
                if let Some(h) = headers_cloned {
                    req = req.headers(h);
                }
                if let Some(rem) = remaining {
                    req = req.timeout(rem);
                }
                req.send().await
            })?;

            match res {
                Ok(resp) => {
                    if retry_on_server_error
                        && resp.status().is_server_error()
                        && attempt < status_retry_count
                    {
                        attempt = attempt.saturating_add(1);
                        if let Some(delay) =
                            sleep_delay_bounded(attempt, start, self.timeout)
                        {
                            drop(resp);
                            std::thread::sleep(delay);
                            continue;
                        }
                    }
                    let status = resp.status();
                    let resp_headers = resp.headers().clone();
                    let content_length = resp.content_length();
                    return Ok(BlockingResponse {
                        status,
                        headers: resp_headers,
                        content_length,
                        body: BlockingResponseBody::Unread(resp),
                    });
                }
                Err(err) => {
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
        match std::mem::replace(&mut self.body, BlockingResponseBody::Consumed)
        {
            BlockingResponseBody::Unread(resp) => {
                let exec = shared_executor()?;
                exec.block_on(async move {
                    resp.bytes()
                        .await
                        .context("Failed to read HTTP response body")
                        .map(|body| body.to_vec())
                })?
            }
            BlockingResponseBody::Streaming(mut stream) => {
                let mut out = Vec::new();
                std::io::Read::read_to_end(&mut stream, &mut out)
                    .context("Failed to read HTTP response body")?;
                Ok(out)
            }
            BlockingResponseBody::Consumed => Ok(Vec::new()),
        }
    }

    pub fn text(mut self) -> Result<String> {
        match std::mem::replace(&mut self.body, BlockingResponseBody::Consumed)
        {
            BlockingResponseBody::Unread(resp) => {
                let exec = shared_executor()?;
                exec.block_on(async move {
                    resp.text()
                        .await
                        .context("Failed to read HTTP response body as text")
                })?
            }
            BlockingResponseBody::Streaming(mut stream) => {
                let mut raw = Vec::new();
                std::io::Read::read_to_end(&mut stream, &mut raw)
                    .context("Failed to read HTTP response body")?;
                Ok(String::from_utf8_lossy(&raw).into_owned())
            }
            BlockingResponseBody::Consumed => Ok(String::new()),
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

        if matches!(self.body, BlockingResponseBody::Unread(_)) {
            let prev = std::mem::replace(
                &mut self.body,
                BlockingResponseBody::Consumed,
            );
            if let BlockingResponseBody::Unread(resp) = prev {
                let exec = shared_executor().map_err(std::io::Error::other)?;
                let stream = SyncByteStream::new(
                    exec,
                    ReqwestChunkSource { response: resp },
                )
                .map_err(std::io::Error::other)?;
                self.body = BlockingResponseBody::Streaming(stream);
            }
        }

        let BlockingResponseBody::Streaming(ref mut stream) = self.body else {
            return Ok(0);
        };
        stream.read(buf)
    }
}

fn ensure_network_allowed_in_tests(url: &str) -> Result<()> {
    anyhow::ensure!(
        !crate::testing::is_in_test(),
        "Real HTTP requests are forbidden in tests; inject a mock HTTP client for {url}"
    );
    Ok(())
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
