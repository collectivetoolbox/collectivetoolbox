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

//! Authoring, CSV table validation, and `.ctbnodes` graph ledger/inbox maintenance
//! tools separated from the core Dc document runtime.

pub mod column_spec;
pub mod csv_resolver;
pub mod data_constraint;
pub mod format;
pub mod graph_ledger;
pub mod layout;
pub mod report;
pub mod seed_graph_reconstruct;
pub mod shared;
pub mod syntax_ast;
pub mod syntax_parser;
pub mod syntax_validator;
pub mod updater;
pub mod validation;
pub mod wat;
