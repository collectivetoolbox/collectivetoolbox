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

//! Projection policies, diagnostic actions, and forgiving defaults.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{
    Arc, BTreeMap, BinaryPayload, DEFAULT_MAX_PROJECTION_DEPTH,
    DcBinaryResolver, DcCharExt, DcExpressionEvaluator, DcMixedDecode,
    DcMstring, DcString, Digest, Duration, Integer, IpAddr, Ipv4Addr, Ipv6Addr,
    Natural, PathBuf, Read, Seek, SyntaxRuleResolver, SystemTime, UNIX_EPOCH,
};

/// Maximum binary payload size (in bytes) eagerly loaded into memory when
/// navigating or expanding frames from a seekable on-disk `.dcmm` file.
/// Larger binary regions are represented as [`BinaryPayloadRef::FileSpan`].
pub const INLINE_BINARY_THRESHOLD: u64 = 4096;

/// Default maximum eagerly parsed metadata size for a seekable encapsulation.
pub const MAX_BINARY_METADATA_BYTES: u64 = 1_048_576;

/// Default maximum checksum-metadata nesting depth allowed when parsing
/// textual binary encapsulation metadata.
pub const DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH: usize = 32;

/// Default maximum decoded payload size allowed when parsing a textual binary
/// encapsulation into memory.
pub const DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES: u64 = 67_108_864;

/// Action to take when a syntax or projection anomaly is encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticAction {
    /// Immediately abort parsing or projection and return an `Err`.
    Error,
    /// Emit a warning via `warn_fmt!` and continue with recovery/fallback.
    Warn,
    /// Emit an informational log via `log_fmt!` and continue with recovery/fallback.
    Log,
    /// Continue silently with recovery/fallback (diagnostics are still recorded in the arena/inspector).
    Ignore,
}

/// Version of the minimal trusted bootstrap kernel contract governing physical token
/// codecs (`DcTF`, `.dcmm` header, `Dc 203`..`Dc 204`), `SeedGraphIndex` active triple
/// resolution (`Dc 584`, `Dc 610`, `Dc 591`, `Dc 592`, `Dc 611`), `.Semantic:Pattern`
/// operators (`Dc 599`..`Dc 607`, `Dc 642`..`Dc 644`), and Dc-WASM binary lowering
/// (`Dc 647`..`Dc 1928`) + 32-byte mailbox ABI (`0..32`).
///
/// The foundational execution contract is explicitly specified and selectable by version
/// during bootstrap before any document or graph interpretation that depends on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BootstrapKernelVersion {
    /// Initial frozen minimal trusted bootstrap kernel specification (v1).
    V1 = 1,
}

impl Default for BootstrapKernelVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl BootstrapKernelVersion {
    /// The current supported bootstrap kernel version (`V1`).
    pub const CURRENT: Self = Self::V1;

    /// Returns the numeric version identifier (`1` for `V1`).
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::V1 => 1,
        }
    }

    /// Selects a [`BootstrapKernelVersion`] from an explicit numeric version identifier
    /// during bootstrap before interpretation that depends on it.
    ///
    /// # Errors
    /// Returns an error if `version` is not a supported bootstrap kernel version.
    pub fn try_from_u32(version: u32) -> Result<Self> {
        match version {
            1 => Ok(Self::V1),
            other => bail!(
                "Unsupported bootstrap kernel version {other}: only version {} (V1) is supported by this host",
                Self::CURRENT.as_u32()
            ),
        }
    }
}

/// Unified caller-specified policy governing how syntax errors, unknown tags,
/// deprecated constructs, malformed or missing fields, and resource budgets
/// are handled across Dc document parsing, validation, inspection, and typed
/// projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcProjectionPolicy {
    /// Framing or syntax rule violation in Stage 1 (e.g. unclosed frame, bad literal header).
    /// Unsupported checksum algorithms follow this action; recovery retains
    /// payload bytes without verifying those checksums.
    pub syntax_error: DiagnosticAction,
    /// Unknown Dc tag encountered inside a container (e.g. forward-compatible field or statement).
    pub unknown_field: DiagnosticAction,
    /// Deprecated syntax or legacy representation (e.g. `<prefer>` replacement).
    pub deprecated_syntax: DiagnosticAction,
    /// Type mismatch or domain error when projecting a known field onto its Rust type.
    pub malformed_field: DiagnosticAction,
    /// Missing required field when projecting a struct (uses [`DcForgivingDefault`] if non-`Error`).
    pub missing_required_field: DiagnosticAction,
    /// Maximum container or wrapper nesting depth allowed during arena construction and projection.
    pub max_depth: usize,
    /// Action taken when container or wrapper nesting exceeds [`Self::max_depth`].
    pub depth_limit_exceeded: DiagnosticAction,
    /// Maximum eagerly allocated metadata bytes per seekable binary header.
    /// Exceeding this resource budget is an error under every diagnostic policy.
    pub max_binary_metadata_bytes: u64,
    /// Maximum nested checksum-metadata levels allowed when parsing textual
    /// binary encapsulation metadata. Exceeding this resource budget is an
    /// error under every diagnostic policy.
    pub max_encapsulation_metadata_depth: usize,
    /// Maximum decoded payload bytes allowed when parsing a textual binary
    /// encapsulation into memory. Exceeding this resource budget is an error
    /// under every diagnostic policy.
    pub max_encapsulation_payload_bytes: u64,
    /// Optional diagnostic action when a document or nested document omits an
    /// explicit `Dc 641` System-block most-recent node ID pin and does not
    /// inherit one from an enclosing document or caller override.
    pub missing_system_graph_pin: Option<DiagnosticAction>,
    /// Optional designated baseline System-block snapshot node ID used when
    /// resolving legacy documents that omit `Dc 641`. When `None`, unpinned
    /// legacy documents resolve against the bundled System graph `HEAD` with
    /// explicit [`super::DcPinProvenance::UnpinnedLegacyFallback`] provenance.
    pub unpinned_fallback_most_recent_node_id: Option<u128>,
    /// Foundational bootstrap kernel contract version governing physical codecs,
    /// seed graph triple resolution, `.Semantic:Pattern` operators, and Dc-WASM lowering.
    pub kernel_version: BootstrapKernelVersion,
}

impl DcProjectionPolicy {
    /// Strict policy: errors on any syntax violation, unknown field tag,
    /// malformed field, missing required field, or exceeded nesting depth;
    /// warns on deprecated syntax.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            syntax_error: DiagnosticAction::Error,
            unknown_field: DiagnosticAction::Error,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Error,
            missing_required_field: DiagnosticAction::Error,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Error,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth:
                DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes:
                DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
            missing_system_graph_pin: None,
            unpinned_fallback_most_recent_node_id: None,
            kernel_version: BootstrapKernelVersion::CURRENT,
        }
    }

    /// Forgiving / best-effort policy: logs warnings and recovers at frame or
    /// field boundaries, projecting all valid fields and substituting fallback
    /// values for missing or damaged fields.
    #[must_use]
    pub const fn forgiving() -> Self {
        Self {
            syntax_error: DiagnosticAction::Warn,
            unknown_field: DiagnosticAction::Warn,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Warn,
            missing_required_field: DiagnosticAction::Warn,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Warn,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth:
                DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes:
                DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
            missing_system_graph_pin: None,
            unpinned_fallback_most_recent_node_id: None,
            kernel_version: BootstrapKernelVersion::CURRENT,
        }
    }

    /// Quiet forgiving policy for interactive source inspection (`DcSource::inspect`),
    /// recording diagnostics without emitting warning logs.
    #[must_use]
    pub const fn forgiving_quiet() -> Self {
        Self {
            syntax_error: DiagnosticAction::Ignore,
            unknown_field: DiagnosticAction::Ignore,
            deprecated_syntax: DiagnosticAction::Ignore,
            malformed_field: DiagnosticAction::Ignore,
            missing_required_field: DiagnosticAction::Ignore,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Ignore,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth:
                DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes:
                DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
            missing_system_graph_pin: None,
            unpinned_fallback_most_recent_node_id: None,
            kernel_version: BootstrapKernelVersion::CURRENT,
        }
    }

    /// Returns a copy of this policy with a custom maximum nesting depth and action.
    #[must_use]
    pub const fn with_max_depth(
        mut self,
        max_depth: usize,
        action: DiagnosticAction,
    ) -> Self {
        self.max_depth = max_depth;
        self.depth_limit_exceeded = action;
        self
    }

    /// Sets the eager metadata allocation budget for each seekable binary header.
    #[must_use]
    pub const fn with_max_binary_metadata_bytes(
        mut self,
        max_bytes: u64,
    ) -> Self {
        self.max_binary_metadata_bytes = max_bytes;
        self
    }

    /// Sets the checksum-metadata nesting budget for textual binary
    /// encapsulation metadata.
    #[must_use]
    pub const fn with_max_encapsulation_metadata_depth(
        mut self,
        max_depth: usize,
    ) -> Self {
        self.max_encapsulation_metadata_depth = max_depth;
        self
    }

    /// Sets the decoded payload allocation budget for textual binary
    /// encapsulations.
    #[must_use]
    pub const fn with_max_encapsulation_payload_bytes(
        mut self,
        max_bytes: u64,
    ) -> Self {
        self.max_encapsulation_payload_bytes = max_bytes;
        self
    }

    /// Configures the diagnostic action taken when a document omits an explicit
    /// `Dc 641` System-block most-recent node ID pin and does not inherit one
    /// from an enclosing scope or caller override.
    #[must_use]
    pub const fn with_missing_system_graph_pin(
        mut self,
        action: DiagnosticAction,
    ) -> Self {
        self.missing_system_graph_pin = Some(action);
        self
    }

    /// Configures the designated baseline System-block snapshot node ID used
    /// when resolving legacy documents that omit `Dc 641`.
    #[must_use]
    pub const fn with_unpinned_fallback_most_recent_node_id(
        mut self,
        node_id: u128,
    ) -> Self {
        self.unpinned_fallback_most_recent_node_id = Some(node_id);
        self
    }

    /// Selects the explicit [`BootstrapKernelVersion`] for this projection policy.
    #[must_use]
    pub const fn with_kernel_version(
        mut self,
        kernel_version: BootstrapKernelVersion,
    ) -> Self {
        self.kernel_version = kernel_version;
        self
    }

    /// Validates and selects a [`BootstrapKernelVersion`] by numeric identifier (`1` for `V1`)
    /// during bootstrap before any document or graph interpretation that depends on it.
    ///
    /// # Errors
    /// Returns an error if `version` is not a supported bootstrap kernel version.
    pub fn try_with_kernel_version_number(
        mut self,
        version: u32,
    ) -> Result<Self> {
        self.kernel_version = BootstrapKernelVersion::try_from_u32(version)?;
        Ok(self)
    }

    /// Returns true if all structural and projection anomalies are set to `Error`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self.syntax_error, DiagnosticAction::Error)
            && matches!(self.unknown_field, DiagnosticAction::Error)
            && matches!(self.malformed_field, DiagnosticAction::Error)
            && matches!(self.missing_required_field, DiagnosticAction::Error)
            && matches!(self.depth_limit_exceeded, DiagnosticAction::Error)
    }

    /// Returns true if any structural or projection anomaly is configured to recover/warn/ignore.
    #[must_use]
    pub const fn is_forgiving(&self) -> bool {
        !self.is_strict()
    }

    /// Applies the configured [`DiagnosticAction`] for an anomaly message.
    ///
    /// # Errors
    /// Returns an error if `action` is [`DiagnosticAction::Error`].
    pub fn handle(
        &self,
        action: DiagnosticAction,
        message: impl Into<String>,
    ) -> Result<()> {
        let msg = message.into();
        match action {
            DiagnosticAction::Error => bail!("{msg}"),
            DiagnosticAction::Warn => {
                warn_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Log => {
                log_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Ignore => Ok(()),
        }
    }
}

/// Fallback value provider used when projecting structs or fields under a
/// non-`Error` [`DcProjectionPolicy`] (`forgiving()`).
pub trait DcForgivingDefault: Sized {
    /// Produces a placeholder value when a required field or element is missing
    /// or malformed in forgiving mode.
    fn dc_forgiving_default() -> Self;
}

macro_rules! impl_forgiving_default_via_default {
    ($($t:ty),*) => {
        $(
            impl DcForgivingDefault for $t {
                fn dc_forgiving_default() -> Self {
                    Self::default()
                }
            }
        )*
    };
}

impl_forgiving_default_via_default!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    bool,
    String,
    PathBuf,
    Natural,
    Integer,
    BinaryPayload,
    DcMstring,
    DcString
);

impl DcForgivingDefault for Ipv4Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for Ipv6Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for IpAddr {
    fn dc_forgiving_default() -> Self {
        Self::V4(Ipv4Addr::UNSPECIFIED)
    }
}

impl DcForgivingDefault for SystemTime {
    fn dc_forgiving_default() -> Self {
        UNIX_EPOCH
    }
}

impl DcForgivingDefault for Duration {
    fn dc_forgiving_default() -> Self {
        Self::ZERO
    }
}

impl DcForgivingDefault for FormatId {
    fn dc_forgiving_default() -> Self {
        Self::Utf8
    }
}

impl<T> DcForgivingDefault for Option<T> {
    fn dc_forgiving_default() -> Self {
        None
    }
}

impl<T> DcForgivingDefault for Vec<T> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> DcForgivingDefault for BTreeMap<K, V> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Box<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Arc<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault, const N: usize> DcForgivingDefault for [T; N] {
    fn dc_forgiving_default() -> Self {
        std::array::from_fn(|_| T::dc_forgiving_default())
    }
}

impl<A: DcForgivingDefault, B: DcForgivingDefault> DcForgivingDefault
    for (A, B)
{
    fn dc_forgiving_default() -> Self {
        (A::dc_forgiving_default(), B::dc_forgiving_default())
    }
}

impl<A: DcForgivingDefault, B: DcForgivingDefault, C: DcForgivingDefault>
    DcForgivingDefault for (A, B, C)
{
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
        )
    }
}

impl<
    A: DcForgivingDefault,
    B: DcForgivingDefault,
    C: DcForgivingDefault,
    D: DcForgivingDefault,
> DcForgivingDefault for (A, B, C, D)
{
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
            D::dc_forgiving_default(),
        )
    }
}
