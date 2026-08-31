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
use crate::get_embedded_asset;
use crate::invocation_settings::{self, TlsValidatorOverride};
use crate::pc_settings::{PcSettingBoolKey, get_bool_setting};

pub mod crlite;

pub const BASE_RETRY_DELAY_MS: u64 = 500;
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const STARTUP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

use include_dir::{Dir, include_dir};

static HTTPS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/https/data");

pub(crate) fn get_https_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&HTTPS_DATA_DIR, key)
}

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub connect_timeout: Option<Duration>,
    pub timeout: Option<Duration>,
    pub user_agent: Option<String>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            timeout: None,
            user_agent: None,
        }
    }
}

impl ClientOptions {
    pub fn startup() -> Self {
        Self {
            connect_timeout: Some(STARTUP_CONNECT_TIMEOUT),
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
}

pub struct AsyncResponse {
    pub inner: reqwest::Response,
}

#[derive(Debug, Clone)]
pub struct BlockingClient {
    inner: reqwest::blocking::Client,
    skip_crlite_ready_check: bool,
}

pub struct BlockingResponse {
    inner: reqwest::blocking::Response,
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
    Ok(AsyncClient {
        inner: build_async_client(options)?,
        skip_crlite_ready_check: skip_crlite,
    })
}

pub fn blocking_client(options: ClientOptions) -> Result<BlockingClient> {
    let skip_crlite =
        invocation_settings::get_settings().insecure_skip_crlite_check;
    Ok(BlockingClient {
        inner: build_blocking_client(options)?,
        skip_crlite_ready_check: skip_crlite,
    })
}

pub fn blocking_client_no_crlite() -> Result<BlockingClient> {
    static CACHE: std::sync::OnceLock<Result<BlockingClient, String>> =
        std::sync::OnceLock::new();
    let client_res = CACHE.get_or_init(|| {
        std::thread::spawn(|| {
            let tls = build_rustls_client_config_no_crlite().map_err(|e| {
                format!("Failed to build TLS config without CRLite: {e:#}")
            })?;
            let client = reqwest::blocking::Client::builder()
                .use_preconfigured_tls(tls)
                .build()
                .map_err(|e| {
                    format!("Failed to build blocking HTTP client: {e:#}")
                })?;
            Ok(BlockingClient {
                inner: client,
                skip_crlite_ready_check: true,
            })
        })
        .join()
        .map_err(|_| {
            "Thread panicked while constructing blocking HTTP client"
                .to_string()
        })
        .and_then(|res| res)
    });

    match client_res {
        Ok(client) => Ok(client.clone()),
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

impl AsyncClient {
    async fn ensure_crlite_ready(&self, url: &str) -> Result<()> {
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
        crate::spawn_blocking_with_current_test_name(
            crlite::ensure_crlite_cache_ready_sync,
        )
        .await
        .context("CRLite ready check task panicked")?
        .context("Failed to ensure CRLite cache is ready")?;
        Ok(())
    }

    pub async fn get(&self, url: &str) -> Result<AsyncResponse> {
        self.ensure_crlite_ready(url).await?;
        let response = self
            .inner
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to GET {url}"))?;
        Ok(AsyncResponse { inner: response })
    }

    pub async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<AsyncResponse> {
        self.ensure_crlite_ready(url).await?;
        let mut builder = self.inner.post(url).body(body);
        if let Some(h) = headers {
            builder = builder.headers(h);
        }
        let response = builder
            .send()
            .await
            .with_context(|| format!("Failed to POST {url}"))?;
        Ok(AsyncResponse { inner: response })
    }

    pub async fn get_with_backoff(
        &self,
        url: &str,
        retry_count: usize,
    ) -> Result<AsyncResponse> {
        self.ensure_crlite_ready(url).await?;
        let mut attempt = 0;
        loop {
            let res = self.inner.get(url).send().await;
            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if (status.is_server_error() || status.as_u16() == 429)
                        && attempt < retry_count
                    {
                        attempt = attempt.saturating_add(1);
                        tokio::time::sleep(backoff_delay(attempt)).await;
                        continue;
                    }
                    return Ok(AsyncResponse { inner: resp });
                }
                Err(err) => {
                    if is_transient_error(&err) && attempt < retry_count {
                        attempt = attempt.saturating_add(1);
                        tokio::time::sleep(backoff_delay(attempt)).await;
                        continue;
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
    fn ensure_crlite_ready(&self, url: &str) -> Result<()> {
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
        crlite::ensure_crlite_cache_ready_sync()
            .context("Failed to ensure CRLite cache is ready")?;
        Ok(())
    }

    pub fn get(&self, url: &str) -> Result<BlockingResponse> {
        self.ensure_crlite_ready(url)?;
        let response = self
            .inner
            .get(url)
            .send()
            .with_context(|| format!("Failed to GET {url}"))?;
        Ok(BlockingResponse { inner: response })
    }

    pub fn get_with_backoff(
        &self,
        url: &str,
        retry_count: usize,
    ) -> Result<BlockingResponse> {
        self.ensure_crlite_ready(url)?;
        let mut attempt = 0;
        loop {
            let res = self.inner.get(url).send();
            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_server_error() && attempt < retry_count {
                        attempt = attempt.saturating_add(1);
                        std::thread::sleep(backoff_delay(attempt));
                        continue;
                    }
                    return Ok(BlockingResponse { inner: resp });
                }
                Err(err) => {
                    if is_transient_error(&err) && attempt < retry_count {
                        attempt = attempt.saturating_add(1);
                        std::thread::sleep(backoff_delay(attempt));
                        continue;
                    }
                    return Err(anyhow::Error::from(err)
                        .context(format!("Failed to GET {url}")));
                }
            }
        }
    }
}

impl BlockingResponse {
    pub fn status_code(&self) -> u16 {
        self.inner.status().as_u16()
    }

    pub fn is_success(&self) -> bool {
        self.inner.status().is_success()
    }

    pub fn bytes(self) -> Result<Vec<u8>> {
        self.inner
            .bytes()
            .context("Failed to read HTTP response body")
            .map(|body| body.to_vec())
    }

    pub fn text(self) -> Result<String> {
        self.inner
            .text()
            .context("Failed to read HTTP response body as text")
    }

    pub fn copy_to<W: std::io::Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<u64> {
        self.inner
            .copy_to(writer)
            .context("Failed to stream HTTP response body")
    }
}

fn build_async_client(options: ClientOptions) -> Result<reqwest::Client> {
    let builder = configure_async_builder(reqwest::Client::builder(), options)?;
    builder.build().context("Failed to build async HTTP client")
}

fn build_blocking_client(
    options: ClientOptions,
) -> Result<reqwest::blocking::Client> {
    let builder = configure_blocking_builder(
        reqwest::blocking::Client::builder(),
        options,
    )?;
    builder
        .build()
        .context("Failed to build blocking HTTP client")
}

fn configure_async_builder(
    mut builder: reqwest::ClientBuilder,
    options: ClientOptions,
) -> Result<reqwest::ClientBuilder> {
    if let Some(connect_timeout) = options.connect_timeout {
        builder = builder.connect_timeout(connect_timeout);
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

fn configure_blocking_builder(
    mut builder: reqwest::blocking::ClientBuilder,
    options: ClientOptions,
) -> Result<reqwest::blocking::ClientBuilder> {
    if let Some(connect_timeout) = options.connect_timeout {
        builder = builder.connect_timeout(connect_timeout);
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
    configure_blocking_tls(builder)
}

fn configure_async_tls(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::ClientBuilder> {
    let tls = build_rustls_client_config()?;
    Ok(builder.use_preconfigured_tls(tls))
}

fn configure_blocking_tls(
    builder: reqwest::blocking::ClientBuilder,
) -> Result<reqwest::blocking::ClientBuilder> {
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
