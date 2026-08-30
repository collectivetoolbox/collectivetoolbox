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

//! Tool Result Abstractions

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use futures::Stream;
use std::pin::Pin;

// ---------------------------
// Tool Result Abstractions
// ---------------------------

pub enum ToolResult {
    // Immediate, single-buffer outputs
    Immediate {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    },
    // Streaming output (future extensibility)
    Streaming {
        stream: Pin<Box<dyn Stream<Item = OutputChunk> + Send>>,
        exit_code: i32,
    },
}

pub enum OutputChunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

impl ToolResult {
    pub fn immediate_ok(stdout: Vec<u8>) -> Self {
        ToolResult::Immediate {
            stdout,
            stderr: Vec::new(),
            exit_code: 0,
        }
    }
    pub fn immediate_err(stderr: Vec<u8>, code: i32) -> Self {
        ToolResult::Immediate {
            stdout: Vec::new(),
            stderr,
            exit_code: code,
        }
    }
}
