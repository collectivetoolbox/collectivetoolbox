//! Controller for update status API.
//!
//! Provides a polling endpoint for the web UI to check for available updates.
//! The actual update checking is performed by the workspace process; this
//! controller fetches the current status via IPC.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{Json, extract::Query};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
pub struct UpdateStatusQuery {
    /// Version string of the web UI currently loaded in the browser.
    pub version: Option<String>,
    /// Build timestamp of the web UI currently loaded in the browser.
    pub build_date: Option<String>,
}

#[derive(Clone)]
struct BuildComparison {
    is_newer: bool,
    server_version: String,
    server_build_date: String,
}

/// Response structure for the update status endpoint.
#[derive(Serialize)]
pub struct UpdateStatusResponse {
    /// Whether an update is available.
    pub available: bool,
    /// The new version string (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// URL to release notes (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes_url: Option<String>,
    /// Whether the server is running a newer build than the client sent.
    pub is_newer: bool,
    /// Current server version.
    pub server_version: String,
    /// Current server build timestamp.
    pub server_build_date: String,
}

fn parse_build_date(build_date: &str) -> Result<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(build_date)
        .map(|parsed| parsed.with_timezone(&Utc))
        .with_context(|| {
            format!("Failed to parse build timestamp for update status: {build_date}")
        })
}

fn compare_against_client_build(query: &UpdateStatusQuery) -> BuildComparison {
    let server_build = build_info();
    let mut is_newer = false;

    let Some(client_version) = query.version.as_deref() else {
        return BuildComparison {
            is_newer,
            server_version: server_build.version,
            server_build_date: server_build.build_date,
        };
    };

    let server_version = match semver::Version::parse(&server_build.version) {
        Ok(version) => version,
        Err(e) => {
            warn_fmt!("Failed to parse server build version: {e:#}");
            return BuildComparison {
                is_newer,
                server_version: server_build.version,
                server_build_date: server_build.build_date,
            };
        }
    };
    let client_version = match semver::Version::parse(client_version) {
        Ok(version) => version,
        Err(e) => {
            warn_fmt!("Failed to parse client build version: {e:#}");
            return BuildComparison {
                is_newer,
                server_version: server_build.version,
                server_build_date: server_build.build_date,
            };
        }
    };

    if server_version > client_version {
        is_newer = true;
    } else if server_version == client_version {
        let Some(client_build_date) = query.build_date.as_deref() else {
            return BuildComparison {
                is_newer,
                server_version: server_build.version,
                server_build_date: server_build.build_date,
            };
        };

        let server_build_date = match parse_build_date(&server_build.build_date)
        {
            Ok(build_date) => build_date,
            Err(e) => {
                warn_fmt!("Failed to parse server build date: {e:#}");
                return BuildComparison {
                    is_newer,
                    server_version: server_build.version,
                    server_build_date: server_build.build_date,
                };
            }
        };
        let client_build_date = match parse_build_date(client_build_date) {
            Ok(build_date) => build_date,
            Err(e) => {
                warn_fmt!("Failed to parse client build date: {e:#}");
                return BuildComparison {
                    is_newer,
                    server_version: server_build.version,
                    server_build_date: server_build.build_date,
                };
            }
        };

        is_newer = server_build_date > client_build_date;
    }

    BuildComparison {
        is_newer,
        server_version: server_build.version,
        server_build_date: server_build.build_date,
    }
}

/// GET /api/update-status
///
/// Returns the current update status. The web UI polls this endpoint every 60s
/// when the tab is visible to check if an update is available.
///
/// The update status is written by the workspace process to a shared JSON file.
pub async fn get_update_status(
    Query(query): Query<UpdateStatusQuery>,
) -> Json<UpdateStatusResponse> {
    #[derive(Deserialize)]
    struct WorkspaceUpdateStatus {
        available: bool,
        version: Option<String>,
        release_notes_url: Option<String>,
    }

    let comparison = compare_against_client_build(&query);

    let json_str = match ipc!().get_update_status().await {
        Ok(v) => v,
        Err(e) => {
            warn_fmt!("Workspace IPC update-status call failed: {e:#}");
            return Json(UpdateStatusResponse {
                available: false,
                version: None,
                release_notes_url: None,
                is_newer: comparison.is_newer,
                server_version: comparison.server_version,
                server_build_date: comparison.server_build_date,
            });
        }
    };

    let status: WorkspaceUpdateStatus = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            warn_fmt!("Failed to parse workspace update-status JSON: {e:#}");
            return Json(UpdateStatusResponse {
                available: false,
                version: None,
                release_notes_url: None,
                is_newer: comparison.is_newer,
                server_version: comparison.server_version,
                server_build_date: comparison.server_build_date,
            });
        }
    };

    Json(UpdateStatusResponse {
        available: status.available,
        version: status.version,
        release_notes_url: status.release_notes_url,
        is_newer: comparison.is_newer,
        server_version: comparison.server_version,
        server_build_date: comparison.server_build_date,
    })
}
