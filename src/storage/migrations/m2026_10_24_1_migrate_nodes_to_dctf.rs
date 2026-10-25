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

//! Database migration transcoding stored `Statements` and `System` node bodies
//! from legacy `DcUtf` (`f85`) to `DcTF` (`f641`) and recomputing SHA-256
//! checksums.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::migrations::DbSchemaType;
use sha2::{Digest, Sha256};
use turso::{Connection, Value};

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_10_24_1_migrate_nodes_to_dctf";
pub const DESCRIPTION: &str =
    "Transcode Statements and System node bodies from DcUtf to DcTF and update checksums";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare(
            "SELECT id, graph_id, data, checksum FROM nodes WHERE type IN (2, 3)",
        )
        .await?;
    let mut rows = stmt.query(()).await?;
    let mut updates = Vec::new();

    while let Some(row) = rows.next().await? {
        let Value::Blob(id_blob) = row.get_value(0)? else {
            bail!("Expected BLOB for nodes.id during DcTF migration");
        };
        let Value::Blob(graph_id_blob) = row.get_value(1)? else {
            bail!("Expected BLOB for nodes.graph_id during DcTF migration");
        };
        let Value::Blob(data_blob) = row.get_value(2)? else {
            bail!("Expected BLOB for nodes.data during DcTF migration");
        };
        match row.get_value(3)? {
            Value::Blob(checksum_blob) => {
                let actual_source_hash = Sha256::digest(&data_blob);
                ensure!(
                    actual_source_hash.as_slice() == checksum_blob.as_slice(),
                    "Stored node checksum mismatch prior to DcTF migration"
                );
            }
            Value::Null => {}
            other => {
                bail!(
                    "Unexpected value type for nodes.checksum during DcTF migration: {other:?}"
                );
            }
        }

        let dc_string = ctb_formats_dcstring::DcString::from_dcutf(data_blob)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Invalid DcUtf node body during DcTF migration: {e}"
                )
            })?;
        let new_data = dc_string.into_bytes();
        let new_checksum = Sha256::digest(&new_data).to_vec();
        updates.push((new_data, new_checksum, id_blob, graph_id_blob));
    }
    drop(rows);
    drop(stmt);

    for (new_data, new_checksum, id_blob, graph_id_blob) in updates {
        conn.execute(
            "UPDATE nodes SET data = ?1, checksum = ?2 WHERE id = ?3 AND graph_id = ?4",
            (new_data, new_checksum, id_blob, graph_id_blob),
        )
        .await?;
    }

    Ok(())
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
    use ctb_formats_dcstring::dctext::{dctext_to_dctf, dctext_to_dcutf};

    #[crate::ctb_test("tokio")]
    async fn test_migrate_nodes_to_dctf_transcodes_and_verifies_checksums() {
        let db = turso::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        conn.execute(
            "CREATE TABLE nodes (
                id BLOB NOT NULL,
                graph_id BLOB NOT NULL,
                type INTEGER NOT NULL,
                data BLOB NOT NULL,
                checksum BLOB,
                PRIMARY KEY (id, graph_id)
            )",
            (),
        )
        .await
        .unwrap();

        let graph_id = 1_u128.to_be_bytes().to_vec();
        let id_data = 10_u128.to_be_bytes().to_vec();
        let id_stmt = 11_u128.to_be_bytes().to_vec();
        let id_sys = 12_u128.to_be_bytes().to_vec();

        // Data node with raw bytes (including invalid DcUtf bytes) must remain untouched
        let raw_data = vec![0xFF_u8, 0x00, 0xFE, 0x42];
        let raw_data_checksum = Sha256::digest(&raw_data).to_vec();
        conn.execute(
            "INSERT INTO nodes (id, graph_id, type, data, checksum) VALUES (?1, ?2, 1, ?3, ?4)",
            (
                id_data.clone(),
                graph_id.clone(),
                raw_data.clone(),
                raw_data_checksum.clone(),
            ),
        )
        .await
        .unwrap();

        // Statements (type = 2) and System (type = 3) nodes stored as legacy DcUtf
        let stmt_text = b"stmt @1114112@ @L42@".to_vec();
        let stmt_dcutf = dctext_to_dcutf(stmt_text.clone()).unwrap();
        let stmt_dctf = dctext_to_dctf(stmt_text).unwrap();
        assert_ne!(stmt_dcutf, stmt_dctf);
        let stmt_old_checksum = Sha256::digest(&stmt_dcutf).to_vec();

        let sys_text = b"@1114409@@42@".to_vec();
        let sys_dcutf = dctext_to_dcutf(sys_text.clone()).unwrap();
        let sys_dctf = dctext_to_dctf(sys_text).unwrap();
        assert_ne!(sys_dcutf, sys_dctf);
        let sys_old_checksum = Sha256::digest(&sys_dcutf).to_vec();

        conn.execute(
            "INSERT INTO nodes (id, graph_id, type, data, checksum) VALUES (?1, ?2, 2, ?3, ?4)",
            (id_stmt.clone(), graph_id.clone(), stmt_dcutf, stmt_old_checksum),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, graph_id, type, data, checksum) VALUES (?1, ?2, 3, ?3, ?4)",
            (id_sys.clone(), graph_id.clone(), sys_dcutf, sys_old_checksum),
        )
        .await
        .unwrap();

        run_rust_migration(&conn).await.unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, data, checksum FROM nodes ORDER BY id ASC",
            )
            .await
            .unwrap();
        let mut rows = stmt.query(()).await.unwrap();

        let r0 = rows.next().await.unwrap().unwrap();
        assert_eq!(r0.get_value(0).unwrap(), Value::Blob(id_data));
        assert_eq!(r0.get_value(1).unwrap(), Value::Blob(raw_data));
        assert_eq!(r0.get_value(2).unwrap(), Value::Blob(raw_data_checksum));

        let r1 = rows.next().await.unwrap().unwrap();
        assert_eq!(r1.get_value(0).unwrap(), Value::Blob(id_stmt));
        assert_eq!(r1.get_value(1).unwrap(), Value::Blob(stmt_dctf.clone()));
        assert_eq!(
            r1.get_value(2).unwrap(),
            Value::Blob(Sha256::digest(&stmt_dctf).to_vec())
        );

        let r2 = rows.next().await.unwrap().unwrap();
        assert_eq!(r2.get_value(0).unwrap(), Value::Blob(id_sys));
        assert_eq!(r2.get_value(1).unwrap(), Value::Blob(sys_dctf.clone()));
        assert_eq!(
            r2.get_value(2).unwrap(),
            Value::Blob(Sha256::digest(&sys_dctf).to_vec())
        );
        drop(rows);
        drop(stmt);

        // Corrupt checksum must fail migration
        conn.execute(
            "UPDATE nodes SET checksum = ?1 WHERE id = ?2",
            (vec![0_u8; 32], 12_u128.to_be_bytes().to_vec()),
        )
        .await
        .unwrap();
        assert!(run_rust_migration(&conn).await.is_err());
    }
}
