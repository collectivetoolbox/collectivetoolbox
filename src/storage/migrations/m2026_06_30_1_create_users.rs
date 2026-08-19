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

//! Database migration creating the initial users and tenancy tables.

use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_06_30_1_create_users";
pub const DESCRIPTION: &str = "Create initial users database tables";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let sqls = [
        "CREATE TABLE IF NOT EXISTS users_ids (key TEXT PRIMARY KEY, value INTEGER)",
        "CREATE TABLE IF NOT EXISTS users_ids_rev (key INTEGER PRIMARY KEY, value TEXT)",
        "CREATE TABLE IF NOT EXISTS users_uuids (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_auth (key INTEGER PRIMARY KEY, value TEXT)",
        "CREATE TABLE IF NOT EXISTS users_display_names (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_pictures (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_key_encryption_key_params (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_wrapped_dek (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_pubkeys (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_metadata (key TEXT PRIMARY KEY, value INTEGER)",
        "CREATE TABLE IF NOT EXISTS sync_local_id_ranges (key INTEGER PRIMARY KEY, value TEXT)",
    ];
    for sql in sqls {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}
