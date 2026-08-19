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

//! Database migration adding remote sync status tracking for users.

use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_07_05_3_add_user_remote_status";
pub const DESCRIPTION: &str = "Add remote_status column to users table";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let has_column = {
        let mut stmt = conn.prepare("PRAGMA table_info(users)").await?;
        let mut rows = stmt.query(()).await?;
        let mut found = false;
        while let Some(row) = rows.next().await? {
            if let Ok(turso::Value::Text(col_name)) = row.get_value(1) {
                if col_name == "remote_status" {
                    found = true;
                    break;
                }
            }
        }
        found
    };

    if !has_column {
        conn.execute(
            "ALTER TABLE users ADD COLUMN remote_status TEXT DEFAULT 'Pending'",
            (),
        )
        .await?;
    }
    Ok(())
}
