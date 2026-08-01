#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
pub use ctb_utilities::ipc::service_prelude::*;

use async_trait::async_trait;
use ctb_utilities::ipc::service_traits::ChildIpcContext;
use std::sync::{Arc, OnceLock};

// pub mod webrtc;

#[ipc_method]
/// Echo bytes back.
pub async fn echo(bytes: Vec<u8>) -> Result<Vec<u8>> {
    Ok(bytes.clone())
}

#[ipc_method]
/// Fetch bytes from a URL.
pub async fn fetch(url: String) -> Result<Vec<u8>> {
    if url.starts_with("https://") {
        return backend()?.fetch_https(&url).await;
    }

    if url.starts_with("http://") {
        return backend()?.fetch_http(&url).await;
    }

    bail!("unsupported scheme for url {url}");
}

#[ipc_method]
/// Read a file into memory.
pub async fn read_file(path: String) -> Result<Vec<u8>> {
    backend()?.read_file(path).await
}

pub async fn get_url(key: &str) -> Result<Vec<u8>> {
    fetch(key.to_string()).await
}

static NETWORK_BACKEND: OnceLock<Arc<dyn NetworkBackend>> = OnceLock::new();

/// Initialize the network backend used by registry-based IPC methods.
///
/// This is intended to be called once by the network subprocess during
/// startup.
pub fn init_backend(backend: &Arc<dyn NetworkBackend>) -> Result<()> {
    if NETWORK_BACKEND.set(Arc::clone(backend)).is_err() {
        return Ok(());
    }
    Ok(())
}

fn backend() -> Result<&'static Arc<dyn NetworkBackend>> {
    NETWORK_BACKEND
        .get()
        .ok_or_else(|| anyhow::anyhow!("network backend not initialized"))
}

/// Networking backend that the IPC layer depends on.
#[async_trait]
pub trait NetworkBackend: Send + Sync {
    /// Fetch bytes over HTTP.
    async fn fetch_http(&self, url: &str) -> Result<Vec<u8>>;

    /// Fetch bytes over HTTPS.
    async fn fetch_https(&self, url: &str) -> Result<Vec<u8>>;

    /// Read a local file into memory.
    async fn read_file(&self, path: String) -> Result<Vec<u8>>;
}

/// Default implementation. This will eventually include APIs for a
/// WebRTC transport, but for now returns clear errors for unimplemented paths.
#[derive(Debug, Default)]
pub struct DefaultNetworkBackend;

#[async_trait]
impl NetworkBackend for DefaultNetworkBackend {
    async fn fetch_http(&self, url: &str) -> Result<Vec<u8>> {
        https::get_success(url)
            .await
            .with_context(|| format!("http(s) GET {url}"))
    }

    async fn fetch_https(&self, url: &str) -> Result<Vec<u8>> {
        return self.fetch_http(url).await;
    }

    async fn read_file(&self, path: String) -> Result<Vec<u8>> {
        tokio::fs::read(&path)
            .await
            .with_context(|| format!("read file at {path}"))
    }
}

#[derive(Debug, Default)]
pub struct MockNetworkBackend;
#[async_trait]
impl NetworkBackend for MockNetworkBackend {
    async fn fetch_http(&self, url: &str) -> Result<Vec<u8>> {
        if url == "http://example.com" {
            return Ok(b"Example HTTP Fetch".to_vec());
        }
        bail!(
            "MockNetworkingBackend only supports http and https fetches of http://example.com"
        );
    }

    async fn fetch_https(&self, url: &str) -> Result<Vec<u8>> {
        if url == "https://example.com" {
            return Ok(b"Example HTTPS Fetch".to_vec());
        }
        bail!(
            "MockNetworkingBackend only supports http and https fetches of https://example.com"
        );
    }

    async fn read_file(&self, path: String) -> Result<Vec<u8>> {
        if path == "/tmp/example/file.txt" {
            return Ok(b"Example File Read".to_vec());
        }
        bail!("MockNetworkingBackend only provides /tmp/example/file.txt");
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {}
