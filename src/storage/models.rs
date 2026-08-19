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

//! Models can be called by other crates, not just storage crate. They abstract
//! away the process of querying the database. The actual query building all
//! happens in the storage singleton service. The storage service holds any
//! active session tokens to access the tenant databases. So, keeping the SQL
//! isolated within the storage service prevents a different compromised
//! process (for instance a runtime process) running arbitrary queries or
//! accessing users' data other than its own.

pub mod graph;
pub mod node;
pub mod sync;
pub mod user;

pub mod graph_impl;
pub mod node_impl;
pub mod sync_impl;
pub mod user_impl;
