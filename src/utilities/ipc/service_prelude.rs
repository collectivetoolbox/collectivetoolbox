use crate::ipc::service_traits::ChildIpcContext;
use crate::ipc::ChildKind;
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
    let bypassed = Arc::new(crate::ipc::in_process::BypassingChildIpcContext::new(
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
