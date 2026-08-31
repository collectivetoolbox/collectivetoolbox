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

//! IO subsystem coordinating `WebUI`, IPC, and peripheral communications.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub use ctb_utilities::ipc::service_prelude::*;

pub use ctb_io_webui as webui;

/// Start a local web UI server, returning the port number.
#[ipc_method]
pub fn start_local_webui() -> u16 {
    webui::start_webui()
}

/// Print raw bytes as UTF-8 (lossy) to stdout.
#[ipc_method]
pub fn print(document: Vec<u8>) -> Result<()> {
    let string = String::from_utf8_lossy(&document).to_string();
    let string = string.as_str();

    println!("{string}");
    Ok(())
}

/// Print a string to stdout.
#[ipc_method]
pub fn print_string(document: String) -> Result<()> {
    print(strtovec(&document))?;
    Ok(())
}

#[cfg(test)]
#[expect(
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

    #[crate::ctb_test]
    fn can_start() {
        // put("key".to_string(), "value".to_string());
        // assert_eq!("key", String::from_utf8_lossy(&get("key").unwrap()));
    }
}
