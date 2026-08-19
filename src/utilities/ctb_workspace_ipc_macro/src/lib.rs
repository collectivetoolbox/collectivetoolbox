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

//! Proc macros for generating typed workspace IPC client helpers.
//!
//! The generated code intentionally hides postcard encoding and raw IPC calls
//! from application code.

use proc_macro::TokenStream;

mod workspace_ipc_methods;

/// Generate a `WorkspaceIpcExt` trait + impls from a list of method
/// signatures.
///
/// Example:
/// ```ignore
/// workspace_ipc_methods! {
///     async fn get_update_status() -> Result<String>;
/// }
/// ```
#[proc_macro]
pub fn workspace_ipc_methods(input: TokenStream) -> TokenStream {
    workspace_ipc_methods::workspace_ipc_methods_impl(input)
}
