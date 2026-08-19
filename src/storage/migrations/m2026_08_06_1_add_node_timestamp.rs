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

//! Database migration adding timestamp columns to graph nodes.

use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_08_06_1_add_node_timestamp";
pub const DESCRIPTION: &str =
    "Add high-resolution timestamp column to nodes table";
pub const UP_SQL: Option<&str> = Some("ALTER TABLE nodes ADD COLUMN timestamp BLOB");

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        // Reason for fallback: system clock time prior to UNIX epoch defaults duration to 0 duration
        .unwrap_or_default()
        .as_micros();
    let now_blob = now.to_be_bytes().to_vec();
    conn.execute(
        "UPDATE nodes SET timestamp = ?1 WHERE timestamp IS NULL",
        (now_blob,),
    )
    .await?;
    Ok(())
}
