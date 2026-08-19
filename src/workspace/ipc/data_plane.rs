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

#[cfg(unix)]
use crate::error::Error;
#[cfg(unix)]
use crate::multiplex::session::Session;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub use ctb_utilities::shared_memory;
#[cfg(unix)]
use ctb_utilities::shared_memory::{ProducerBlob, SharedBlobDescriptor};

// ============================================================================
// Session-layer FD transfer helpers
// ============================================================================

/// Send a blob's file descriptor via the session layer.
///
/// For blobs backed by `UnixFd`, this sends the FD using `SCM_RIGHTS`.
/// For other blob types (`FilePath`, Named), this is a no-op since they don't
/// require out-of-band FD transfer.
///
/// # Arguments
/// * `session` - The session to use for FD transfer
/// * `descriptor` - The blob descriptor containing the FD to send
///
/// # Returns
/// Returns `Ok(())` on success, or an error if FD transfer fails.
#[cfg(unix)]
pub async fn send_blob_fd<S: Session + ?Sized>(
    session: &S,
    descriptor: &SharedBlobDescriptor,
) -> Result<(), Error> {
    match descriptor {
        SharedBlobDescriptor::UnixFd(fd) => session.send_fd(*fd).await,
        // No FD transfer needed for file path or named descriptors
        SharedBlobDescriptor::FilePath(_) | SharedBlobDescriptor::Named(_) => {
            Ok(())
        }
    }
}

/// Receive a blob's file descriptor via the session layer.
///
/// For blobs expected to be backed by `UnixFd`, this receives the FD using
/// `SCM_RIGHTS` and returns an updated descriptor with the local FD number.
///
/// # Arguments
/// * `session` - The session to use for FD transfer
/// * `descriptor` - The original blob descriptor (FD value will be replaced)
///
/// # Returns
/// Returns an updated `SharedBlobDescriptor` with the received FD, or the
/// original descriptor unchanged for non-FD blob types.
#[cfg(unix)]
pub async fn recv_blob_fd<S: Session + ?Sized>(
    session: &S,
    descriptor: &SharedBlobDescriptor,
) -> Result<SharedBlobDescriptor, Error> {
    match descriptor {
        SharedBlobDescriptor::UnixFd(_) => {
            // The original FD number is meaningless in the receiving process;
            // we need to receive the actual FD via SCM_RIGHTS
            let received_fd = session.recv_fd().await?;
            Ok(SharedBlobDescriptor::UnixFd(received_fd))
        }
        // No FD transfer needed for file path or named descriptors
        SharedBlobDescriptor::FilePath(p) => {
            Ok(SharedBlobDescriptor::FilePath(p.clone()))
        }
        SharedBlobDescriptor::Named(n) => {
            Ok(SharedBlobDescriptor::Named(n.clone()))
        }
    }
}

/// Send a producer blob with its FD via the session layer.
///
/// This combines sending the blob's metadata (via control plane messages) with
/// sending the underlying FD (via `SCM_RIGHTS` for Unix FD-backed blobs).
///
/// # Protocol
/// The caller should:
/// 1. Send a control message containing the `BlobToken` and `SharedBlobDescriptor`
/// 2. Call this function to send the FD out-of-band (for `UnixFd` descriptors)
///
/// # Arguments
/// * `session` - The session to use for FD transfer
/// * `blob` - The producer blob to send
///
/// # Returns
/// Returns `Ok(())` on success.
#[cfg(unix)]
pub async fn send_producer_blob_fd<S: Session>(
    session: &S,
    blob: &ProducerBlob,
) -> Result<(), Error> {
    send_blob_fd(session, &blob.descriptor).await
}
