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

//! CLI execution helpers for Internet Archive commands.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Context;
use clap::Subcommand;
use std::path::PathBuf;
use crate as ctb_formats_internetarchive;


#[derive(Subcommand, Debug)]
pub enum IACommand {
    /// Verify a local Internet Archive item directory against its files XML.
    Verify {
        /// Path to the item directory, or omit to use the current directory.
        item_path: Option<PathBuf>,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch the current files XML from archive.org instead of using a
        /// local copy.
        #[arg(long)]
        check_live: bool,
        /// Only verify files with source="original".
        #[arg(long)]
        original: bool,
    },
    /// Print the expected sha1 for a file in an Internet Archive item.
    Sha1 {
        /// A local file path, item/file path, or archive.org download URL.
        target: String,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch live metadata instead of reading a local files XML.
        #[arg(long)]
        check_live: bool,
    },
    /// Print the expected md5 for a file in an Internet Archive item.
    Md5 {
        /// A local file path, item/file path, or archive.org download URL.
        target: String,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch live metadata instead of reading a local files XML.
        #[arg(long)]
        check_live: bool,
    },
    /// Check whether an item contains a particular file.
    #[command(name = "contains")]
    Contains {
        /// An item identifier, item/file path, or archive.org URL.
        target: String,
        /// File path inside the item to check for.
        desired_file: String,
    },
    /// List item files one per line.
    #[command(name = "listplain")]
    ListPlain {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch live item metadata as pretty JSON.
    Metadata {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch the live `_files.xml` document.
    #[command(name = "filesxml")]
    FilesXml {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch the live `_meta.xml` document.
    #[command(name = "metaxml")]
    MetaXml {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Download an item or file from archive.org.
    Download {
        /// An item identifier, item/file path, or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Only download files with source="original".
        #[arg(long)]
        original: bool,
        /// Show progress during download
        #[arg(long, overrides_with = "no_progress")]
        progress: bool,
        /// Suppress progress during download
        #[arg(long, overrides_with = "progress")]
        no_progress: bool,
    },
    /// Download a single file and write it to stdout.
    #[command(name = "downloadAsStream")]
    DownloadAsStream {
        /// An item/file path or archive.org download URL.
        target: String,
    },
    /// Download a single file into the current directory.
    #[command(name = "downloadHere")]
    DownloadHere {
        /// An item/file path or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Show progress during download
        #[arg(long, overrides_with = "no_progress")]
        progress: bool,
        /// Suppress progress during download
        #[arg(long, overrides_with = "progress")]
        no_progress: bool,
    },
    /// Download an item or file, then verify the downloaded content.
    Checkeddl {
        /// An item identifier, item/file path, or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Only download and verify files with source="original".
        #[arg(long)]
        original: bool,
        /// Show progress during download
        #[arg(long, overrides_with = "no_progress")]
        progress: bool,
        /// Suppress progress during download
        #[arg(long, overrides_with = "progress")]
        no_progress: bool,
    },
}


pub fn run_internetarchive(cmd: &IACommand) -> Result<ToolResult> {
    match cmd {
            IACommand::Verify {
                item_path,
                identifier,
                check_live,
                original,
            } => {
                let item_path = if let Some(item_path) = item_path {
                    item_path.clone()
                } else {
                    std::env::current_dir()
                        .context("Failed to get current directory")?
                };
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::verify(
                        item_path.as_path(),
                        identifier.as_deref(),
                        *check_live,
                        *original,
                    )?,
                ))
            }
            IACommand::Sha1 {
                target,
                identifier,
                check_live,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::iasha1(
                    target,
                    identifier.as_deref(),
                    *check_live,
                )?,
            )),
            IACommand::Md5 {
                target,
                identifier,
                check_live,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::iamd5(
                    target,
                    identifier.as_deref(),
                    *check_live,
                )?,
            )),
            IACommand::Contains {
                target,
                desired_file,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::contains(target, desired_file)?,
            )),
            IACommand::ListPlain { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::listplain(target)?,
            )),
            IACommand::Metadata { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::metadata(target)?,
            )),
            IACommand::FilesXml { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::filesxml(target)?,
            )),
            IACommand::MetaXml { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::metaxml(target)?,
            )),
            IACommand::Download {
                target,
                output_dir,
                original,
                progress,
                no_progress,
            } => {
                let progress_reporter =
                    ctb_utilities::ui::progress::Progress::from_flags(
                        *progress,
                        *no_progress,
                    );
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download(
                        target,
                        output_dir.as_deref(),
                        *original,
                        progress_reporter,
                    )?,
                ))
            }
            IACommand::DownloadAsStream { target } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download_as_stream(target)?,
                ))
            }
            IACommand::DownloadHere {
                target,
                output_dir,
                progress,
                no_progress,
            } => {
                let progress_reporter =
                    ctb_utilities::ui::progress::Progress::from_flags(
                        *progress,
                        *no_progress,
                    );
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download_here(
                        target,
                        output_dir.as_deref(),
                        progress_reporter,
                    )?,
                ))
            }
            IACommand::Checkeddl {
                target,
                output_dir,
                original,
                progress,
                no_progress,
            } => {
                let progress_reporter =
                    ctb_utilities::ui::progress::Progress::from_flags(
                        *progress,
                        *no_progress,
                    );
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::checkeddl(
                        target,
                        output_dir.as_deref(),
                        *original,
                        progress_reporter,
                    )?,
                ))
            }
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

#[crate::ctb_test]
fn test_ia_cli() {

}

}