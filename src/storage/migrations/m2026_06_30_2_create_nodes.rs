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
pub const NAME: &str = "2026_06_01_1_create_nodes";
pub const DESCRIPTION: &str = "Create initial nodes table";
pub const UP_SQL: &str = "CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    data BLOB NOT NULL
)";

pub async fn run_rust_migration(_conn: &Connection) -> Result<()> {
    Ok(())
}
