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

//! CLI execution helpers for storage operations.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;


use crate as ctb_storage;

pub fn adduser(username: &str, password_stdin: bool) -> Result<ToolResult> {
                use std::io::IsTerminal;
            use std::io::Write;

            let password_str =
                if !password_stdin && std::io::stdin().is_terminal() {
                    print!("Enter password for '{username}': ");
                    std::io::stdout().flush()?;
                    let mut p1 = String::new();
                    std::io::stdin().read_line(&mut p1)?;
                    let p1 = p1.trim_end_matches(['\r', '\n']).to_string();

                    print!("Confirm password: ");
                    std::io::stdout().flush()?;
                    let mut p2 = String::new();
                    std::io::stdin().read_line(&mut p2)?;
                    let p2 = p2.trim_end_matches(['\r', '\n']).to_string();

                    if p1 != p2 {
                        bail!("Passwords do not match");
                    }
                    p1
                } else {
                    let mut p = String::new();
                    std::io::stdin().read_line(&mut p)?;
                    p.trim_end_matches(['\r', '\n']).to_string()
                };

            if password_str.is_empty() {
                bail!("Password cannot be empty");
            }

            let password =
                ctb_utilities::password::Password::from_string(&password_str);
            let user =
                ctb_storage::user::add_non_admin_user(username, &password)?;

            println!(
                "User '{name}' registered successfully with ID {id}.",
                name = user.name(),
                id = user.local_id()
            );
            Ok(ToolResult::immediate_ok(Vec::new()))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_adduser_command() -> Result<()> {
        let username = format!("cli_user_{}", function_name!());
        ctb_storage::user::User::delete_by_name(&username).ok();

        // Ensure allow_local_account_creation is true for this test
        let mut settings =
            ctb_utilities::pc_settings::PcSettings::load().unwrap_or_default();
        settings.allow_local_account_creation =
            ctb_utilities::json::maybe_value::MaybeValue::Value(true);
        settings.save()?;

        // Note: Password comes from stdin or add_non_admin_user directly.
        // Let's test calling add_non_admin_user
        let password = ctb_utilities::password::Password::from_string(
            ctb_utilities::password::TEST_USER_PASS,
        );
        let user = ctb_storage::user::add_non_admin_user(&username, &password)?;
        assert_eq!(user.name(), username);
        assert!(!user.is_admin());

        user.delete()?;
        Ok(())
    }

}