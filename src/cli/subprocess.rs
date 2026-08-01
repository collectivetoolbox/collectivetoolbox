#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ipc::ChildKind;

/// Parse workspace/subprocess CLI arguments for the IPC runner.
///
/// This helper recognizes the subprocess marker [`ctb_utilities::ipc::IPC_ARG`]
/// and returns:
/// - `Some(kind)` when invoked as a subprocess (e.g. `--runtime`)
/// - `None` when invoked as the workspace (normal parent process)
///
/// It also returns the remaining non-IPC arguments:
/// - for subprocesses: arguments after the POSIX `--` separator
/// - for workspace: all arguments (excluding the program name)
pub fn parse_subprocess_cli(
    raw: Vec<String>,
) -> Result<(Option<ChildKind>, Vec<String>)> {
    use ctb_utilities::ipc::IPC_ARG;
    use ipc::child_kind_from_string;

    let args = raw.get(1..).unwrap_or(&[]);
    let is_subprocess = args.first().is_some_and(|a| a == IPC_ARG);
    if !is_subprocess {
        return Ok((None, args.to_vec()));
    }

    let args = if is_subprocess {
        args.get(1..).unwrap_or(&[])
    } else {
        args
    };

    // Split at POSIX arg separator.
    let (ipc_args, remaining) = match args.iter().position(|a| a == "--") {
        Some(idx) => (
            args.get(..idx).unwrap_or(&[]),
            args.get(idx.saturating_add(1)..).unwrap_or(&[]).to_vec(),
        ),
        None => (args, Vec::new()),
    };

    if !is_subprocess {
        return Ok((None, args.to_vec()));
    }

    // Determine which child we are.
    let mut kind: Option<ChildKind> = None;
    for arg in ipc_args {
        let Some(flag) = arg.strip_prefix("--") else {
            continue;
        };
        if flag == "socket" {
            continue;
        }
        if let Ok(k) = child_kind_from_string(flag) {
            kind = Some(k);
            return Ok((kind, remaining));
        }
    }

    Err(anyhow::anyhow!(
        "subprocess mode requires a child kind flag"
    ))
}

/// Extract `--socket <path>` from the IPC args.
pub fn parse_subprocess_socket(raw: &[String]) -> Result<Option<String>> {
    use ctb_utilities::ipc::IPC_ARG;

    let args = raw.get(1..).unwrap_or(&[]);
    let is_subprocess = args.first().is_some_and(|a| a == IPC_ARG);
    if !is_subprocess {
        return Ok(None);
    }
    let args = args.get(1..).unwrap_or(&[]);

    // Only consider args before POSIX separator.
    let ipc_args_end =
        args.iter().position(|a| a == "--").unwrap_or(args.len());
    let ipc_args = args.get(..ipc_args_end).unwrap_or(&[]);

    let mut it = ipc_args.iter();
    while let Some(a) = it.next() {
        if a == "--socket" {
            return Ok(it.next().cloned());
        }
    }

    Ok(None)
}

/// Read a single-line IPC capability token from stdin.
pub fn read_token_from_stdin() -> Result<String> {
    use std::io::Read;

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let token = buf.lines().next().unwrap_or("").trim().to_string();
    anyhow::ensure!(!token.is_empty(), "missing IPC token on stdin");
    Ok(token)
}
