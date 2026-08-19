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

//! Prelude and global context accessors for child process IPC services.

use crate::ipc::ChildKind;
use crate::ipc::service_traits::ChildIpcContext;
use anyhow::Result;
use std::sync::{Arc, OnceLock};

static IPC_CONTEXT: OnceLock<Arc<dyn ChildIpcContext>> = OnceLock::new();
static IN_PROCESS_TEST_IPC_CONTEXT: OnceLock<Arc<dyn ChildIpcContext>> =
    OnceLock::new();

/// Initialize the runtime service's IPC context.
///
/// This is intended to be called once by the runtime subprocess during
/// startup, so registry-based IPC methods can call back into the workspace.
pub fn init_ipc_context(
    ctx: &Arc<dyn ChildIpcContext>,
    local_kind: Option<ChildKind>,
) -> Result<()> {
    let bypassed =
        Arc::new(crate::ipc::in_process::BypassingChildIpcContext::new(
            Arc::clone(ctx),
            local_kind,
        ));
    if IPC_CONTEXT.set(bypassed).is_err() {
        return Ok(());
    }
    Ok(())
}

pub fn ipc() -> Result<&'static dyn ChildIpcContext> {
    if let Some(ctx) = IPC_CONTEXT.get() {
        return Ok(ctx.as_ref());
    }

    // In tests, default to an in-process IPC context unless the test
    // explicitly opts into real IPC.
    if crate::is_in_test() && !real_ipc_in_test_enabled() {
        let ctx = IN_PROCESS_TEST_IPC_CONTEXT.get_or_init(|| {
            Arc::new(crate::ipc::in_process::InProcessChildIpcContext::new())
        });
        return Ok(ctx.as_ref());
    }

    Err(anyhow::anyhow!("runtime IPC context not initialized"))
}

fn real_ipc_in_test_enabled() -> bool {
    std::env::var("CTB_REAL_IPC_IN_TEST")
        .ok()
        .is_some_and(|v| v == "1")
}
