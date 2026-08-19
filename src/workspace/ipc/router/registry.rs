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

//! Registry-based IPC method dispatch.
//!
//! These helpers route requests to `ctb_utilities::ipc::registry` handlers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::Arc;

impl super::IpcRouter {
    pub(crate) async fn dispatch_registry_with_ctx(
        ipc_ctx: Arc<dyn ctb_utilities::ipc::registry::IpcRequestContext>,
        request: &crate::protocol::Request,
    ) -> Result<Option<crate::protocol::Response>, crate::error::Error> {
        let Some(reg) = ctb_utilities::ipc::registry::find(
            request.method.service.as_str(),
            request.method.method.as_str(),
        ) else {
            return Ok(None);
        };

        match (reg.handler)(ipc_ctx, &request.args).await {
            Ok(bytes) => {
                Ok(Some(super::IpcRouter::success_response(request.id, bytes)))
            }
            Err(e) => Ok(Some(super::IpcRouter::error_response(
                request.id,
                "internal",
                &e.to_string(),
            ))),
        }
    }
}
