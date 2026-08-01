// SPDX-License-Identifier for parts derived from Mozilla Firefox and moz_crlite_query: MPL-2.0
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CRLite cache and local-state helpers.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use clubcard::Queryable;
use clubcard_crlite::{CRLiteClubcard, CRLiteKey, CRLiteQuery, CRLiteStatus};
use rustls::client::WebPkiServerVerifier;
use rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    CertificateError, DigitallySignedStruct, Error, RootCertStore,
    SignatureScheme,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x509_parser::extensions::ParsedExtension;
use x509_parser::parse_x509_certificate;

use crate::bin2hex;
use crate::pc_settings::get_settings;
use crate::storage::get_storage_dir;
use crate::warn_fmt;

/// Default maximum age for locally cached CRLite state before a refresh is
/// required.
pub const DEFAULT_CRLITE_MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
pub const DEFAULT_CRLITE_CHANNEL: &str = "default";
pub const DEFAULT_CRLITE_REQUIRED_COVERED_TIMESTAMPS: usize = 1;
pub const DEFAULT_MOZILLA_CRLITE_COLLECTION_URL: &str = "https://firefox.settings.services.mozilla.com/v1/buckets/security-state/collections/cert-revocations/records";
pub const DEFAULT_MOZILLA_ATTACHMENTS_BASE_URL: &str =
    "https://firefox-settings-attachments.cdn.mozilla.net/";

const CRLITE_CACHE_DIR_NAME: &str = "crlite";
const CRLITE_MANIFEST_FILE_NAME: &str = "manifest.json";
const CRLITE_COLLECTION_FILE_NAME: &str = "collection.json";

/// Where this instance should fetch CRLite data from.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CRLiteSourceKind {
    /// Fetch directly from Mozilla Remote Settings.
    Mozilla,
    /// Fetch from the ctoolbox-hosted mirror.
    CtbMirror,
}

/// Local CRLite source configuration.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CRLiteSource {
    pub kind: CRLiteSourceKind,
    pub collection_url: String,
    pub attachments_base_url: String,
}

impl CRLiteSource {
    pub fn mozilla() -> Self {
        Self {
            kind: CRLiteSourceKind::Mozilla,
            collection_url: DEFAULT_MOZILLA_CRLITE_COLLECTION_URL.to_string(),
            attachments_base_url: DEFAULT_MOZILLA_ATTACHMENTS_BASE_URL
                .to_string(),
        }
    }

    pub fn ctb_mirror(server_url: &str) -> Self {
        let server_url = server_url.trim_end_matches('/');
        Self {
            kind: CRLiteSourceKind::CtbMirror,
            collection_url: format!("{server_url}/crlite/manifest.json"),
            attachments_base_url: format!("{server_url}/crlite/artifacts/"),
        }
    }
}

/// Metadata describing one cached CRLite artifact.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CRLiteArtifact {
    pub relative_path: String,
    pub sha256_hex: String,
    pub size_bytes: u64,
    pub effective_timestamp: Option<u64>,
    pub is_incremental: bool,
}

/// Persisted CRLite cache state shared by the workspace updater, web routes,
/// and HTTPS verifier.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CRLiteCacheManifest {
    pub source: CRLiteSource,
    pub channel: String,
    pub last_updated_unix_seconds: u64,
    pub current_filter: Option<CRLiteArtifact>,
    #[serde(default)]
    pub deltas: Vec<CRLiteArtifact>,
}

impl CRLiteCacheManifest {
    /// Returns true when the manifest is still fresh enough to reuse.
    pub fn is_fresh(&self, now: SystemTime, max_age: Duration) -> bool {
        let Some(updated_at) = UNIX_EPOCH
            .checked_add(Duration::from_secs(self.last_updated_unix_seconds))
        else {
            return false;
        };

        let Ok(age) = now.duration_since(updated_at) else {
            return true;
        };

        age <= max_age
    }

    /// Returns the absolute path for the current full filter if present.
    pub fn current_filter_path(&self) -> Result<Option<PathBuf>> {
        let Some(filter) = &self.current_filter else {
            return Ok(None);
        };

        let dir = get_crlite_cache_dir()?;
        Ok(Some(dir.join(&filter.relative_path)))
    }

    /// Returns the absolute paths for cached delta artifacts.
    pub fn delta_paths(&self) -> Result<Vec<PathBuf>> {
        let dir = get_crlite_cache_dir()?;
        Ok(self
            .deltas
            .iter()
            .map(|delta| dir.join(&delta.relative_path))
            .collect())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CRLiteCollectionResponse {
    pub data: Vec<CRLiteCollectionRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CRLiteCollectionRecord {
    pub id: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    pub details: CRLiteRecordDetails,
    pub attachment: CRLiteAttachment,
    #[serde(default)]
    pub incremental: bool,
    #[serde(rename = "effectiveTimestamp", default)]
    pub effective_timestamp: Option<u64>,
}

impl CRLiteCollectionRecord {
    pub fn channel_name(&self) -> &str {
        self.channel.as_deref().unwrap_or(DEFAULT_CRLITE_CHANNEL)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CRLiteRecordDetails {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CRLiteAttachment {
    pub hash: String,
    pub size: u64,
    pub filename: String,
    pub location: String,
    #[serde(default)]
    pub mimetype: Option<String>,
}

/// A parsed CRLite filter along with the bytes used to construct it.
pub struct LoadedCRLiteFilter {
    pub bytes: Vec<u8>,
    pub clubcard: CRLiteClubcard,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CRLiteTimestamp {
    pub log_id: [u8; 32],
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CRLiteLookupStatus {
    Good,
    NoFilter,
    NotCovered,
    NotEnrolled,
    Revoked,
}

pub struct CachedCRLiteState {
    pub manifest: CRLiteCacheManifest,
    pub filters: Vec<LoadedCRLiteFilter>,
}

pub struct CRLiteVerifier {
    inner: Arc<dyn ServerCertVerifier>,
}

impl std::fmt::Debug for CRLiteVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRLiteVerifier").finish()
    }
}

impl LoadedCRLiteFilter {
    fn revocation_status(
        &self,
        crlite_key: &CRLiteKey,
        timestamps: &[CRLiteTimestamp],
        required_covered_timestamps: usize,
    ) -> CRLiteLookupStatus {
        let timestamp_entries: Vec<(&[u8; 32], u64)> = timestamps
            .iter()
            .map(|timestamp| (&timestamp.log_id, timestamp.timestamp))
            .collect();
        let mut covered_timestamp_count = 0usize;
        for timestamp in timestamp_entries.iter().copied() {
            if CRLiteQuery::new(crlite_key, Some(timestamp))
                .in_universe(self.clubcard.universe())
            {
                covered_timestamp_count = covered_timestamp_count.saturating_add(1);
            }
        }
        if covered_timestamp_count < required_covered_timestamps {
            return CRLiteLookupStatus::NotCovered;
        }

        match self
            .clubcard
            .contains(crlite_key, timestamp_entries.iter().copied())
        {
            CRLiteStatus::Good => CRLiteLookupStatus::Good,
            CRLiteStatus::NotCovered => CRLiteLookupStatus::NotCovered,
            CRLiteStatus::NotEnrolled => CRLiteLookupStatus::NotEnrolled,
            CRLiteStatus::Revoked => CRLiteLookupStatus::Revoked,
        }
    }
}

impl CachedCRLiteState {
    pub fn load_from_disk() -> Result<Self> {
        let manifest = load_crlite_manifest()?
            .ok_or_else(|| anyhow::anyhow!("No CRLite manifest is cached"))?;
        let mut filters = Vec::new();

        if let Some(path) = manifest.current_filter_path()? {
            let filter_meta =
                manifest.current_filter.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Current filter metadata missing")
                })?;
            let bytes = std::fs::read(&path).with_context(|| {
                format!("Failed to read CRLite filter {}", path.display())
            })?;
            let hash = bin2hex(Sha256::digest(&bytes));
            if u64::try_from(bytes.len()).ok() != Some(filter_meta.size_bytes)
                || hash != filter_meta.sha256_hex
            {
                anyhow::bail!(
                    "CRLite filter integrity check failed for {}",
                    path.display()
                );
            }
            filters.push(load_crlite_filter_from_bytes(bytes)?);
        }

        let delta_paths = manifest.delta_paths()?;
        for (i, path) in delta_paths.into_iter().enumerate() {
            let delta_meta = manifest.deltas.get(i).ok_or_else(|| {
                anyhow::anyhow!("delta_paths index {} out of bounds for manifest.deltas", i)
            })?;
            let bytes = std::fs::read(&path).with_context(|| {
                format!("Failed to read CRLite delta filter {}", path.display())
            })?;
            let hash = bin2hex(Sha256::digest(&bytes));
            if u64::try_from(bytes.len()).ok() != Some(delta_meta.size_bytes)
                || hash != delta_meta.sha256_hex
            {
                anyhow::bail!(
                    "CRLite delta filter integrity check failed for {}",
                    path.display()
                );
            }
            filters.push(load_crlite_filter_from_bytes(bytes)?);
        }

        Ok(Self { manifest, filters })
    }

    pub fn is_fresh(&self, now: SystemTime, max_age: Duration) -> bool {
        self.manifest.is_fresh(now, max_age)
    }

    pub fn revocation_status(
        &self,
        issuer_spki: &[u8],
        serial_number: &[u8],
        timestamps: &[CRLiteTimestamp],
    ) -> CRLiteLookupStatus {
        self.revocation_status_with_coverage_requirement(
            issuer_spki,
            serial_number,
            timestamps,
            DEFAULT_CRLITE_REQUIRED_COVERED_TIMESTAMPS,
        )
    }

    pub fn revocation_status_with_coverage_requirement(
        &self,
        issuer_spki: &[u8],
        serial_number: &[u8],
        timestamps: &[CRLiteTimestamp],
        required_covered_timestamps: usize,
    ) -> CRLiteLookupStatus {
        if !self.is_fresh(SystemTime::now(), DEFAULT_CRLITE_MAX_AGE) {
            return CRLiteLookupStatus::NoFilter;
        }
        if self.filters.is_empty() {
            return CRLiteLookupStatus::NoFilter;
        }

        let issuer_spki_hash = Sha256::digest(issuer_spki);
        let crlite_key =
            CRLiteKey::new(issuer_spki_hash.as_ref(), serial_number);
        let mut maybe_good = false;
        let mut covered = false;

        let mut max_filter_timestamp = self
            .manifest
            .current_filter
            .as_ref()
            .and_then(|f| f.effective_timestamp)
            .unwrap_or(0);
        for delta in &self.manifest.deltas {
            if let Some(ts) = delta.effective_timestamp {
                if ts > max_filter_timestamp {
                    max_filter_timestamp = ts;
                }
            }
        }

        let max_merge_delay_ms = 24 * 60 * 60 * 1000; // 24 hours in milliseconds
        let covered_timestamps: Vec<CRLiteTimestamp> = timestamps
            .iter()
            .copied()
            .filter(|ts| {
                ts.timestamp
                    .checked_add(max_merge_delay_ms)
                    .is_some_and(|limit| limit <= max_filter_timestamp)
            })
            .collect();

        for filter in &self.filters {
            match filter.revocation_status(
                &crlite_key,
                &covered_timestamps,
                required_covered_timestamps,
            ) {
                CRLiteLookupStatus::Revoked => {
                    return CRLiteLookupStatus::Revoked;
                }
                CRLiteLookupStatus::Good => maybe_good = true,
                CRLiteLookupStatus::NotEnrolled => covered = true,
                CRLiteLookupStatus::NoFilter
                | CRLiteLookupStatus::NotCovered => {}
            }
        }

        if maybe_good {
            return CRLiteLookupStatus::Good;
        }
        if covered {
            return CRLiteLookupStatus::NotEnrolled;
        }

        CRLiteLookupStatus::NotCovered
    }
}

pub fn resolve_crlite_source() -> CRLiteSource {
    let settings = get_settings();

    if settings.is_official_ctb_domain() {
        return CRLiteSource::mozilla();
    }

    CRLiteSource::ctb_mirror(&crate::official_url())
}

pub fn validate_relative_artifact_path(relative_path: &str) -> Result<PathBuf> {
    let path = Path::new(relative_path);
    ensure!(
        path.is_relative(),
        "CRLite artifact path must be relative: {relative_path}"
    );

    for component in path.components() {
        ensure!(
            matches!(component, std::path::Component::Normal(_)),
            "Invalid CRLite artifact path component in {relative_path}"
        );
    }

    Ok(path.to_path_buf())
}

fn collection_attachment_url(base_url: &str, location: &str) -> String {
    if location.starts_with("http://") || location.starts_with("https://") {
        return location.to_string();
    }

    let base_url = base_url.trim_end_matches('/');
    let location = location.trim_start_matches('/');
    format!("{base_url}/{location}")
}

pub fn select_current_records(
    response: &CRLiteCollectionResponse,
    channel: &str,
) -> Result<(CRLiteCollectionRecord, Vec<CRLiteCollectionRecord>)> {
    let mut records: Vec<CRLiteCollectionRecord> = response
        .data
        .iter()
        .filter(|record| record.channel_name() == channel)
        .cloned()
        .collect();

    ensure!(
        !records.is_empty(),
        "No CRLite records found for channel {channel}"
    );

    records.sort_by_key(|record| record.effective_timestamp.unwrap_or(0));

    let full_filter = records
        .iter()
        .rfind(|record| !record.incremental)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("No CRLite full filter found for channel {channel}")
        })?;

    let full_effective_timestamp = full_filter.effective_timestamp.unwrap_or(0);
    let deltas = records
        .into_iter()
        .filter(|record| {
            record.incremental
                && record.effective_timestamp.unwrap_or(0)
                    >= full_effective_timestamp
        })
        .collect();

    Ok((full_filter, deltas))
}

fn artifact_from_record(
    record: &CRLiteCollectionRecord,
    channel: &str,
) -> CRLiteArtifact {
    CRLiteArtifact {
        relative_path: format!("{channel}/{}", record.attachment.filename),
        sha256_hex: record.attachment.hash.clone(),
        size_bytes: record.attachment.size,
        effective_timestamp: record.effective_timestamp,
        is_incremental: record.incremental,
    }
}

type TestDownloadOverride = dyn Fn(&str) -> Result<Vec<u8>> + Send + Sync;

fn test_download_overrides()
-> &'static Mutex<HashMap<String, Arc<TestDownloadOverride>>> {
    static TEST_DOWNLOAD_OVERRIDES: OnceLock<
        Mutex<HashMap<String, Arc<TestDownloadOverride>>>,
    > = OnceLock::new();
    TEST_DOWNLOAD_OVERRIDES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_test_download_overrides()
-> std::sync::MutexGuard<'static, HashMap<String, Arc<TestDownloadOverride>>> {
    match test_download_overrides().lock() {
        Ok(overrides) => overrides,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub struct TestDownloadOverrideGuard {
    test_name: String,
    previous: Option<Arc<TestDownloadOverride>>,
}

impl Drop for TestDownloadOverrideGuard {
    fn drop(&mut self) {
        let mut overrides = lock_test_download_overrides();

        if let Some(previous) = self.previous.take() {
            overrides.insert(self.test_name.clone(), previous);
        } else {
            overrides.remove(&self.test_name);
        }
    }
}

pub fn set_test_download_override<F>(
    download: F,
) -> Result<TestDownloadOverrideGuard>
where
    F: Fn(&str) -> Result<Vec<u8>> + Send + Sync + 'static,
{
    let test_name =
        crate::testing::try_get_current_test_name().ok_or_else(|| {
            anyhow::anyhow!(
                "CRLite test download override requires an active named test"
            )
        })?;

    let mut overrides = lock_test_download_overrides();
    let previous = overrides.insert(test_name.clone(), Arc::new(download));

    Ok(TestDownloadOverrideGuard {
        test_name,
        previous,
    })
}

fn try_download_from_override(url: &str) -> Result<Option<Vec<u8>>> {
    let Some(test_name) = crate::testing::try_get_current_test_name() else {
        return Ok(None);
    };

    let maybe_download =
        lock_test_download_overrides().get(&test_name).cloned();

    maybe_download.map(|download| download(url)).transpose()
}

fn download_text(url: &str) -> Result<String> {
    if let Some(bytes) = try_download_from_override(url)? {
        return String::from_utf8(bytes)
            .context("Failed to decode CRLite test response body as UTF-8");
    }

    let client = super::blocking_client_no_crlite()?;
    let response = client
        .get_with_backoff(url, 3)
        .with_context(|| format!("GET {url}"))?;
    if !response.is_success() {
        anyhow::bail!(
            "HTTP GET {url} failed with status {}",
            response.status_code()
        );
    }
    response.text()
}

fn download_bytes(url: &str) -> Result<Vec<u8>> {
    if let Some(bytes) = try_download_from_override(url)? {
        return Ok(bytes);
    }

    let client = super::blocking_client_no_crlite()?;
    let response = client
        .get_with_backoff(url, 3)
        .with_context(|| format!("GET {url}"))?;
    if !response.is_success() {
        anyhow::bail!(
            "HTTP GET {url} failed with status {}",
            response.status_code()
        );
    }
    response.bytes()
}

fn write_artifact(relative_path: &str, bytes: &[u8]) -> Result<()> {
    let cache_dir = get_crlite_cache_dir()?;
    let relative_path = validate_relative_artifact_path(relative_path)?;
    let absolute_path = cache_dir.join(relative_path);
    let parent = match absolute_path.parent() {
        Some(parent) => parent,
        None => {
            return Err(anyhow::anyhow!(
                "CRLite artifact path has no parent: {}",
                absolute_path.display()
            ));
        }
    };
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "Failed to create CRLite artifact directory {}",
            parent.display()
        )
    })?;
    let tmp_path = absolute_path.with_extension("tmp");
    std::fs::write(&tmp_path, bytes).with_context(|| {
        format!(
            "Failed to write temporary CRLite artifact {}",
            tmp_path.display()
        )
    })?;
    std::fs::rename(&tmp_path, &absolute_path).with_context(|| {
        format!(
            "Failed to replace CRLite artifact {} with {}",
            absolute_path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}

fn fetch_and_cache_artifact(
    source: &CRLiteSource,
    record: &CRLiteCollectionRecord,
    channel: &str,
) -> Result<CRLiteArtifact> {
    let artifact = artifact_from_record(record, channel);
    let cache_dir = get_crlite_cache_dir()?;
    let relative_path =
        validate_relative_artifact_path(&artifact.relative_path)?;
    let absolute_path = cache_dir.join(relative_path);

    if let Ok(bytes) = std::fs::read(&absolute_path) {
        let computed_hash = bin2hex(Sha256::digest(&bytes));
        if u64::try_from(bytes.len()).ok() == Some(record.attachment.size)
            && computed_hash == record.attachment.hash
        {
            return Ok(artifact);
        }
    }

    let url = collection_attachment_url(
        &source.attachments_base_url,
        &record.attachment.location,
    );
    let bytes = download_bytes(&url)?;
    let computed_hash = bin2hex(Sha256::digest(&bytes));
    ensure!(
        computed_hash == record.attachment.hash,
        "CRLite artifact hash mismatch for {}: expected {}, got {}",
        url,
        record.attachment.hash,
        computed_hash
    );
    let byte_len = u64::try_from(bytes.len())
        .context("CRLite artifact length exceeds u64 range")?;
    ensure!(
        byte_len == record.attachment.size,
        "CRLite artifact size mismatch for {}: expected {}, got {}",
        url,
        record.attachment.size,
        byte_len
    );

    write_artifact(&artifact.relative_path, &bytes)?;
    Ok(artifact)
}

pub fn refresh_crlite_cache_sync() -> Result<CRLiteCacheManifest> {
    let source = resolve_crlite_source();
    let collection_json = download_text(&source.collection_url)?;
    let response: CRLiteCollectionResponse =
        serde_json::from_str(&collection_json)
            .context("Failed to parse CRLite collection response")?;
    let (full_filter_record, delta_records) =
        select_current_records(&response, DEFAULT_CRLITE_CHANNEL)?;

    let current_filter = fetch_and_cache_artifact(
        &source,
        &full_filter_record,
        DEFAULT_CRLITE_CHANNEL,
    )?;
    let mut deltas = Vec::with_capacity(delta_records.len());
    for record in &delta_records {
        deltas.push(fetch_and_cache_artifact(
            &source,
            record,
            DEFAULT_CRLITE_CHANNEL,
        )?);
    }

    save_crlite_collection_json(&collection_json)?;

    let last_updated_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before the Unix epoch")?
        .as_secs();

    let manifest = CRLiteCacheManifest {
        source,
        channel: DEFAULT_CRLITE_CHANNEL.to_string(),
        last_updated_unix_seconds,
        current_filter: Some(current_filter),
        deltas,
    };
    save_crlite_manifest(&manifest)?;
    Ok(manifest)
}

static CORRUPTION_REFRESH_ATTEMPTS: Mutex<Vec<SystemTime>> =
    Mutex::new(Vec::new());

pub fn should_rate_limit_refresh() -> bool {
    let mut guard = match CORRUPTION_REFRESH_ATTEMPTS.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let now = SystemTime::now();

    // Clean up timestamps older than 24 hours
    if let Some(day_ago) = now.checked_sub(Duration::from_secs(24 * 60 * 60)) {
        guard.retain(|&t| t > day_ago);
    }

    // Check daily limit of 10 attempts
    if guard.len() >= 10 {
        return true;
    }

    // Check 15-minute spacing limit
    if let Some(&last_time) = guard.last() {
        if let Ok(elapsed) = now.duration_since(last_time) {
            if elapsed < Duration::from_secs(15 * 60) {
                return true;
            }
        }
    }

    false
}

pub fn record_refresh_time() {
    let mut guard = match CORRUPTION_REFRESH_ATTEMPTS.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    guard.push(SystemTime::now());
}

static CRLITE_REFRESH_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

pub fn ensure_crlite_cache_ready_sync() -> Result<CRLiteCacheManifest> {
    let mut is_corrupted = false;

    if let Some(manifest) = load_crlite_manifest()? {
        if manifest.is_fresh(SystemTime::now(), DEFAULT_CRLITE_MAX_AGE) {
            match CachedCRLiteState::load_from_disk() {
                Ok(state) => return Ok(state.manifest),
                Err(e) => {
                    warn_fmt!(
                        "CRLite cache files are missing or corrupted, will attempt refresh: {e:#}"
                    );
                    is_corrupted = true;
                }
            }
        }
    }

    let _lock = CRLITE_REFRESH_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|e| anyhow::anyhow!("CRLITE_REFRESH_MUTEX poisoned: {e}"))?;

    if let Some(manifest) = load_crlite_manifest()? {
        if manifest.is_fresh(SystemTime::now(), DEFAULT_CRLITE_MAX_AGE) {
            if let Ok(state) = CachedCRLiteState::load_from_disk() {
                return Ok(state.manifest);
            }
            is_corrupted = true;
        }
    }

    if is_corrupted && should_rate_limit_refresh() {
        return Err(anyhow::anyhow!(
            "CRLite cache is corrupted or missing, and refresh is rate-limited"
        ));
    }

    if is_corrupted {
        record_refresh_time();
    }

    refresh_crlite_cache_sync()
}

fn parse_crlite_timestamp_entries(
    leaf_der: &[u8],
) -> Result<Vec<CRLiteTimestamp>> {
    let (_, cert) = parse_x509_certificate(leaf_der).map_err(|error| {
        anyhow::anyhow!("Failed to parse X.509 certificate: {error:?}")
    })?;

    let mut timestamps = Vec::new();
    for extension in cert.tbs_certificate.extensions() {
        if let ParsedExtension::SCT(scts) = extension.parsed_extension() {
            for sct in scts {
                timestamps.push(CRLiteTimestamp {
                    log_id: *sct.id.key_id,
                    timestamp: sct.timestamp,
                });
            }
        }
    }
    Ok(timestamps)
}

fn end_entity_serial_number(
    end_entity: &CertificateDer<'_>,
) -> Result<Vec<u8>> {
    let (_, cert) =
        parse_x509_certificate(end_entity.as_ref()).map_err(|error| {
            anyhow::anyhow!("Failed to parse end-entity certificate: {error:?}")
        })?;
    Ok(cert.tbs_certificate.raw_serial().to_vec())
}

fn issuer_spki_bytes(intermediates: &[CertificateDer<'_>]) -> Result<Vec<u8>> {
    let issuer = intermediates.first().ok_or_else(|| {
        anyhow::anyhow!(
            "TLS peer did not provide an issuer certificate for CRLite"
        )
    })?;
    let (_, cert) =
        parse_x509_certificate(issuer.as_ref()).map_err(|error| {
            anyhow::anyhow!("Failed to parse issuer certificate: {error:?}")
        })?;
    Ok(cert.tbs_certificate.subject_pki.raw.to_vec())
}

static CRLITE_STATE_CACHE: OnceLock<RwLock<Option<Arc<CachedCRLiteState>>>> =
    OnceLock::new();

fn get_crlite_state_cache() -> &'static RwLock<Option<Arc<CachedCRLiteState>>> {
    CRLITE_STATE_CACHE.get_or_init(|| RwLock::new(None))
}

/// This funciton is only used for tests, but can't use `#[cfg(test)]` because it is used by a different crate's test.
pub fn clear_in_memory_cache() {
    if let Ok(mut guard) = get_crlite_state_cache().write() {
        *guard = None;
    }
}

pub fn get_or_load_crlite_state() -> Result<Arc<CachedCRLiteState>> {
    let cache = get_crlite_state_cache();

    // 1. Read lock check
    {
        let guard = cache.read().map_err(|e| anyhow::anyhow!("CRLite state cache read lock poisoned: {e}"))?;
        if let Some(state) = &*guard {
            if let Ok(Some(manifest)) = load_crlite_manifest() {
                if manifest.last_updated_unix_seconds
                    == state.manifest.last_updated_unix_seconds
                {
                    return Ok(Arc::clone(state));
                }
            }
        }
    }

    // 2. Write lock check and load
    let mut guard = cache.write().map_err(|e| anyhow::anyhow!("CRLite state cache write lock poisoned: {e}"))?;
    if let Some(state) = &*guard {
        if let Ok(Some(manifest)) = load_crlite_manifest() {
            if manifest.last_updated_unix_seconds
                == state.manifest.last_updated_unix_seconds
            {
                return Ok(Arc::clone(state));
            }
        }
    }

    let new_state = Arc::new(CachedCRLiteState::load_from_disk()?);
    *guard = Some(Arc::clone(&new_state));
    Ok(new_state)
}

fn crlite_lookup_status_from_chain(
    end_entity: &CertificateDer<'_>,
    intermediates: &[CertificateDer<'_>],
) -> Result<CRLiteLookupStatus> {
    let _manifest = ensure_crlite_cache_ready_sync()?;
    let timestamps = parse_crlite_timestamp_entries(end_entity.as_ref())?;
    let serial_number = end_entity_serial_number(end_entity)?;
    let issuer_spki = issuer_spki_bytes(intermediates)?;
    let state = get_or_load_crlite_state()?;
    Ok(state.revocation_status(&issuer_spki, &serial_number, &timestamps))
}

impl CRLiteVerifier {
    pub fn with_webpki_roots(root_store: RootCertStore) -> Result<Self> {
        let verifier = WebPkiServerVerifier::builder(Arc::new(root_store))
            .build()
            .map_err(|error| {
                anyhow::anyhow!("Failed to build rustls verifier: {error:?}")
            })?;
        Ok(Self { inner: verifier })
    }

    pub fn with_platform_verifier() -> Result<Self> {
        let provider = rustls::crypto::CryptoProvider::get_default()
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(rustls::crypto::aws_lc_rs::default_provider())
            });
        let verifier = rustls_platform_verifier::Verifier::new(provider)
            .map_err(|error| {
                anyhow::anyhow!("Failed to build platform verifier: {error:?}")
            })?;
        Ok(Self {
            inner: Arc::new(verifier),
        })
    }
}

impl ServerCertVerifier for CRLiteVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, Error> {
        self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;

        match crlite_lookup_status_from_chain(end_entity, intermediates) {
            Ok(CRLiteLookupStatus::Good) => Ok(ServerCertVerified::assertion()),
            Ok(CRLiteLookupStatus::Revoked) => {
                Err(Error::InvalidCertificate(CertificateError::Revoked))
            }
            Ok(
                CRLiteLookupStatus::NoFilter
                | CRLiteLookupStatus::NotCovered
                | CRLiteLookupStatus::NotEnrolled,
            ) => Ok(ServerCertVerified::assertion()),
            Err(_) => Ok(ServerCertVerified::assertion()),
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Returns the CRLite cache directory, creating it if needed.
pub fn get_crlite_cache_dir() -> Result<PathBuf> {
    let mut dir = get_storage_dir()?.join(CRLITE_CACHE_DIR_NAME);
    if let Some(test_name) = crate::testing::try_get_current_test_name() {
        dir.push(test_name);
    }
    std::fs::create_dir_all(&dir).with_context(|| {
        format!("Failed to create CRLite cache directory {}", dir.display())
    })?;
    Ok(dir)
}

/// Returns the on-disk path for the CRLite manifest file.
pub fn get_crlite_manifest_path() -> Result<PathBuf> {
    Ok(get_crlite_cache_dir()?.join(CRLITE_MANIFEST_FILE_NAME))
}

/// Returns the on-disk path for the cached `CRLite` collection JSON.
pub fn get_crlite_collection_path() -> Result<PathBuf> {
    Ok(get_crlite_cache_dir()?.join(CRLITE_COLLECTION_FILE_NAME))
}

/// Loads the persisted CRLite manifest, if one exists.
pub fn load_crlite_manifest() -> Result<Option<CRLiteCacheManifest>> {
    let path = get_crlite_manifest_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&path).with_context(|| {
        format!("Failed to read CRLite manifest {}", path.display())
    })?;
    let manifest = serde_json::from_str(&json).with_context(|| {
        format!("Failed to parse CRLite manifest {}", path.display())
    })?;
    Ok(Some(manifest))
}

/// Writes the CRLite manifest atomically.
pub fn save_crlite_manifest(manifest: &CRLiteCacheManifest) -> Result<()> {
    let path = get_crlite_manifest_path()?;
    write_manifest_to_path(&path, manifest)
}

/// Loads the cached `CRLite` collection JSON, if one exists.
pub fn load_crlite_collection_json() -> Result<Option<String>> {
    let path = get_crlite_collection_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&path).with_context(|| {
        format!("Failed to read CRLite collection JSON {}", path.display())
    })?;
    Ok(Some(json))
}

/// Writes the cached `CRLite` collection JSON atomically.
pub fn save_crlite_collection_json(json: &str) -> Result<()> {
    let path = get_crlite_collection_path()?;
    write_text_to_path(&path, json, "CRLite collection JSON")
}

fn write_manifest_to_path(
    path: &Path,
    manifest: &CRLiteCacheManifest,
) -> Result<()> {
    let json = serde_json::to_string_pretty(manifest)
        .context("Failed to serialize CRLite manifest")?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, json).with_context(|| {
        format!(
            "Failed to write temporary CRLite manifest {}",
            tmp_path.display()
        )
    })?;
    std::fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to replace CRLite manifest {} with {}",
            path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}

fn write_text_to_path(path: &Path, text: &str, label: &str) -> Result<()> {
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, text).with_context(|| {
        format!("Failed to write temporary {label} {}", tmp_path.display())
    })?;
    std::fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to replace {label} {} with {}",
            path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}

/// Loads and parses a CRLite filter file from disk.
pub fn load_crlite_filter_from_path(path: &Path) -> Result<LoadedCRLiteFilter> {
    let bytes = std::fs::read(path).with_context(|| {
        format!("Failed to read CRLite filter {}", path.display())
    })?;
    load_crlite_filter_from_bytes(bytes)
}

/// Loads and parses a CRLite filter from raw bytes.
pub fn load_crlite_filter_from_bytes(
    bytes: Vec<u8>,
) -> Result<LoadedCRLiteFilter> {
    let clubcard = CRLiteClubcard::from_bytes(&bytes).map_err(|error| {
        anyhow::anyhow!("Failed to parse CRLite clubcard bytes: {error:?}")
    })?;
    Ok(LoadedCRLiteFilter { bytes, clubcard })
}

/// Loads one of the embedded CRLite test fixtures that ship with the repo.
pub fn load_embedded_test_filter(name: &str) -> Result<LoadedCRLiteFilter> {
    let key = format!("fixtures/test_crlite_filters/{name}");
    let bytes = crate::https::get_https_data(&key)
        .with_context(|| format!("Missing embedded CRLite fixture {key}"))?;
    load_crlite_filter_from_bytes(bytes)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use crate::bin2hex;
    use std::fs;
    use std::time::Duration;
    use std::time::SystemTime;

    use anyhow::Context;
    use rustls::RootCertStore;
    use rustls::client::danger::ServerCertVerifier;
    use rustls::pki_types::CertificateDer;
    use rustls::pki_types::ServerName;
    use rustls::pki_types::UnixTime;
    use rustls::pki_types::pem::PemObject;
    use std::io::Cursor;

    use super::{
        CRLiteArtifact, CRLiteAttachment, CRLiteCacheManifest,
        CRLiteCollectionRecord, CRLiteCollectionResponse, CRLiteLookupStatus,
        CRLiteRecordDetails, CRLiteSource, CRLiteSourceKind, CRLiteVerifier,
        CachedCRLiteState, UNIX_EPOCH, end_entity_serial_number,
        get_crlite_cache_dir, issuer_spki_bytes, load_crlite_manifest,
        parse_crlite_timestamp_entries, save_crlite_manifest,
        select_current_records, validate_relative_artifact_path,
        write_manifest_to_path,
    };

    fn sample_manifest() -> CRLiteCacheManifest {
        CRLiteCacheManifest {
            source: CRLiteSource {
                kind: CRLiteSourceKind::Mozilla,
                collection_url: "https://example.invalid/records".to_string(),
                attachments_base_url: "https://example.invalid/attachments/"
                    .to_string(),
            },
            channel: "default".to_string(),
            last_updated_unix_seconds: 1_700_000_000,
            current_filter: Some(CRLiteArtifact {
                relative_path: "filters/current.filter".to_string(),
                sha256_hex: "abc123".to_string(),
                size_bytes: 42,
                effective_timestamp: Some(1_700_000_000_000),
                is_incremental: false,
            }),
            deltas: vec![CRLiteArtifact {
                relative_path: "filters/next.delta".to_string(),
                sha256_hex: "def456".to_string(),
                size_bytes: 11,
                effective_timestamp: Some(1_700_000_600_000),
                is_incremental: true,
            }],
        }
    }

    #[crate::ctb_test]
    fn embedded_fixture_bytes_are_available() {
        let fixture = crate::https::get_https_data(
            "fixtures/test_crlite_filters/valid.example.com.pem",
        )
        .context("failed to get embedded CRLite PEM fixture")
        .unwrap();
        assert!(!fixture.is_empty(), "fixture should not be empty");
    }

    #[crate::ctb_test]
    fn manifest_freshness_respects_age_limit() {
        let manifest = sample_manifest();
        let now = UNIX_EPOCH + Duration::from_secs(1_700_000_000 + 60);
        assert!(manifest.is_fresh(now, Duration::from_secs(120)));
        assert!(!manifest.is_fresh(now, Duration::from_secs(30)));
    }

    #[crate::ctb_test]
    fn can_parse_embedded_full_filter_fixture() {
        let loaded = super::load_embedded_test_filter("20200101-0-filter")
            .context("failed to parse embedded CRLite full filter")
            .unwrap();
        assert!(!loaded.bytes.is_empty(), "full filter bytes should exist");
    }

    #[crate::ctb_test]
    fn can_parse_embedded_delta_filter_fixture() {
        let loaded =
            super::load_embedded_test_filter("20200101-1-filter.delta")
                .context("failed to parse embedded CRLite delta filter")
                .unwrap();
        assert!(!loaded.bytes.is_empty(), "delta filter bytes should exist");
    }

    fn load_pem_certificate(name: &str) -> CertificateDer<'static> {
        let key = format!("fixtures/test_crlite_filters/{name}");
        let bytes = crate::https::get_https_data(&key)
            .with_context(|| format!("Missing embedded CRLite fixture {key}"))
            .unwrap();
        let mut cursor = Cursor::new(bytes);
        CertificateDer::pem_reader_iter(&mut cursor)
            .next()
            .unwrap()
            .unwrap()
    }

    fn load_fixture_bytes(name: &str) -> Vec<u8> {
        let key = format!("fixtures/test_crlite_filters/{name}");
        crate::https::get_https_data(&key)
            .with_context(|| format!("Missing embedded CRLite fixture {key}"))
            .unwrap()
    }

    fn seed_cached_filter_state() {
        seed_cached_filter_state_with_deltas(true);
    }

    fn fixture_verification_time() -> UnixTime {
        UnixTime::since_unix_epoch(Duration::from_secs(1_577_836_800))
    }

    fn seed_cached_filter_state_with_deltas(include_delta: bool) {
        let cache_dir = get_crlite_cache_dir().unwrap();
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).unwrap();
        }
        let filter_dir = cache_dir.join("default");
        fs::create_dir_all(&filter_dir).unwrap();

        let full_name = "20200101-0-filter";
        let delta_name = "20200101-1-filter.delta";
        let full_bytes = load_fixture_bytes(full_name);
        let delta_bytes = load_fixture_bytes(delta_name);
        fs::write(filter_dir.join(full_name), &full_bytes).unwrap();
        if include_delta {
            fs::write(filter_dir.join(delta_name), &delta_bytes).unwrap();
        }

        static NEXT_TIME: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + NEXT_TIME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let manifest = CRLiteCacheManifest {
            source: CRLiteSource::mozilla(),
            channel: "default".to_string(),
            last_updated_unix_seconds: now,
            current_filter: Some(CRLiteArtifact {
                relative_path: format!("default/{full_name}"),
                sha256_hex: bin2hex(
                    <sha2::Sha256 as sha2::Digest>::digest(&full_bytes),
                ),
                size_bytes: u64::try_from(full_bytes.len()).unwrap(),
                effective_timestamp: Some(1_577_836_800_000),
                is_incremental: false,
            }),
            deltas: if include_delta {
                vec![CRLiteArtifact {
                    relative_path: format!("default/{delta_name}"),
                    sha256_hex: bin2hex(
                        <sha2::Sha256 as sha2::Digest>::digest(&delta_bytes),
                    ),
                    size_bytes: u64::try_from(delta_bytes.len()).unwrap(),
                    effective_timestamp: Some(1_577_923_200_000),
                    is_incremental: true,
                }]
            } else {
                vec![]
            },
        };
        save_crlite_manifest(&manifest).unwrap();
        super::clear_in_memory_cache();
    }

    #[crate::ctb_test]
    fn parses_scts_from_fixture_certificate() {
        let cert = load_pem_certificate("valid.example.com.pem");
        let timestamps = parse_crlite_timestamp_entries(cert.as_ref()).unwrap();
        assert!(!timestamps.is_empty());
    }

    #[crate::ctb_test]
    fn reports_missing_scts_for_no_sct_fixture() {
        let cert = load_pem_certificate("revoked-no-sct.example.com.pem");
        let timestamps = parse_crlite_timestamp_entries(cert.as_ref()).unwrap();
        assert!(timestamps.is_empty());
    }

    #[crate::ctb_test]
    fn extracts_serial_and_issuer_spki_from_fixture_chain() {
        let leaf = load_pem_certificate("valid.example.com.pem");
        let issuer = load_pem_certificate("int.pem");
        let serial = end_entity_serial_number(&leaf).unwrap();
        let issuer_spki = issuer_spki_bytes(&[issuer]).unwrap();
        assert!(!serial.is_empty());
        assert!(!issuer_spki.is_empty());
    }

    #[crate::ctb_test]
    fn verifier_accepts_valid_fixture_and_rejects_revoked_fixture() {
        seed_cached_filter_state();
        assert!(load_crlite_manifest().unwrap().is_some());

        let mut root_store = RootCertStore::empty();
        root_store.add(load_pem_certificate("ca.pem")).unwrap();
        let verifier = CRLiteVerifier::with_webpki_roots(root_store).unwrap();
        let issuer = load_pem_certificate("int.pem");
        let server_name = ServerName::try_from("valid.example.com")
            .unwrap()
            .to_owned();

        let valid = verifier.verify_server_cert(
            &load_pem_certificate("valid.example.com.pem"),
            std::slice::from_ref(&issuer),
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(valid.is_ok());

        let revoked = verifier.verify_server_cert(
            &load_pem_certificate("revoked.example.com.pem"),
            &[issuer],
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(revoked.is_err());
    }

    #[crate::ctb_test]
    fn verifier_accepts_not_covered_fixture() {
        seed_cached_filter_state();

        let mut root_store = RootCertStore::empty();
        root_store.add(load_pem_certificate("ca.pem")).unwrap();
        let verifier = CRLiteVerifier::with_webpki_roots(root_store).unwrap();
        let issuer = load_pem_certificate("int.pem");
        let server_name = ServerName::try_from("not-covered.example.com")
            .unwrap()
            .to_owned();

        let result = verifier.verify_server_cert(
            &load_pem_certificate("not-covered.example.com.pem"),
            &[issuer],
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(result.is_ok());
    }

    #[crate::ctb_test]
    fn verifier_accepts_not_covered_and_delta_without_delta() {
        seed_cached_filter_state();

        let mut root_store = RootCertStore::empty();
        root_store.add(load_pem_certificate("ca.pem")).unwrap();
        let verifier = CRLiteVerifier::with_webpki_roots(root_store).unwrap();
        let issuer = load_pem_certificate("int.pem");

        // 1. Not covered cert is accepted
        let server_name = ServerName::try_from("not-covered.example.com")
            .unwrap()
            .to_owned();
        let result = verifier.verify_server_cert(
            &load_pem_certificate("not-covered.example.com.pem"),
            &[issuer.clone()],
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(result.is_ok());

        // 2. Revoked in delta but without delta filter is accepted
        seed_cached_filter_state_with_deltas(false);
        let server_name = ServerName::try_from("revoked-in-delta.example.com")
            .unwrap()
            .to_owned();
        let result = verifier.verify_server_cert(
            &load_pem_certificate("revoked-in-delta.example.com.pem"),
            &[issuer],
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(result.is_ok());
    }

    #[crate::ctb_test]
    fn revoked_in_delta_requires_delta_filter() {
        let issuer = load_pem_certificate("int.pem");
        let server_name = ServerName::try_from("revoked-in-delta.example.com")
            .unwrap()
            .to_owned();
        let mut root_store = RootCertStore::empty();
        root_store.add(load_pem_certificate("ca.pem")).unwrap();
        let verifier = CRLiteVerifier::with_webpki_roots(root_store).unwrap();

        seed_cached_filter_state_with_deltas(false);
        let without_delta = verifier.verify_server_cert(
            &load_pem_certificate("revoked-in-delta.example.com.pem"),
            std::slice::from_ref(&issuer),
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(without_delta.is_ok());

        seed_cached_filter_state_with_deltas(true);
        let with_delta = verifier.verify_server_cert(
            &load_pem_certificate("revoked-in-delta.example.com.pem"),
            &[issuer],
            &server_name,
            &[],
            fixture_verification_time(),
        );
        assert!(with_delta.is_err());
    }

    #[crate::ctb_test]
    fn manifest_round_trip_preserves_fields() {
        let root = std::env::temp_dir()
            .join(format!("ctb-crlite-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root)
            .with_context(|| {
                format!("Failed to create temp dir {}", root.display())
            })
            .unwrap();
        let path = root.join("manifest.json");
        let manifest = sample_manifest();

        write_manifest_to_path(&path, &manifest).unwrap();

        let json = std::fs::read_to_string(&path)
            .with_context(|| {
                format!("Failed to read test manifest {}", path.display())
            })
            .unwrap();
        let decoded: CRLiteCacheManifest = serde_json::from_str(&json)
            .context("Failed to decode round-tripped CRLite manifest")
            .unwrap();
        assert_eq!(decoded, manifest);

        std::fs::remove_dir_all(&root)
            .with_context(|| {
                format!("Failed to remove temp dir {}", root.display())
            })
            .unwrap();
    }

    #[crate::ctb_test]
    fn validates_relative_artifact_paths() {
        assert!(validate_relative_artifact_path("default/file.filter").is_ok());
        assert!(validate_relative_artifact_path("../file.filter").is_err());
    }

    #[crate::ctb_test]
    fn no_filters_in_channel_is_rejected() {
        let response = CRLiteCollectionResponse {
            data: vec![CRLiteCollectionRecord {
                id: "other-channel-full".to_string(),
                parent: None,
                channel: Some("priority".to_string()),
                details: CRLiteRecordDetails {
                    name: "20260601-full".to_string(),
                },
                attachment: CRLiteAttachment {
                    hash: "a".repeat(64),
                    size: 1,
                    filename: "20260601.filter".to_string(),
                    location: "full.filter".to_string(),
                    mimetype: None,
                },
                incremental: false,
                effective_timestamp: Some(10),
            }],
        };

        let result = select_current_records(&response, "default");
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn incremental_only_records_are_rejected() {
        let response = CRLiteCollectionResponse {
            data: vec![CRLiteCollectionRecord {
                id: "delta-only".to_string(),
                parent: Some("missing-parent".to_string()),
                channel: Some("default".to_string()),
                details: CRLiteRecordDetails {
                    name: "20260601-diff".to_string(),
                },
                attachment: CRLiteAttachment {
                    hash: "a".repeat(64),
                    size: 1,
                    filename: "20260601.delta".to_string(),
                    location: "delta.filter".to_string(),
                    mimetype: None,
                },
                incremental: true,
                effective_timestamp: Some(10),
            }],
        };

        let result = select_current_records(&response, "default");
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn selects_latest_full_filter_and_newer_deltas() {
        let response = CRLiteCollectionResponse {
            data: vec![
                CRLiteCollectionRecord {
                    id: "full-old".to_string(),
                    parent: None,
                    channel: Some("default".to_string()),
                    details: CRLiteRecordDetails {
                        name: "20260601-full".to_string(),
                    },
                    attachment: CRLiteAttachment {
                        hash: "a".repeat(64),
                        size: 1,
                        filename: "20260601.filter".to_string(),
                        location: "old.filter".to_string(),
                        mimetype: None,
                    },
                    incremental: false,
                    effective_timestamp: Some(10),
                },
                CRLiteCollectionRecord {
                    id: "full-new".to_string(),
                    parent: None,
                    channel: Some("default".to_string()),
                    details: CRLiteRecordDetails {
                        name: "20260602-full".to_string(),
                    },
                    attachment: CRLiteAttachment {
                        hash: "b".repeat(64),
                        size: 1,
                        filename: "20260602.filter".to_string(),
                        location: "new.filter".to_string(),
                        mimetype: None,
                    },
                    incremental: false,
                    effective_timestamp: Some(20),
                },
                CRLiteCollectionRecord {
                    id: "delta-before".to_string(),
                    parent: None,
                    channel: Some("default".to_string()),
                    details: CRLiteRecordDetails {
                        name: "20260601-diff".to_string(),
                    },
                    attachment: CRLiteAttachment {
                        hash: "c".repeat(64),
                        size: 1,
                        filename: "20260601.delta".to_string(),
                        location: "before.delta".to_string(),
                        mimetype: None,
                    },
                    incremental: true,
                    effective_timestamp: Some(15),
                },
                CRLiteCollectionRecord {
                    id: "delta-after".to_string(),
                    parent: None,
                    channel: Some("default".to_string()),
                    details: CRLiteRecordDetails {
                        name: "20260603-diff".to_string(),
                    },
                    attachment: CRLiteAttachment {
                        hash: "d".repeat(64),
                        size: 1,
                        filename: "20260603.delta".to_string(),
                        location: "after.delta".to_string(),
                        mimetype: None,
                    },
                    incremental: true,
                    effective_timestamp: Some(25),
                },
            ],
        };

        let (full, deltas) =
            select_current_records(&response, "default").unwrap();
        assert_eq!(full.id, "full-new");
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas.first().unwrap().id, "delta-after");
    }

    #[crate::ctb_test]
    fn coverage_requirement_can_force_not_covered_status() {
        seed_cached_filter_state();

        let state = CachedCRLiteState::load_from_disk().unwrap();
        let leaf = load_pem_certificate("valid.example.com.pem");
        let issuer = load_pem_certificate("int.pem");
        let timestamps = parse_crlite_timestamp_entries(leaf.as_ref()).unwrap();
        let serial = end_entity_serial_number(&leaf).unwrap();
        let issuer_spki = issuer_spki_bytes(&[issuer]).unwrap();

        let status = state.revocation_status_with_coverage_requirement(
            &issuer_spki,
            &serial,
            &timestamps,
            100,
        );
        assert_eq!(status, CRLiteLookupStatus::NotCovered);
    }

    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        if items.is_empty() {
            return vec![vec![]];
        }
        let mut result = Vec::new();
        for i in 0..items.len() {
            let mut remaining = items.to_vec();
            let current = remaining.remove(i);
            for sub in permutations(&remaining) {
                let mut p = vec![current.clone()];
                p.extend(sub);
                result.push(p);
            }
        }
        result
    }

    #[crate::ctb_test]
    fn test_regenerate_fixtures() {
        use sha2::Digest;

        // Load certificates
        let int_pem = load_pem_certificate("int.pem");
        let valid_pem = load_pem_certificate("valid.example.com.pem");
        let revoked_pem = load_pem_certificate("revoked.example.com.pem");
        let revoked_no_sct_pem =
            load_pem_certificate("revoked-no-sct.example.com.pem");
        let revoked_in_delta_pem =
            load_pem_certificate("revoked-in-delta.example.com.pem");

        // Compute issuer SPKI hash
        let issuer_spki = issuer_spki_bytes(&[int_pem]).unwrap();
        let issuer_spki_hash: [u8; 32] =
            sha2::Sha256::digest(&issuer_spki).into();

        // Get serials
        let valid_serial = end_entity_serial_number(&valid_pem).unwrap();
        let revoked_serial = end_entity_serial_number(&revoked_pem).unwrap();
        let revoked_no_sct_serial =
            end_entity_serial_number(&revoked_no_sct_pem).unwrap();
        let revoked_in_delta_serial =
            end_entity_serial_number(&revoked_in_delta_pem).unwrap();

        // Build L1: due to L1 duplicate appends in make_filters.sh, revoked-no-sct is added twice.
        let l1_known = vec![
            valid_serial.clone(),
            revoked_serial.clone(),
            revoked_no_sct_serial.clone(),
            revoked_no_sct_serial.clone(),
        ];
        let l1_approx_revoked = vec![
            revoked_serial.clone(),
            revoked_no_sct_serial.clone(),
            revoked_no_sct_serial.clone(),
        ];
        let l1_exact_revoked = vec![revoked_serial.clone()];
        let fixture_l1 = load_fixture_bytes("20200101-0-filter");

        // Try all permutations of known serials to match the exact pivot order in rust-create-cascade
        // Note: rust-create-cascade can be found at https://github.com/mozilla/crlite.git rev 1b50fb4cc9b7e61c6c2badb7d157b037d467e7c9
        let mut found_l1 = None;
        for perm in permutations(&l1_known) {
            for _ in 0..5000 {
                let gen_l1 = build_clubcard_in_memory(
                    issuer_spki_hash,
                    &perm,
                    &l1_approx_revoked,
                    &l1_exact_revoked,
                );
                if gen_l1 == fixture_l1 {
                    found_l1 = Some(gen_l1);
                    break;
                }
            }
            if found_l1.is_some() {
                break;
            }
        }
        let generated_l1 = found_l1.unwrap_or_else(|| {
            build_clubcard_in_memory(
                issuer_spki_hash,
                &l1_known,
                &l1_approx_revoked,
                &l1_exact_revoked,
            )
        });
        assert_eq!(
            generated_l1, fixture_l1,
            "Generated L1 filter does not match fixture under any permutation"
        );

        // Build L2 Delta
        let l2_known = vec![
            valid_serial.clone(),
            revoked_serial.clone(),
            revoked_in_delta_serial.clone(),
        ];
        let l2_delta_revoked = vec![revoked_in_delta_serial.clone()];
        let fixture_l2_delta = load_fixture_bytes("20200101-1-filter.delta");

        let mut found_l2 = None;
        for perm in permutations(&l2_known) {
            for _ in 0..5000 {
                let gen_l2 = build_clubcard_in_memory(
                    issuer_spki_hash,
                    &perm,
                    &l2_delta_revoked,
                    &l2_delta_revoked,
                );
                if gen_l2 == fixture_l2_delta {
                    found_l2 = Some(gen_l2);
                    break;
                }
            }
            if found_l2.is_some() {
                break;
            }
        }
        let generated_l2_delta = found_l2.unwrap_or_else(|| {
            build_clubcard_in_memory(
                issuer_spki_hash,
                &l2_known,
                &l2_delta_revoked,
                &l2_delta_revoked,
            )
        });
        assert_eq!(
            generated_l2_delta, fixture_l2_delta,
            "Generated L2 delta filter does not match fixture under any permutation"
        );
    }

    fn build_clubcard_in_memory(
        issuer_spki_hash: [u8; 32],
        known_serials: &[Vec<u8>],
        approx_revoked_serials: &[Vec<u8>],
        exact_revoked_serials: &[Vec<u8>],
    ) -> Vec<u8> {
        use clubcard::builder::ClubcardBuilder;
        use clubcard_crlite::builder::CRLiteBuilderItem;
        use clubcard_crlite::{CRLiteClubcard, CRLiteCoverage, CRLiteQuery};
        use std::collections::HashSet;
        use std::io::Cursor;

        let approx_revoked_set: HashSet<&Vec<u8>> =
            approx_revoked_serials.iter().collect();
        let exact_revoked_set: HashSet<&Vec<u8>> =
            exact_revoked_serials.iter().collect();

        // Initialize builder
        let mut builder = ClubcardBuilder::<4, CRLiteBuilderItem>::new();

        // 1. Include phase: approx ribbon
        let mut approx_builder = builder.new_approx_builder(&issuer_spki_hash);
        let mut universe_size = 0;
        let mut temp_approx_revoked_set = approx_revoked_set.clone();
        for serial in known_serials {
            universe_size += 1;
            if temp_approx_revoked_set.contains(serial) {
                let item = CRLiteBuilderItem::revoked(
                    issuer_spki_hash,
                    serial.clone(),
                );
                approx_builder.insert(item);
                temp_approx_revoked_set.remove(serial);
            }
        }
        approx_builder.set_universe_size(universe_size);
        let approx_ribbon = approx_builder.into();
        builder.collect_approx_ribbons(vec![approx_ribbon]);

        // 2. Exclude phase: exact ribbon
        let mut exact_builder =
            ClubcardBuilder::new_exact_builder(&builder, &issuer_spki_hash);
        for serial in known_serials {
            let item = if exact_revoked_set.contains(serial) {
                CRLiteBuilderItem::revoked(issuer_spki_hash, serial.clone())
            } else {
                CRLiteBuilderItem::not_revoked(issuer_spki_hash, serial.clone())
            };
            exact_builder.insert(item);
        }
        let exact_ribbon = exact_builder.into();
        builder.collect_exact_ribbons(vec![exact_ribbon]);

        // 3. Finalize and Build
        let ct_logs_json = r#"[{
 "LogID": "VCIlmPM9NkgFQtrs4Oa5TeFcDu6MWRTKSNdePEhOgD8=",
 "MinTimestamp": 0,
 "MaxTimestamp": 10000086313599,
 "MMD": 86400,
 "MinEntry": 0
 }]"#;
        let coverage = CRLiteCoverage::from_mozilla_ct_logs_json(Cursor::new(
            ct_logs_json,
        ));
        let clubcard: CRLiteClubcard =
            builder.build::<CRLiteQuery>(coverage, ()).into();

        clubcard.to_bytes().unwrap()
    }
}

/*

Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.

*/
