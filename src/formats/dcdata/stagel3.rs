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

//! Core StageL 3 (`f656` `.sl3`) AST, parser, static validator, Token-Dc codec,
//! and deterministic reference evaluator shared across `ctb-formats-dcdata` and
//! `ctb-formats-stagel`.

pub mod ast;
pub mod dc_tokens;
pub mod eval;
pub mod parser;
pub mod seed_linker;
pub mod stager_parser;
pub mod stager_strings;
pub mod stager_tokens;
pub mod validator;
