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

use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_07_05_1_add_client_sync_tables";
pub const DESCRIPTION: &str =
    "Create client-side sync tables (sync_tokens and sync_id_ranges)";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let sqls = [
        "CREATE TABLE IF NOT EXISTS sync_tokens (key INTEGER PRIMARY KEY, token TEXT)",
        "CREATE TABLE IF NOT EXISTS sync_id_ranges (graph_id TEXT PRIMARY KEY, range_data BLOB)",
    ];
    for sql in sqls {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}
