// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

use std::env;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ipc::ChildKind;

pub async fn parse_subprocess_cli_and_maybe_start_service(
    args: Vec<String>,
) -> Result<(Option<ChildKind>, Vec<String>)> {
    let (kind, remaining_args) =
        ctb_cli::subprocess::parse_subprocess_cli(env::args().collect())?;

    if let Some(kind) = kind {
        start_service(&kind, &args).await?;
        unreachable!();
    }

    Ok((kind, remaining_args))
}

pub async fn start_service(
    kind: &ChildKind,
    args: &Vec<String>,
) -> Result<(Option<ChildKind>, Vec<String>)> {
    let _process_kind = ipc::format_child_kind(kind);

    let socket = ctb_cli::subprocess::parse_subprocess_socket(args)?
        .ok_or_else(|| {
            anyhow::anyhow!("--socket required for subprocess mode")
        })?;
    let token = ctb_cli::subprocess::read_token_from_stdin()?;
    crate::services::run_service(*kind, &socket, &token).await?;
    std::process::exit(0);
}
