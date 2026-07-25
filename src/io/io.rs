#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
pub use ctb_utilities::ipc::service_prelude::*;

pub use ctb_io_webui as webui;

/// Start a local web UI server, returning the port number.
#[ipc_method]
pub fn start_local_webui() -> u16 {
    webui::start_webui()
}

/// Print raw bytes as UTF-8 (lossy) to stdout.
#[ipc_method]
pub fn print(document: Vec<u8>) -> Result<()> {
    let string = String::from_utf8_lossy(&document).to_string();
    let string = string.as_str();

    println!("{string}");
    Ok(())
}

/// Print a string to stdout.
#[ipc_method]
pub fn print_string(document: String) -> Result<()> {
    print(strtovec(&document))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn can_start() {
        // put("key".to_string(), "value".to_string());
        // assert_eq!("key", String::from_utf8_lossy(&get("key").unwrap()));
    }
}
