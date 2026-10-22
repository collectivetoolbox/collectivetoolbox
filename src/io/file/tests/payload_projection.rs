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

//! Tests verifying payload projection retention over disk spans.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use ctb_utilities::*;

use ctb_formats_dcstring::{BinaryPayloadRef, DcDocumentNavigator, DcProjectionPolicy, SharedValueContext};
use ctb_io_file::{FileEntity, FileEntityWithPayload, MemoryPayloadSource};

#[crate::ctb_test]
fn file_payload_projection_retains_disk_span() -> Result<()> {
    let bytes = vec![0xAB; 131_072];
    let entity = FileEntity::from_vec(bytes.clone());
    let mut source = MemoryPayloadSource::new(bytes.clone())?;
    let mut file = tempfile::tempfile()?;
    entity.encode_to_writer_with_payload_and_context(&mut file, &mut source, &SharedValueContext::default())?;
    let mut navigator = DcDocumentNavigator::from_dcmm_file(file.try_clone()?, 0)?;
    let policy = DcProjectionPolicy::strict();
    let stub = navigator.next_sibling_stub(&policy)?.context("Missing file record")?;
    let record: FileEntityWithPayload = navigator.decode_frame(&stub, None, &policy)?;
    ensure!(record.entity == entity);
    let payload = record.payload.context("Missing file data")?;
    ensure!(matches!(payload.0, BinaryPayloadRef::FileSpan { .. }));
    ensure!(payload.0.materialize(Some(&mut file))?.as_ref() == bytes);
    Ok(())
}