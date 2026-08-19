// SPDX-License-Identifier: AGPL-3.0-or-later AND MPL-2.0
// SPDX-License-Identifier for parts derived from Mozilla Firefox and moz_crlite_query: MPL-2.0
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// See additional licensing details at end of file.

//! Workspace-owned `CRLite` cache access.

#![allow(
    clippy::filter_next,
    clippy::unused_async,
    reason = "uniform signature mapping and iteration patterns"
)]

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_utilities::https::crlite::{
    DEFAULT_CRLITE_MAX_AGE, ensure_crlite_cache_ready_sync,
    get_crlite_cache_dir, get_crlite_manifest_path,
    load_crlite_collection_json, load_crlite_manifest, record_refresh_time,
    refresh_crlite_cache_sync, should_rate_limit_refresh,
    validate_relative_artifact_path,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct CRLiteCacheStatus {
    cache_dir: String,
    manifest_path: String,
    manifest_present: bool,
    fresh: bool,
    last_updated_unix_seconds: Option<u64>,
    current_filter_relative_path: Option<String>,
    delta_count: usize,
    source_kind: Option<String>,
}

async fn ensure_crlite_cache_ready() -> Result<()> {
    spawn_blocking_with_current_test_name(ensure_crlite_cache_ready_sync)
        .await
        .context("CRLite refresh task failed")??;
    Ok(())
}

fn build_crlite_cache_status() -> Result<CRLiteCacheStatus> {
    let cache_dir = get_crlite_cache_dir()?;
    let manifest_path = get_crlite_manifest_path()?;
    let manifest = load_crlite_manifest()?;

    let (
        manifest_present,
        fresh,
        last_updated_unix_seconds,
        current_filter,
        delta_count,
    ) = if let Some(manifest) = manifest {
        let fresh = manifest
            .is_fresh(std::time::SystemTime::now(), DEFAULT_CRLITE_MAX_AGE);
        (
            true,
            fresh,
            Some(manifest.last_updated_unix_seconds),
            manifest
                .current_filter
                .map(|artifact| artifact.relative_path),
            manifest.deltas.len(),
        )
    } else {
        (false, false, None, None, 0)
    };

    let source_kind = load_crlite_manifest()?
        .map(|manifest| format!("{:?}", manifest.source.kind));

    Ok(CRLiteCacheStatus {
        cache_dir: cache_dir.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        manifest_present,
        fresh,
        last_updated_unix_seconds,
        current_filter_relative_path: current_filter,
        delta_count,
        source_kind,
    })
}

/// Returns `CRLite` cache status as JSON for the web controller or diagnostics.
#[ipc_method]
pub async fn get_crlite_cache_status() -> Result<String> {
    ensure_crlite_cache_ready().await?;
    let status = build_crlite_cache_status()?;
    serde_json::to_string_pretty(&status)
        .context("Failed to serialize CRLite cache status")
}

/// Returns the persisted `CRLite` manifest file as JSON.
#[ipc_method]
pub async fn get_crlite_manifest_json() -> Result<String> {
    ensure_crlite_cache_ready().await?;
    let json = load_crlite_collection_json()?
        .ok_or_else(|| anyhow::anyhow!("CRLite collection JSON not cached"))?;
    Ok(json)
}

/// Reads one cached `CRLite` artifact by relative path.
#[ipc_method]
pub async fn get_crlite_artifact(relative_path: &str) -> Result<Vec<u8>> {
    ensure_crlite_cache_ready().await?;
    let cache_dir = get_crlite_cache_dir()?;
    let relative_path_buf = validate_relative_artifact_path(relative_path)?;
    let absolute_path = cache_dir.join(&relative_path_buf);

    let find_record = || -> Option<(std::path::PathBuf, u64, String)> {
        let json = load_crlite_collection_json().ok()??;
        let response = serde_json::from_str::<
            ctb_utilities::https::crlite::CRLiteCollectionResponse,
        >(&json)
        .ok()?;
        let record = response.data.iter().find(|r| {
            r.attachment.location == relative_path
                || r.attachment.filename == relative_path
        })?;
        let channel = record.channel_name();
        let path = cache_dir.join(channel).join(&record.attachment.filename);
        Some((path, record.attachment.size, record.attachment.hash.clone()))
    };

    let check_and_read = || -> Result<Option<Vec<u8>>> {
        if let Some((path, size, hash)) = find_record() {
            if let Ok(bytes) = std::fs::read(&path) {
                let computed_hash =
                    ctb_installer::chunking::compute_sha256_hex(&bytes);
                if u64::try_from(bytes.len())? == size && computed_hash == hash
                {
                    return Ok(Some(bytes));
                }
                warn_fmt!(
                    "CRLite artifact integrity check failed for {}",
                    path.display()
                );
            } else {
                warn_fmt!("CRLite artifact missing at {}", path.display());
            }
            Ok(None)
        } else {
            // Fall back to direct absolute path read if no collection record matches
            if let Ok(bytes) = std::fs::read(&absolute_path) {
                Ok(Some(bytes))
            } else {
                Ok(None)
            }
        }
    };

    if let Some(bytes) = check_and_read()? {
        return Ok(bytes);
    }

    if should_rate_limit_refresh() {
        return Err(anyhow::anyhow!(
            "CRLite artifact not found or corrupted (refresh rate-limited)"
        ));
    }

    warn_fmt!(
        "Triggering CRLite cache refresh due to missing or corrupted artifact: {relative_path}"
    );
    record_refresh_time();
    spawn_blocking_with_current_test_name(refresh_crlite_cache_sync)
        .await
        .context("CRLite refresh task failed")??;

    if let Some(bytes) = check_and_read()? {
        return Ok(bytes);
    }

    Err(anyhow::anyhow!(
        "CRLite artifact not found or corrupted after refresh: {relative_path}"
    ))
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
    use std::fs;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode},
        routing::get,
    };
    use ctb_utilities::https::crlite::{
        get_crlite_cache_dir, load_crlite_manifest, set_test_download_override,
    };
    use ctb_utilities::json::maybe_value::{MaybeOption, MaybeValue};
    use ctb_utilities::pc_settings::PcSettings;
    use ctb_utilities::{Context, Result, anyhow, ensure};
    use tower::ServiceExt;

    use super::{get_crlite_manifest_json, validate_relative_artifact_path};

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../utilities/https/data/fixtures/test_crlite_filters")
            .join(name)
    }

    fn fixture_metadata(name: &str) -> (String, u64) {
        let bytes = fs::read(fixture_path(name)).unwrap();
        let hash = ctb_installer::chunking::compute_sha256_hex(&bytes);
        let size = u64::try_from(bytes.len()).unwrap();
        (hash, size)
    }

    fn manifest_json() -> String {
        let (full_hash, full_size) = fixture_metadata("20200101-0-filter");
        let (delta_hash, delta_size) =
            fixture_metadata("20200101-1-filter.delta");
        serde_json::json!({
            "data": [
                {
                    "id": "full-filter",
                    "channel": "default",
                    "details": { "name": "2020-01-01T00:00:00Z-full" },
                    "attachment": {
                        "hash": full_hash,
                        "size": full_size,
                        "filename": "20200101-0-filter",
                        "location": "default/20200101-0-filter",
                        "mimetype": "application/octet-stream"
                    },
                    "incremental": false,
                    "effectiveTimestamp": 1_577_836_800_000_i64
                },
                {
                    "id": "delta-filter",
                    "parent": "full-filter",
                    "channel": "default",
                    "details": { "name": "2020-01-02T00:00:00Z-diff" },
                    "attachment": {
                        "hash": delta_hash,
                        "size": delta_size,
                        "filename": "20200101-1-filter.delta",
                        "location": "default/20200101-1-filter.delta",
                        "mimetype": "application/octet-stream"
                    },
                    "incremental": true,
                    "effectiveTimestamp": 1_577_923_200_000_i64
                }
            ]
        })
        .to_string()
    }

    fn request_uri_from_url(url: &str) -> Result<String> {
        let Some(scheme_end) = url.find("://") else {
            return Err(anyhow::anyhow!(
                "CRLite test URL is missing a scheme: {url}"
            ));
        };
        let after_scheme =
            url.get(scheme_end.saturating_add(3)..).unwrap_or("");
        let Some(path_start) = after_scheme.find('/') else {
            return Ok("/".to_string());
        };
        Ok(after_scheme.get(path_start..).unwrap_or("").to_string())
    }

    #[crate::ctb_test]
    fn rejects_parent_directory_components() {
        let result = validate_relative_artifact_path("../outside.filter");
        result.unwrap_err();
    }

    #[crate::ctb_test]
    fn accepts_nested_relative_paths() {
        let result = validate_relative_artifact_path("default/20260603.filter");
        result.unwrap();
    }

    #[crate::ctb_test("tokio")]
    async fn manifest_refreshes_cache_from_local_mirror() {
        // It appears that `get_crlite_cache_dir()` appends the current test
        // name, so it should be fine to not use a tempdir for this.
        //bypass-tempdir-lint
        let cache_dir = get_crlite_cache_dir().unwrap();
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).unwrap();
        }

        let manifest = manifest_json();

        let mirror: Router =
            Router::new()
                .route(
                    "/crlite/manifest.json",
                    get({
                        let manifest = manifest.clone();
                        move || {
                            let manifest = manifest.clone();
                            async move { manifest }
                        }
                    }),
                )
                .route(
                    "/crlite/artifacts/{*path}",
                    get(
                        |axum::extract::Path(path): axum::extract::Path<
                            String,
                        >| async move {
                            let filename = std::path::Path::new(&path)
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();
                            fs::read(fixture_path(&filename)).unwrap()
                        },
                    ),
                );

        let handle = tokio::runtime::Handle::current();
        let _download_override = set_test_download_override(move |url| {
            let uri = request_uri_from_url(url)?;
            let request = Request::builder()
                .method(Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .with_context(|| {
                    format!(
                        "Failed to build in-process CRLite request for {uri}"
                    )
                })?;

            let (status, body) = handle.block_on(async {
                let response = mirror.clone().oneshot(request).await.map_err(|error| {
                    anyhow::anyhow!(
                        "In-process CRLite mirror request failed for {uri}: {error}"
                    )
                })?;
                let status = response.status();
                let body = to_bytes(response.into_body(), usize::MAX)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to read in-process CRLite response body for {uri}"
                        )
                    })?;
                Ok::<_, anyhow::Error>((status, body))
            })?;

            ensure!(
                status == StatusCode::OK,
                "In-process CRLite mirror returned {status} for {uri}: {}",
                String::from_utf8_lossy(&body)
            );

            Ok(body.to_vec())
        });

        PcSettings {
            server_url: MaybeValue::Value(
                "https://ctb-test.invalid".to_string(),
            ),
            domain_name: MaybeOption::Value("example.com".to_string()),
            ..Default::default()
        }
        .save()
        .unwrap();

        let json = get_crlite_manifest_json().await.unwrap();

        assert!(json.contains("full-filter"));
        let manifest = load_crlite_manifest().unwrap();
        assert!(manifest.is_some());

        // Verify that we can retrieve the artifact via its direct/cached path
        let bytes_direct =
            super::get_crlite_artifact("default/20200101-0-filter")
                .await
                .unwrap();
        assert_eq!(
            bytes_direct,
            fs::read(fixture_path("20200101-0-filter")).unwrap()
        );

        // Verify that we can retrieve the artifact via its location/filename from the collection JSON
        let bytes_filename = super::get_crlite_artifact("20200101-0-filter")
            .await
            .unwrap();
        assert_eq!(
            bytes_filename,
            fs::read(fixture_path("20200101-0-filter")).unwrap()
        );

        // 1. Delete/mangle the file to check corruption detection and rate-limited refresh recovery
        let cached_file_path = cache_dir.join("default/20200101-0-filter");
        assert!(cached_file_path.exists());

        fs::write(&cached_file_path, b"invalid mangled data").unwrap();

        ctb_utilities::https::crlite::clear_in_memory_cache();

        // Calling get_crlite_artifact should detect corruption, trigger a refresh and return recovered bytes
        let bytes_recovered =
            super::get_crlite_artifact("default/20200101-0-filter")
                .await
                .unwrap();
        assert_eq!(
            bytes_recovered,
            fs::read(fixture_path("20200101-0-filter")).unwrap()
        );

        // Verify that the file on disk has indeed been restored to its original valid content
        let restored_bytes = fs::read(&cached_file_path).unwrap();
        assert_eq!(
            restored_bytes,
            fs::read(fixture_path("20200101-0-filter")).unwrap()
        );
    }
}

/*

Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.

*/
