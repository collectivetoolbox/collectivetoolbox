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

//! Database migration adding user subscription and billing columns.

use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_07_02_1_add_user_subscription";
pub const DESCRIPTION: &str =
    "Add subscription_expiry and token_quota columns to users table";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    // Check if subscription_expiry already exists
    let has_subscription = crate::migrations::check_column_exists(
        conn,
        "users",
        "subscription_expiry",
    )
    .await?;
    if !has_subscription {
        conn.execute(
            "ALTER TABLE users ADD COLUMN subscription_expiry INTEGER",
            (),
        )
        .await?;
    }

    // Check if token_quota already exists
    let has_quota =
        crate::migrations::check_column_exists(conn, "users", "token_quota")
            .await?;
    if !has_quota {
        conn.execute("ALTER TABLE users ADD COLUMN token_quota INTEGER", ())
            .await?;
    }

    Ok(())
}
