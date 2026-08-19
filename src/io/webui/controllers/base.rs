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

//! Controller for static assets and other routes shared between the app and the
//! general web site.
use crate::utilities::*;

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use mime_guess;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::{AppState, RequestState, error_400, error_404, render_view};
use crate::{respond_markdown_unsafe, utilities::build_info};
use ctb_installer::chunking::{
    compute_sha256_hex, read_chunk_from_directory_compressed,
};
use ctb_installer::manifest::{FileEntry, ReleaseManifest};
use ctb_storage::get_asset;

fn add_redirect_headers(resp: &mut Response) {
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    resp.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("X-CollectiveToolbox-IsJsRequest"),
    );
}

/// Build a JSON redirect response with X-CollectiveToolbox-IsJsRedirect header.
fn json_redirect_response(location: &str) -> Response {
    let mut resp = Response::new(Body::from(
        serde_json::json!({ "url": location }).to_string(),
    ));
    resp.headers_mut().insert(
        "X-CollectiveToolbox-IsJsRedirect",
        HeaderValue::from_static("true"),
    );
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    *resp.status_mut() = axum::http::StatusCode::OK;
    add_redirect_headers(&mut resp);
    resp
}

/// Redirect helper for 307 (Temporary Redirect, preserves method).
/// If X-CollectiveToolbox-IsJsRequest header is present, returns JSON with
/// target URL and X-CollectiveToolbox-IsJsRedirect response header.
pub fn redirect_temporary_preserve_method(
    is_js_req: bool,
    location: &str,
) -> Response {
    let mut resp = if is_js_req {
        json_redirect_response(location)
    } else {
        axum::response::Redirect::temporary(location).into_response()
    };
    add_redirect_headers(&mut resp);
    resp
}

/// Redirect helper for 303 (See Other, changes POST to GET).
/// If X-CollectiveToolbox-IsJsRequest header is present, returns JSON with
/// target URL and X-CollectiveToolbox-IsJsRedirect response header.
pub fn redirect_temporary(is_js_req: bool, location: &str) -> Response {
    let mut resp = if is_js_req {
        json_redirect_response(location)
    } else {
        axum::response::Redirect::to(location).into_response()
    };
    add_redirect_headers(&mut resp);
    resp
}

/// Redirect helper for 308 (Permanent Redirect, preserves method).
/// If X-CollectiveToolbox-IsJsRequest header is present, returns JSON with
/// target URL and X-CollectiveToolbox-IsJsRedirect response header.
pub fn redirect_permanent(is_js_req: bool, location: &str) -> Response {
    let mut resp = if is_js_req {
        json_redirect_response(location)
    } else {
        axum::response::Redirect::permanent(location).into_response()
    };
    add_redirect_headers(&mut resp);
    resp
}

pub async fn get_app_css(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    match render_view(
        &state.hbs,
        "encre_css".to_string(),
        &req,
        &json_value!({}),
    ) {
        Ok(css) => {
            let mut resp = Response::new(Body::from(css));
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/css"),
            );
            resp
        }
        Err(e) => error_400(&state, &req, e),
    }
}

pub async fn get_installer_linux_x64(
    State(state): State<AppState>,
    req: RequestState,
    headers: HeaderMap,
) -> Response {
    serve_latest_installer_release(&state, &req, &headers, "linux-x64").await
}

pub async fn get_installer_linux_x86(
    State(state): State<AppState>,
    req: RequestState,
    headers: HeaderMap,
) -> Response {
    serve_latest_installer_release(&state, &req, &headers, "linux-x86").await
}

fn get_releases_dir(storage_dir_override: &Option<PathBuf>) -> Result<PathBuf> {
    let base = if let Some(path) = storage_dir_override {
        path.clone()
    } else {
        ctb_utilities::storage::get_storage_dir()?
    };
    Ok(base.join("releases"))
}

fn get_release_chunks_dir(
    storage_dir_override: &Option<PathBuf>,
) -> Result<PathBuf> {
    Ok(get_releases_dir(storage_dir_override)?.join("bh"))
}

fn load_latest_release_manifest(
    storage_dir_override: &Option<PathBuf>,
    platform: &str,
) -> Result<ReleaseManifest> {
    let manifest_path = get_releases_dir(storage_dir_override)?
        .join(format!("ctb-{platform}-latest.json"));
    let manifest_text =
        std::fs::read_to_string(&manifest_path).with_context(|| {
            format!(
                "Failed to read release manifest {}",
                manifest_path.display()
            )
        })?;
    serde_json::from_str(&manifest_text).with_context(|| {
        format!(
            "Failed to parse release manifest {}",
            manifest_path.display()
        )
    })
}

fn find_release_file<'a>(
    manifest: &'a ReleaseManifest,
    install_path: &str,
) -> Result<&'a FileEntry> {
    manifest
        .files
        .iter()
        .find(|entry| entry.path == install_path)
        .ok_or_else(|| {
            anyhow!("Release file not found in manifest: {install_path}")
        })
}

fn assemble_release_file_bytes(
    storage_dir_override: &Option<PathBuf>,
    entry: &FileEntry,
) -> Result<Vec<u8>> {
    let total_len = usize::try_from(entry.file_size)
        .context("Release file size exceeds usize range")?;
    let mut bytes = vec![0u8; total_len];
    let chunks_dir = get_release_chunks_dir(storage_dir_override)?;

    for info in &entry.chunks {
        let start = usize::try_from(info.offset)
            .context("Chunk offset exceeds usize range")?;
        let length = usize::try_from(info.length)
            .context("Chunk length exceeds usize range")?;
        let end = start
            .checked_add(length)
            .context("Chunk range overflow while assembling installer")?;
        if end > bytes.len() {
            bail!("Chunk range exceeds assembled file size for {}", entry.path);
        }

        let chunk = read_chunk_from_directory_compressed(
            &info.hash,
            &chunks_dir,
            info.offset,
        )?;
        if chunk.data.len() != length {
            bail!(
                "Chunk length mismatch for {}: expected {}, got {}",
                info.hash,
                info.length,
                chunk.data.len()
            );
        }

        let Some(target) = bytes.get_mut(start..end) else {
            bail!("Chunk range exceeds assembled file size for {}", entry.path);
        };
        target.copy_from_slice(&chunk.data);
    }

    let assembled_hash = compute_sha256_hex(&bytes);
    if assembled_hash != entry.checksum {
        bail!(
            "Assembled file checksum mismatch for {}: expected {}, got {}",
            entry.path,
            entry.checksum,
            assembled_hash
        );
    }

    Ok(bytes)
}

fn gzip_compress_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::GzBuilder;
    use std::io::Write;

    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::best());
    encoder.write_all(bytes)?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

async fn serve_manifest_file(
    state: &AppState,
    req: &RequestState,
    headers: &HeaderMap,
    platform: &str,
    file_name: &str,
    download_file_name: &str,
    download_mime_type: &str,
    fallback_url: &str,
    archive_gzip: bool,
) -> Response {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    use tokio_util::io::ReaderStream;

    let platform_owned = platform.to_string();
    let filename_owned = file_name.to_string();
    let storage_override = state.storage_dir_override.clone();

    // 1. Get cache file path
    let releases_dir = match get_releases_dir(&storage_override) {
        Ok(d) => d,
        Err(e) => return error_400(state, req, e),
    };
    let cache_dir = if storage_override.is_some() {
        releases_dir.join("downloads_cache")
    } else {
        match ctb_utilities::storage::get_cache_dir() {
            Ok(d) => d.join("downloads_cache"),
            Err(e) => return error_400(state, req, e),
        }
    };
    let cached_file_path = cache_dir.join(download_file_name);

    // 2. Coordinate concurrent generation
    let filename_string = download_file_name.to_string();
    let mut is_generator = false;

    loop {
        if cached_file_path.exists() {
            break;
        }

        let mut generating = state.generating_downloads.lock().await;
        if cached_file_path.exists() {
            break;
        }

        if generating.contains(&filename_string) {
            // Someone else is generating it, drop lock and wait
            drop(generating);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        } else {
            // We are the generator!
            generating.insert(filename_string.clone());
            is_generator = true;
            break;
        }
    }

    if is_generator {
        let storage_override = storage_override.clone();
        let platform_owned = platform_owned.clone();
        let filename_owned = filename_owned.clone();
        let cached_file_path_clone = cached_file_path.clone();

        let generate_res =
            tokio::task::spawn_blocking(move || -> Result<()> {
                let manifest = load_latest_release_manifest(
                    &storage_override,
                    &platform_owned,
                )?;
                let installer = find_release_file(&manifest, &filename_owned)?;
                let bytes =
                    assemble_release_file_bytes(&storage_override, installer)?;

                // If archive_gzip is true, we compress it, otherwise we write the raw bytes
                let bytes_to_write = if archive_gzip {
                    gzip_compress_bytes(&bytes)?
                } else {
                    bytes
                };

                // Write to a temporary file and rename atomically
                let temp_file_path =
                    cached_file_path_clone.with_extension("tmp");
                if let Some(parent) = temp_file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&temp_file_path, bytes_to_write)?;
                std::fs::rename(&temp_file_path, &cached_file_path_clone)?;
                Ok(())
            })
            .await;

        // Clean up from the generating set
        let mut generating = state.generating_downloads.lock().await;
        generating.remove(&filename_string);
        drop(generating);

        match generate_res {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                return if environment::is_official_public_website() {
                    error_404(state, req, error)
                } else {
                    redirect_temporary(req.is_js_request, fallback_url)
                };
            }
            Err(error) => return error_400(state, req, anyhow!(error)),
        }
    }

    // 3. Serve the cached file
    let file = match tokio::fs::File::open(&cached_file_path).await {
        Ok(f) => f,
        Err(e) => {
            return error_400(
                state,
                req,
                anyhow!("Failed to open cached download file: {e}"),
            );
        }
    };
    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            return error_400(
                state,
                req,
                anyhow!("Failed to read cached download metadata: {e}"),
            );
        }
    };
    let total_size = metadata.len();

    let range =
        crate::controllers::releases::parse_range_header(headers, total_size);

    if let Some((start, end)) = range {
        // Range request / Partial content
        let mut file = file;
        if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
            return error_400(
                state,
                req,
                anyhow!("Failed to seek cached file: {e}"),
            );
        }
        let content_length = end.saturating_sub(start);
        let stream = ReaderStream::new(file.take(content_length));
        let body = Body::new(SizedStreamBody {
            stream,
            size: content_length,
        });

        let mut resp = Response::new(body);
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;

        let headers_mut = resp.headers_mut();
        headers_mut.insert(
            header::CONTENT_TYPE,
            // Reason for fallback: invalid MIME type header string defaults to application/octet-stream
            HeaderValue::from_str(download_mime_type).unwrap_or_else(|_| {
                HeaderValue::from_static("application/octet-stream")
            }),
        );
        headers_mut
            .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

        if let Ok(len_val) = HeaderValue::from_str(&content_length.to_string())
        {
            headers_mut.insert(header::CONTENT_LENGTH, len_val);
        }

        let range_str =
            format!("bytes {start}-{}/{total_size}", end.saturating_sub(1));
        if let Ok(range_val) = HeaderValue::from_str(&range_str) {
            headers_mut.insert(header::CONTENT_RANGE, range_val);
        }

        if let Ok(disposition) = HeaderValue::from_str(&format!(
            "attachment; filename=\"{download_file_name}\""
        )) {
            headers_mut.insert(header::CONTENT_DISPOSITION, disposition);
        }
        headers_mut.insert(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "ctb_download_started=1; Path=/; SameSite=Lax",
            ),
        );
        resp
    } else {
        // Full response (200 OK)
        let stream = ReaderStream::new(file);
        let body = Body::new(SizedStreamBody {
            stream,
            size: total_size,
        });

        let mut resp = Response::new(body);
        let headers_mut = resp.headers_mut();
        headers_mut.insert(
            header::CONTENT_TYPE,
            // Reason for fallback: invalid MIME type header string defaults to application/octet-stream
            HeaderValue::from_str(download_mime_type).unwrap_or_else(|_| {
                HeaderValue::from_static("application/octet-stream")
            }),
        );
        headers_mut
            .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

        if let Ok(len_val) = HeaderValue::from_str(&total_size.to_string()) {
            headers_mut.insert(header::CONTENT_LENGTH, len_val);
        }
        if let Ok(disposition) = HeaderValue::from_str(&format!(
            "attachment; filename=\"{download_file_name}\""
        )) {
            headers_mut.insert(header::CONTENT_DISPOSITION, disposition);
        }
        headers_mut.insert(
            header::SET_COOKIE,
            HeaderValue::from_static(
                "ctb_download_started=1; Path=/; SameSite=Lax",
            ),
        );
        resp
    }
}

#[pin_project::pin_project]
pub(crate) struct SizedStreamBody<S> {
    #[pin]
    pub(crate) stream: S,
    pub(crate) size: u64,
}

impl<S, T, E> http_body::Body for SizedStreamBody<S>
where
    S: futures::Stream<Item = Result<T, E>>,
    T: bytes::Buf,
{
    type Data = T;
    type Error = E;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<
        Option<Result<http_body::Frame<Self::Data>, Self::Error>>,
    > {
        let this = self.project();
        match this.stream.poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(buf))) => {
                std::task::Poll::Ready(Some(Ok(http_body::Frame::data(buf))))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
                std::task::Poll::Ready(Some(Err(err)))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::SizeHint::with_exact(self.size)
    }
}

async fn serve_latest_installer_release(
    state: &AppState,
    req: &RequestState,
    headers: &HeaderMap,
    platform: &str,
) -> Response {
    let fallback_url = format!("{}/installer-{platform}", default_url());
    serve_manifest_file(
        state,
        req,
        headers,
        platform,
        "ctoolbox-installer",
        &format!("ctoolbox-installer-{platform}.bin"),
        "application/x-executable",
        fallback_url.as_str(),
        false, // archive_gzip
    )
    .await
}

pub async fn get_src_zip_redirect(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let _ = state;
    redirect_permanent(req.is_js_request, "/src.tar.gz")
}

pub async fn get_dependencies_zip_redirect(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let _ = state;
    redirect_permanent(req.is_js_request, "/dependencies.tar.gz")
}

pub async fn get_src_tar_gz(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let _ = state;
    let build_info = build_info();
    let version = build_info.version;
    let commit = build_info.commit;

    let target =
        format!("/releases/src/ctoolbox-src-{version}-{commit}.tar.gz");
    redirect_temporary(req.is_js_request, &target)
}

pub async fn get_dependencies_tar_gz(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let _ = state;
    let build_info = build_info();
    let version = build_info.version;
    let commit = build_info.commit;

    let target = format!(
        "/releases/src/ctoolbox-dependencies-{version}-{commit}.tar.gz"
    );
    redirect_temporary(req.is_js_request, &target)
}

pub async fn get_versioned_release_source(
    State(state): State<AppState>,
    req: RequestState,
    headers: HeaderMap,
    Path(filename): Path<String>,
) -> Response {
    if !(filename.starts_with("ctoolbox-src-")
        || filename.starts_with("ctoolbox-dependencies-"))
        || !filename.ends_with(".tar.gz")
    {
        return error_400(&state, &req, anyhow!("Invalid filename format"));
    }

    if filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return error_400(&state, &req, anyhow!("Invalid path characters"));
    }

    let original_filename = match filename.strip_suffix(".gz") {
        Some(s) => s.to_string(),
        None => return error_400(&state, &req, anyhow!("Invalid extension")),
    };

    let fallback_url = format!("{}/releases/src/{filename}", default_url());

    serve_manifest_file(
        &state,
        &req,
        &headers,
        "linux-x64",
        &original_filename,
        &filename,
        "application/gzip",
        &fallback_url,
        true, // archive_gzip
    )
    .await
}

pub async fn get_doc_index(
    State(_state): State<AppState>,
    req: RequestState,
) -> Response {
    let target = if req.is_embedded {
        "/docs/index.md?embed=1"
    } else {
        "/docs/index.md"
    };
    redirect_temporary(req.is_js_request, target)
}

pub async fn get_doc_page(
    State(state): State<AppState>,
    req: RequestState,
    path: axum::extract::Path<String>,
) -> Response {
    let path = path.as_str().trim_start_matches('/');

    if path == "lib" || path == "lib/" {
        // Redirect /docs/lib to the library docs index.html
        // No need for ?embed=1 on this one since it's generated documentation that doesn't go through the usual UI template.
        let target = "/docs/lib/ctoolbox/index.html";
        return redirect_temporary(req.is_js_request, target);
    }

    if path == "rust" || path.starts_with("rust/") {
        // The API got reorganized, so just discard the path since they'd 404 anyway.
        return redirect_permanent(req.is_js_request, "/docs/lib");
    }

    if path.ends_with(".md") && !path.starts_with("lib/") {
        // For markdown docs, we can render them on the fly.
        // Content embedded as assets should be safe
        return respond_markdown_unsafe(
            &state,
            req,
            format!("/docs/{path}").as_str(),
        );
    }

    asset_or_404(&state, req, format!("/docs/{path}").as_str())
}

pub async fn static_or_404(
    State(state): State<AppState>,
    req: RequestState,
    uri: Uri,
) -> Response {
    asset_or_404(&state, req, uri.path())
}

fn asset_or_404(state: &AppState, req: RequestState, path: &str) -> Response {
    let path = path.trim_start_matches('/');

    // Try serving conditionally branded asset
    let asset_dir = if branding::is_branded_build() {
        "web/official-branding"
    } else {
        "web/generic-branding"
    };
    let mut asset = get_asset(format!("{asset_dir}/{path}").as_str());

    if asset.is_none() {
        // Try serving embedded asset
        asset = get_asset(path);
        if asset.is_none() {
            asset = get_asset(format!("web/{path}").as_str());
        }
        if asset.is_none() {
            if let Some(tail) = path
                .strip_prefix("vendor/v86_images/")
                .or_else(|| path.strip_prefix("v86_images/"))
            {
                asset = get_asset(format!("images/{tail}").as_str());
            }
        }
    }

    if let Some(bytes) = asset {
        let mime_guess = mime_guess::from_path(path).first();
        let mime_guess_str: Cow<'static, str> = match mime_guess {
            Some(mime) => Cow::Owned(mime.essence_str().to_string()),
            None => Cow::Borrowed("application/octet-stream"),
        };

        let mut resp = Response::new(Body::from(bytes));
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            // Reason for fallback: invalid MIME type header string defaults to application/octet-stream
            HeaderValue::from_str(mime_guess_str.as_ref()).unwrap_or_else(
                |_| HeaderValue::from_static("application/octet-stream"),
            ),
        );
        return resp;
    }

    // Otherwise render 404 page
    error_404(
        state,
        &req,
        format!("The requested URL '{path}' was not found."),
    )
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    use crate::test_helpers::{
        TestApp, body_to_text, test_get_no_login, test_get_no_login_json,
        test_get_redirect_no_login,
    };
    use crate::utilities::{
        assert_string_contains, assert_string_not_contains,
    };
    use axum::http::{Method, StatusCode, header};

    #[crate::ctb_test("tokio")]
    async fn can_get_doc_index() {
        let (status, location) = test_get_redirect_no_login("/docs").await;
        assert_eq!(status, 303);
        assert_eq!(location, "/docs/index.md");
    }

    #[crate::ctb_test("tokio")]
    async fn test_docs_redirect_headers() {
        let app = TestApp::default();
        let resp = app
            .request_get_response::<()>(
                Method::GET,
                "/docs",
                None,
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let headers = resp.headers();
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .unwrap()
                .to_str()
                .unwrap(),
            "no-store, no-cache, must-revalidate"
        );
        assert_eq!(
            headers.get(header::VARY).unwrap().to_str().unwrap(),
            "X-CollectiveToolbox-IsJsRequest"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_docs_rust_redirect() {
        let (status, location) = test_get_redirect_no_login("/docs/rust").await;
        assert_eq!(status, 308);
        assert_eq!(location, "/docs/lib");

        let (status, location) = test_get_redirect_no_login(
            "/docs/rust/ctoolbox/macro.unwrap_or_result_error.html",
        )
        .await;
        assert_eq!(status, 308);
        assert_eq!(location, "/docs/lib");
    }

    #[crate::ctb_test("tokio")]
    async fn can_get_library_docs_index() {
        let (status, location) = test_get_redirect_no_login("/docs/lib").await;
        assert_eq!(status, 303);
        assert_eq!(location, "/docs/lib/ctoolbox/index.html");
        let (status, body) =
            test_get_no_login("/docs/lib/ctoolbox/index.html").await;
        assert_eq!(status, 200);
        let contains_stub = body
            .contains(ctb_build_support::asset_packer::DEBUG_LIBRARY_DOCS_STUB);
        let contains_title = body.contains("<title>ctoolbox - Rust</title>");
        assert!(
            contains_stub || contains_title,
            "Body did not contain debug stub or library title. Body: {body}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn can_get_css() {
        let (status, body) = test_get_no_login("/app.css").await;
        assert_eq!(status, 200);
        assert!(body.contains("Abstract-Polygon-Background")); // Check for app code
        assert!(body.contains("SFMono-Regular")); // Check for encrecss code
    }

    #[crate::ctb_test("tokio")]
    async fn can_download_source() {
        let test_app = TestApp::new();
        let build_info = build_info();
        let version = build_info.version;
        let commit = build_info.commit;

        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/src.tar.gz",
                None,
                None,
                None,
                None,
            )
            .await;
        let status = resp.status();
        let location_header = resp.headers().get(header::LOCATION).cloned();

        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(
            location_header,
            Some(
                format!("/releases/src/ctoolbox-src-{version}-{commit}.tar.gz")
                    .parse()
                    .unwrap()
            )
        );

        // Test the versioned release source URL redirect
        let resp2 = test_app
            .request_get_response::<()>(
                Method::GET,
                &format!(
                    "/releases/src/ctoolbox-src-{version}-{commit}.tar.gz"
                ),
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(resp2.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            resp2.headers().get(header::LOCATION).cloned(),
            Some(
                format!(
                    "{}/releases/src/ctoolbox-src-{version}-{commit}.tar.gz",
                    default_url()
                )
                .parse()
                .unwrap()
            )
        );
    }

    #[crate::ctb_test("tokio")]
    async fn can_get_404() {
        let (status, body) = test_get_no_login("/nonexistent").await;
        assert_eq!(status, 404);
        assert_string_contains("was not found.</h1>", &body);
    }

    #[crate::ctb_test("tokio")]
    async fn can_get_404_json() {
        let (status, body) = test_get_no_login_json("/nonexistent").await;
        assert_eq!(status, 404);
        assert_string_contains("was not found.", &body);
        assert_string_not_contains("was not found.</h1>", &body);
    }

    #[crate::ctb_test("tokio")]
    async fn offline_bundle_404_redirect() {
        let test_app = TestApp::new();
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/releases/linux-x64/latest.tar",
                None,
                None,
                None,
                None,
            )
            .await;

        let status = resp.status();
        let location_header = resp.headers().get(header::LOCATION).cloned();

        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(
            location_header,
            Some(
                format!("{}/releases/linux-x64/latest.tar", default_url())
                    .parse()
                    .unwrap()
            )
        );
    }

    #[crate::ctb_test("tokio")]
    async fn manifest_file_range_request() {
        //bypass-tempdir-lint
        let test_app = TestApp::new();
        let storage_dir = test_app
            .state
            .storage_dir_override
            .as_ref()
            .unwrap()
            .join("releases");
        std::fs::create_dir_all(&storage_dir).unwrap();

        let chunks_dir = storage_dir.join("bh");
        std::fs::create_dir_all(&chunks_dir).unwrap();

        // Write a mock manifest ctb-linux-x64-latest.json
        let manifest_content = r#"{
            "format_version": 1,
            "ctoolbox_version": "0.1.0",
            "platform": "linux-x64",
            "date": "2026-06-16T22:43:00Z",
            "revoked_key_ids": [],
            "files": [
                {
                    "path": "ctoolbox-installer",
                    "checksum": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
                    "file_size": 11,
                    "gzip_after_install": false,
                    "feature_id": "core",
                    "feature_name": {},
                    "requires": [],
                    "required": true,
                    "unavailable": false,
                    "chunks": [
                        {
                            "hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
                            "offset": 0,
                            "length": 11
                        }
                    ]
                }
            ]
        }"#;

        std::fs::write(
            storage_dir.join("ctb-linux-x64-latest.json"),
            manifest_content,
        )
        .unwrap();

        // Write a mock chunk file compressed with Brotli containing "hello world"
        // "hello world" is 11 bytes.
        let mut compressed_data = Vec::new();
        {
            let mut encoder = brotli::CompressorWriter::new(
                &mut compressed_data,
                4096,
                11,
                22,
            );
            use std::io::Write;
            encoder.write_all(b"hello world").unwrap();
            encoder.flush().unwrap();
        }

        let chunk_hash =
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let chunk_prefix1 = chunk_hash.get(0..2).unwrap_or("");
        let chunk_prefix2 = chunk_hash.get(2..4).unwrap_or("");
        let chunk_path_dir = chunks_dir.join(chunk_prefix1).join(chunk_prefix2);
        std::fs::create_dir_all(&chunk_path_dir).unwrap();
        std::fs::write(
            chunk_path_dir.join(format!("{chunk_hash}.br")),
            compressed_data,
        )
        .unwrap();

        // 1. Request without range
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/installer-linux-x64",
                None,
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::ACCEPT_RANGES),
            Some(&HeaderValue::from_static("bytes"))
        );
        assert_eq!(
            resp.headers().get(header::CONTENT_LENGTH),
            Some(&HeaderValue::from_static("11"))
        );
        let body = body_to_text(resp).await;
        assert_eq!(body, "hello world");

        // 2. Request with partial range (0-4)
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, "bytes=0-4".parse().unwrap());
        let resp_range = test_app
            .request_get_response::<()>(
                Method::GET,
                "/installer-linux-x64",
                Some(headers),
                None,
                None,
                None,
            )
            .await;

        assert_eq!(resp_range.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            resp_range.headers().get(header::ACCEPT_RANGES),
            Some(&HeaderValue::from_static("bytes"))
        );
        assert_eq!(
            resp_range.headers().get(header::CONTENT_LENGTH),
            Some(&HeaderValue::from_static("5"))
        );
        assert_eq!(
            resp_range.headers().get(header::CONTENT_RANGE),
            Some(&HeaderValue::from_static("bytes 0-4/11"))
        );
        let body_range = body_to_text(resp_range).await;
        assert_eq!(body_range, "hello");
    }
}
