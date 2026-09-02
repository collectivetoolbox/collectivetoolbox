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

//! Main workspace process coordinating services, IPC, and child lifecycles.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

import_all_ipc_client_ext_traits!();

use ctb_utilities::ipc::{ChildKind, format_child_kind};
use ctb_workspace_ipc::auth::capability::CapabilitySet;
use ctb_workspace_ipc::workspace_runner::cli::start_service;
use ctb_workspace_ipc::workspace_runner::{Workspace, WorkspaceServices};

use crate::panic_hooks::{
    setup_subprocess_panic_hooks, setup_workspace_panic_hooks,
};
use crate::utilities::resource_lock::check_filesystem_lock_support;
use crate::utilities::storage::get_storage_dir;
use cli::Cli;
use ctb_cli as cli;
pub use serde_json::json as utilities_serde_json_json;
use std::sync::OnceLock;

use std::path::PathBuf;

use anyhow::Result;
#[cfg(unix)]
use ctb_workspace_ipc::services::parent::ParentMessageEvent;
use ctb_workspace_ipc::workspace_runner::workspace_runtime::{
    ResolvedParentMessage, WorkspaceRuntime,
};
use ctb_workspace_ipc::workspace_runner::{
    SpawnRequestDecision, SpawnRequester, WorkspaceExt, WorkspaceRunner,
    WorkspaceRunnerConfig,
};

use crate::capabilities::{
    create_io_capabilities, create_network_capabilities,
    create_renderer_capabilities, create_runtime_capabilities,
    create_storage_capabilities,
};

use ctb_utilities::testing::binary_path::resolve_binary_path_supporting_tests_or_example;

pub use ctb_workspace_ipc;
pub mod capabilities;
pub mod crlite;
pub mod panic_hooks;
pub mod process;
pub mod update_status;

pub async fn entry() -> anyhow::Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    invocation_settings::apply_command_line_args(&raw_args)?;

    // Parse either a subprocess invocation or a user CLI command set.
    let invocation = cli::parse_invocation(Some(raw_args))?;

    match invocation {
        cli::Invocation::Subprocess(sub) => {
            setup_logger_for_subprocess(&sub.kind)?;
            setup_subprocess_panic_hooks();
            ctb_storage_minimal::xkb::ensure_xkb_config_root()?;
            ctb_storage_minimal::xkb::ensure_x11_locale_root()?;
            ctb_storage::validate_resource_bundle()?;
            start_service(&sub.kind, &sub.args).await?;
            Ok(())
        }
        cli::Invocation::User(cli_args) => {
            // Skip for subprocesses to avoid race.
            check_filesystem_lock_support()?;

            // Try lightweight tools (hex2dec, etc.).
            if let Some(code) = cli::maybe_run_lightweight(&cli_args).await? {
                // Allow tests to inspect code by returning an error code if
                // desired, but typically just exit here:
                std::process::exit(code);
            }

            // Proceed with full application boot.
            ctb_storage_minimal::xkb::ensure_xkb_config_root()?;
            ctb_storage_minimal::xkb::ensure_x11_locale_root()?;
            setup_logger_for_user()?;
            setup_workspace_panic_hooks();
            ctb_storage::validate_resource_bundle()?;
            boot(cli_args).await?;
            Ok(())
        }
    }
}

fn setup_logger_for_user() -> anyhow::Result<()> {
    crate::utilities::logging::setup_logger(
        "workspace".to_string(),
        "workspace".to_string(),
    )
}

fn setup_logger_for_subprocess(kind: &ChildKind) -> anyhow::Result<()> {
    let sub_index = format_child_kind(kind).to_string();
    let service_name = format_child_kind(kind).to_string();
    crate::utilities::logging::setup_logger(sub_index, service_name)
}

/// Main boot logic for normal application startup.
async fn boot(args: Cli) -> Result<()> {
    // The real workspace should run until it receives a shutdown request
    // (e.g. Ctrl-C or an explicit child->parent shutdown request).
    let runner = WorkspaceRunner::new(
        CtbWorkspace::default().with_args(args),
        WorkspaceRunnerConfig::default(),
        resolve_exe_path()?.into(),
        None,
    );
    let stats = runner.run().await?;

    anyhow::ensure!(
        stats.shutdown_received,
        "Workspace timed out or exited without shutdown"
    );
    anyhow::ensure!(
        !stats.forced_termination,
        "Workspace required forced termination"
    );

    debug_fmt!(
        "--- Shutdown summary: received {} data plane messages, shutdown={} ---",
        stats.data_plane_messages_received,
        stats.shutdown_received
    );
    Ok(())
}

#[derive(Debug)]
struct CtbWorkspace {
    services: WorkspaceServices,
    args: Cli,
}

impl CtbWorkspace {
    fn with_args(mut self, args: Cli) -> Self {
        self.args = args;
        self
    }

    fn args(&self) -> &Cli {
        &self.args
    }
}

impl Default for CtbWorkspace {
    fn default() -> Self {
        Self {
            services: WorkspaceServices::default(),
            args: Cli {
                ctoolbox_ipc_port: None,
                no_update: false,
                use_bundled_tls_validator: false,
                use_system_tls_validator: false,
                insecure_skip_crlite_check: false,
                retry_on_host_error:
                    invocation_settings::DEFAULT_RETRY_ON_HOST_ERROR,
                command: None,
            },
        }
    }
}

#[async_trait::async_trait]
impl Workspace for CtbWorkspace {
    fn services_needed(&self) -> Vec<(ChildKind, CapabilitySet)> {
        vec![
            (ChildKind::Io, create_io_capabilities()),
            (ChildKind::Network, create_network_capabilities()),
            (ChildKind::Storage, create_storage_capabilities()),
        ]
    }

    fn services(&self) -> &WorkspaceServices {
        &self.services
    }

    fn set_services(&mut self, services: WorkspaceServices) {
        self.services = services;
    }

    async fn boot(&mut self, rt: &WorkspaceRuntime) -> Result<()> {
        info_fmt!(
            "Starting Collective Toolbox on socket: {}",
            rt.socket_path()
        );

        if self.args().no_update {
            log!("Skipping update checks due to --no-update");
        } else {
            // Check for pending updates at startup (with 15-second timeout)
            match update_status::check_startup_update().await {
                update_status::StartupUpdateResult::UpgradeStarted => {
                    // Upgrade process was initiated - we need to exit
                    info!(
                        "Upgrade started, exiting to allow canary process to take over"
                    );
                    std::process::exit(0);
                }
                update_status::StartupUpdateResult::NoPendingUpdate => {
                    // No pending update, continue normally
                }
                update_status::StartupUpdateResult::TimedOut => {
                    // Check timed out, continue normally
                    log!("Startup update check timed out, continuing startup");
                }
                update_status::StartupUpdateResult::Error(e) => {
                    warn_fmt!("Startup update check failed: {e}");
                }
            }
        }

        // Best-effort Ctrl-C handling: request graceful shutdown of the
        // workspace runner.
        //
        // Note: `ctrlc` handlers are synchronous, so we forward the request
        // onto the Tokio runtime.
        let rt_for_signal = rt.clone();
        let handle = tokio::runtime::Handle::current();
        if let Err(e) = ctrlc::set_handler(move || {
            let rt = rt_for_signal.clone();
            handle.spawn(async move {
                if let Err(e) =
                    rt.request_shutdown(Some("ctrl-c".to_string())).await
                {
                    warn_fmt!("Failed to request shutdown; will try to force after timeout: {e:#}");
                }
            });
        }) {
            warn_fmt!("Failed to set ctrlc handler: {e:#}");
        }

        // Spawn background update checker
        if !self.args().no_update {
            // Reason for fallback: unconfigured server URL setting uses default official server URL
            let server_url = pc_settings::get_str_setting(
                pc_settings::PcSettingStrKey::ServerUrl,
            )
            .unwrap_or_else(default_url);
            if let Err(e) = update_status::spawn_update_checker(server_url) {
                warn_fmt!("Failed to start background update checker: {e:#}");
            }
        }

        Ok(())
    }

    #[expect(
        clippy::items_after_statements,
        reason = "uniform macro injection patterns"
    )]
    async fn run(&self, rt: &WorkspaceRuntime) -> Result<()> {
        // Allow the ipc! macro to be used in this function, so that instead of
        // writing `self.io()?` one can write `ipc!(io)`.
        macro_rules! __ctb_ipc_ctx {
            () => {
                self
            };
        }
        macro_rules! __ctb_ipc_get {
            ($ctx:expr, $service:ident) => {
                $ctx.$service()?
            };
        }

        let _storage_dir = get_storage_dir()?;
        // let _renderer = ipc!(renderer)?;

        ipc!(io).start_local_webui().await?;

        /* Example calls. A bit outdated. Leave here for now.
        print(strtovec(
            renderer.test_echo_3x("Hello, world!".into())
                .await?,
        ));

        sleep(1);

        let doc = ipc!(storage)
            .get_asset("intro.html").expect("Could not load intro.html");
        // let doc = strtovec("0");
        let doc_str = vectostr(&doc);
        print(strtovec(format!("Document: {doc_str}").as_str()));
        // let pid = ipc!(runtime).start(ctb_formats::convert_from(doc, strtovec("html")));
        // ipc!(runtime).start(ctb_formats::convert_from(doc, strtovec("html")));
        data_channel_test().await; */

        // Keep the workspace alive until shutdown is requested.
        rt.wait_for_shutdown().await;

        Ok(())
    }

    async fn on_parent_message(
        &self,
        rt: &WorkspaceRuntime,
        event: ParentMessageEvent,
    ) -> Result<()> {
        debug_fmt!(
            "Workspace received message from {:?}",
            event.ctx.process_kind
        );
        handle_parent_message(rt, &event).await?;
        Ok(())
    }

    async fn evaluate_spawn_request(
        &self,
        rt: &WorkspaceRuntime,
        requester: SpawnRequester,
        request: ctb_workspace_ipc::services::parent::api::SpawnChildRequest,
    ) -> Result<SpawnRequestDecision> {
        // If the request is for a singleton service (like Storage, Network, Io, Formats),
        // and it is already running, we accept the request.
        let is_singleton = matches!(
            request.kind,
            ChildKind::Storage
                | ChildKind::Network
                | ChildKind::Io
                | ChildKind::Formats
        );

        if is_singleton {
            if let Some(pid) = rt.get_singleton_pid(request.kind).await {
                return Ok(SpawnRequestDecision::Accept {
                    parent: Some(pid),
                    caps: CapabilitySet::default(),
                    extra_args: vec![],
                });
            }
        }

        // Allow the "io" service to spawn/access the runtime process.
        if requester.ctx.process_kind.as_deref() == Some("io")
            && request.kind == ChildKind::Runtime
        {
            let caps = create_runtime_capabilities();
            return Ok(SpawnRequestDecision::Accept {
                parent: None,
                caps,
                extra_args: vec![],
            });
        }

        // Allow runtime processes to spawn nested runtimes or renderers.
        if requester.ctx.process_kind.as_deref() == Some("runtime") {
            if request.kind != ChildKind::Runtime
                && request.kind != ChildKind::Renderer
            {
                return Ok(SpawnRequestDecision::Reject {
                    error: Some(
                        "only runtime + renderer spawn requests are allowed from runtime"
                            .into(),
                    ),
                });
            }

            let caps = match request.kind {
                ChildKind::Runtime => create_runtime_capabilities(),
                ChildKind::Renderer => create_renderer_capabilities(),
                _ => unreachable!(),
            };

            return Ok(SpawnRequestDecision::Accept {
                parent: requester.pid,
                caps,
                extra_args: vec![],
            });
        }

        Ok(SpawnRequestDecision::Reject {
            error: Some(format!(
                "spawn request for {:?} from {:?} denied by policy",
                request.kind, requester.ctx.process_kind
            )),
        })
    }
}

async fn handle_parent_message(
    rt: &WorkspaceRuntime,
    event: &ParentMessageEvent,
) -> Result<()> {
    match rt.resolve_parent_message(event).await? {
        ResolvedParentMessage::DataPlaneBytes { bytes, .. } => {
            debug_fmt!(
                "Workspace received data plane message: {}",
                String::from_utf8_lossy(&bytes)
            );
        }
        ResolvedParentMessage::DataPlaneReadFailed { error, .. } => {
            debug_fmt!("Workspace failed to read data plane blob: {error:#}");
        }
        ResolvedParentMessage::ShutdownRequest { reason } => {
            debug_fmt!(
                "Workspace received shutdown request: {}",
                // Reason for fallback: shutdown request without explicit reason message defaults to "no reason"
                reason.unwrap_or_else(|| "no reason".into())
            );
        }
        ResolvedParentMessage::Text(text) => {
            debug_fmt!("Workspace received text message: {text}");
        }
        ResolvedParentMessage::Other { kind } => {
            debug_fmt!("Workspace received message of kind: {kind:?}");
        }
    }
    Ok(())
}

fn resolve_exe_path() -> Result<PathBuf> {
    resolve_binary_path_supporting_tests_or_example(
        "ctoolbox",
        Some("CTB_WORKSPACE_EXE"),
    )
}

pub struct TestHarness;

static HARNESS: OnceLock<TestHarness> = OnceLock::new();

// Not meant to be used directly - use #[crate::ctb_test] instead.
pub fn boot_for_test() -> Option<&'static TestHarness> {
    if HARNESS.get().is_some() {
        return HARNESS.get();
    }

    // Note that this logger setup is only used for the workspace tests.
    // Individual tests have their own loggers set up by the ctb_test macro.
    if setup_logger_for_user().is_ok() {
        setup_workspace_panic_hooks();
        let _ = HARNESS.set(TestHarness);
    }

    HARNESS.get()
}
