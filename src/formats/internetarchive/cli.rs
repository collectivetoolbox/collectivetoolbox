#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

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

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

#[crate::ctb_test]
fn
() {

}

}