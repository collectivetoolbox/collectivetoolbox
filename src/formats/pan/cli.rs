#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

            let data = read_file_or_stdin(pan_file.as_path())?;
            let include_header = if *no_header { false } else { *header };
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252"
                | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
                    ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Utf8,
            };
            let delim = match delimiter.to_ascii_lowercase().replace('_', "-").as_str() {
                "tab" | "tabs" | "tsv" => {
                    ctb_formats_pan::output::PanExportDelimiter::Tabs
                }
                "tab-no-quotes"
                | "tabs-no-quotes"
                | "tab-without-quotes"
                | "tabs-without-quotes"
                | "tabs-w/o-quotes"
                | "tsv-no-quotes" => {
                    ctb_formats_pan::output::PanExportDelimiter::TabsWithoutQuotes
                }
                "wordperfect" | "wp" => {
                    ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                }
                "commas" | "csv" => ctb_formats_pan::output::PanExportDelimiter::Commas,
                _ => bail!("Unknown format for --delimiter: {delimiter:?}. Valid options are: commas, tabs, tabs-no-quotes, wordperfect"),
            };
            let opts = ctb_formats_pan::output::PanCsvOptions {
                output_patterns: *patterns,
                truncate_multiline: !*keep_multiline,
                include_header,
                encoding: enc,
                delimiter: delim,
                crlf: *crlf
                    || (delim != ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                        && enc == ctb_formats_pan::output::PanCsvEncoding::Windows),
                replicate_double_encoding: *replicate_double_encoding,
                run_startup_procedure: *run_startup_procedure,
            };
            let output =
                ctb_formats_pan::output::pan_to_csv_with_options(&data, &opts)?;
            Ok(ToolResult::immediate_ok(output))

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

#[crate::ctb_test]
fn
() {

}

}