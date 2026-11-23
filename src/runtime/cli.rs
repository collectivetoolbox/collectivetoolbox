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

//! CLI argument parsing and execution dispatch for `ctoolbox run`.
//!
//! Compiles and executes an arbitrary Document Character (`Dc`) document or
//! StageL 3 / StageR 3 / WAT / `.WASM` program through the `Dc 1936` execution
//! kernel (`[Dc 1938 <pin> [Dc 2608 <quoted_doc>]]`), and renders the evaluated
//! semantic `DcMstring` output via `ctb-renderer` (`RenderTarget::Teletype`
//! plain-text UTF-8 lowering by default).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

use ctb_formats_conversion::convert_api::{
    DcConversionOptions, FormatSpec, SupportedDcInputFormat,
    convert_bytes_to_dcmstring,
};
use ctb_formats_conversion::convert_bytes::format_supported_input_formats_table;
use ctb_formats_dcstring::{
    DcChar, DcMstr, DcMstring, SeedGraphIndex, SyntaxRuleResolver,
};
use ctb_formats_utilities::{CliInputSourceArgs, CliOutputDestArgs};
use ctb_renderer::{
    RenderMode, RenderSettings, RenderTarget, render_dc_document_with_resolver,
};
use ctb_utilities::FormatId;

use crate::{ExecutionContext, SystemSupervisorAdmission};

/// Global static lazy help footer for `ctoolbox run`.
pub static RUN_AFTER_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        let in_formats = format_supported_input_formats_table(
            "Supported document input formats (--from):",
        );
        format!(
            "{in_formats}\n\nAdditional executable source formats (--from or by extension):\n  sr3 (f657, .sr3): StageR 3 module\n  sl3 (f656, .sl3): StageL 3 module\n  elad (f17, .elad, .emd): EITE Language in ASCII documents\n  ems (f914, .ems, .elq): EMS short-Dc sequence\n  dems (f915, .dems, .delq): Documented EMS short-Dc sequence\n  wat (.wat): WebAssembly Text Format\n\nExamples:\n  $ ctoolbox run -f hello-world.emd\n  $ ctoolbox run -f idiomatic-hello-world.ems\n  $ ctoolbox run --from dcsh \"256 258 260 262 264 263 57 86 93 93 96 261 259\"\n  $ ctoolbox run -f prog.sr3 --program-input \"world\""
        )
    });

/// CLI rendering target selector for `ctoolbox run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum RunTarget {
    /// Character-based plain-text UTF-8 output (`RenderTarget::Teletype`).
    #[default]
    #[value(name = "teletype")]
    Teletype,
    /// Video terminal with cursor control and escape sequences (not yet implemented).
    #[value(name = "videoterminal")]
    Videoterminal,
    /// Web-based HTML/CSS/JS output (not yet implemented).
    #[value(name = "web")]
    Web,
    /// Raster bitmap graphics output (not yet implemented).
    #[value(name = "raster")]
    Raster,
    /// Vector graphics output (not yet implemented).
    #[value(name = "vector")]
    Vector,
    /// Print-ready output (not yet implemented).
    #[value(name = "print")]
    Print,
}

impl From<RunTarget> for RenderTarget {
    fn from(value: RunTarget) -> Self {
        match value {
            RunTarget::Teletype => Self::Teletype,
            RunTarget::Videoterminal => Self::Videoterminal,
            RunTarget::Web => Self::Web,
            RunTarget::Raster => Self::Raster,
            RunTarget::Vector => Self::Vector,
            RunTarget::Print => Self::Print,
        }
    }
}

/// Parsed `--from` specification for `ctoolbox run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunInputFormat {
    /// Registered `FormatId` (`SupportedDcInputFormat`, `StageR3`, `StageL3`, `Elad`, `Ems`, `Dems`, etc.).
    Format(FormatId),
    /// WebAssembly Text Format (`.wat`).
    Wat,
}

/// Parses a CLI `--from` flag into [`RunInputFormat`].
///
/// # Errors
/// Returns a human-readable format table string if `raw` is unknown.
pub fn parse_run_input_format(
    raw: &str,
) -> std::result::Result<RunInputFormat, String> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("wat") {
        return Ok(RunInputFormat::Wat);
    }
    if let Ok(supported) = trimmed.parse::<SupportedDcInputFormat>() {
        return Ok(RunInputFormat::Format(supported.to_format_id()));
    }
    if let Some(fid) = FormatId::from_shorthand(trimmed)
        .or_else(|| FormatId::from_ident(trimmed))
    {
        return Ok(RunInputFormat::Format(fid));
    }
    Err(format_supported_input_formats_table(&format!(
        "Invalid input format '{raw}'. Specify --from with sr3, sl3, elad, ems, dems, wat, or one of:"
    )))
}

/// CLI arguments for `ctoolbox run`.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(after_help = RUN_AFTER_HELP.as_str())]
pub struct RunArgs {
    /// Document source text. If omitted, reads from `--file` or standard input.
    pub input: Option<String>,

    /// Input format (autodetected if omitted; accepts nickname, `FormatId` name, or `f<N>` shorthand)
    #[arg(long = "from", value_parser = parse_run_input_format)]
    pub from: Option<RunInputFormat>,

    /// Parse input document in permissive mode (recovering from non-fatal format warnings)
    #[arg(long = "permissive")]
    pub permissive: bool,

    /// Parse input document in strict mode
    #[arg(long = "strict", conflicts_with = "permissive")]
    pub strict: bool,

    /// Input document file path (or `-` for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Output file path (or `-` for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file without prompt
    #[arg(short = 'F', long = "force")]
    pub force: bool,

    /// Optional UTF-8 payload passed to the executed program's input mailbox
    #[arg(long = "program-input")]
    pub program_input: Option<String>,

    /// Optional file path containing raw input bytes for the executed program's input mailbox
    #[arg(long = "program-input-file", conflicts_with = "program_input")]
    pub program_input_file: Option<PathBuf>,

    /// Rendering target (`teletype` plain-text UTF-8 output by default)
    #[arg(long = "target", value_enum, default_value_t = RunTarget::Teletype)]
    pub target: RunTarget,
}

impl RunArgs {
    /// Adapts input positional/file fields into [`CliInputSourceArgs`].
    #[must_use]
    pub fn input_source(&self) -> CliInputSourceArgs {
        CliInputSourceArgs {
            input: self.input.clone(),
            file: self.file.clone(),
        }
    }

    /// Adapts output destination fields into [`CliOutputDestArgs`].
    #[must_use]
    pub fn output_dest(&self) -> CliOutputDestArgs {
        CliOutputDestArgs {
            output: self.output.clone(),
            force: self.force,
        }
    }
}

fn load_executable_dc_document(
    bytes: &[u8],
    filename_hint: Option<&str>,
    from: Option<RunInputFormat>,
    permissive: bool,
) -> Result<DcMstring> {
    let ext_lower = filename_hint
        .and_then(|name| Path::new(name).extension())
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);

    let resolved = if let Some(explicit) = from {
        Some(explicit)
    } else if ext_lower.as_deref() == Some("wat") {
        Some(RunInputFormat::Wat)
    } else {
        ctb_formats_detection::detect_format_id(Some(bytes), filename_hint, None)
            .map(RunInputFormat::Format)
    };

    match resolved {
        Some(RunInputFormat::Wat) => {
            let src = std::str::from_utf8(bytes)
                .context("WAT input is not valid UTF-8")?;
            ctb_formats_dcstring::wat_to_dcmstring(src)
        }
        Some(RunInputFormat::Format(FormatId::StageR3)) => {
            let src = std::str::from_utf8(bytes)
                .context("StageR 3 (.sr3) input is not valid UTF-8")?;
            let sg = SeedGraphIndex::shared()?;
            let tokens = ctb_formats_dcstring::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                src, sg,
            )?;
            let mut doc = DcMstring::new();
            for tok in tokens {
                doc.push_char(tok);
            }
            Ok(doc)
        }
        Some(RunInputFormat::Format(FormatId::StageL3)) => {
            let src = std::str::from_utf8(bytes)
                .context("StageL 3 (.sl3) input is not valid UTF-8")?;
            let sg = SeedGraphIndex::shared()?;
            let tokens = ctb_formats_dcstring::stagel3::seed_linker::sl3_to_dc_tokens_with_seed_graph(
                src, sg,
            )?;
            let mut doc = DcMstring::new();
            for tok in tokens {
                doc.push_char(tok);
            }
            Ok(doc)
        }
        Some(RunInputFormat::Format(FormatId::Elad)) => {
            let (doc, log) =
                ctb_formats_dcstring::eite::elad_to_dcmstring_with_options(
                    bytes, permissive,
                )?;
            emit_format_log(&log);
            Ok(doc)
        }
        Some(RunInputFormat::Format(FormatId::Ems | FormatId::Dems)) => {
            let (doc, log) =
                ctb_formats_dcstring::eite::sems_to_dcmstring_with_options(
                    bytes, true,
                )?;
            emit_format_log(&log);
            Ok(doc)
        }
        Some(RunInputFormat::Format(fid)) => {
            let options = DcConversionOptions {
                format: if fid == FormatId::Text {
                    FormatSpec::Format(SupportedDcInputFormat::Utf8)
                } else {
                    FormatSpec::FormatId(fid)
                },
                allow_binary_encapsulation: false,
                permissive,
            };
            let res =
                convert_bytes_to_dcmstring(bytes, filename_hint, &options)?;
            emit_format_log(&res.log);
            Ok(res.document)
        }
        None => {
            let options = DcConversionOptions {
                format: FormatSpec::Auto,
                allow_binary_encapsulation: false,
                permissive,
            };
            let res =
                convert_bytes_to_dcmstring(bytes, filename_hint, &options)?;
            emit_format_log(&res.log);
            Ok(res.document)
        }
    }
}

fn emit_format_log(log: &ctb_formats_utilities::FormatLog) {
    log.auto_log();
    if log.has_errors() {
        eprintln!("{}", log.format_errors().trim_end());
    }
    if log.has_warnings() {
        eprintln!("{}", log.format_warnings().trim_end());
    }
}

/// Executes an already-loaded [`DcMstr`] document through the `Dc 1936` execution
/// kernel and renders its combined yielded frames and completion output with
/// `settings`.
///
/// # Errors
/// Returns an error if compilation, execution, resource limits, or rendering fail.
pub fn run_dc_document_to_rendered_bytes(
    ordinary_doc: &DcMstr,
    program_input: &DcMstr,
    settings: &RenderSettings,
) -> Result<Vec<u8>> {
    ctb_wasm_substitutes::ensure_registered();
    let default_resolver =
        ctb_formats_dcstring::syntax::default_syntax_resolver();
    let adm = SystemSupervisorAdmission::admit_cli_execution_kernel()?;
    let mut ctx = ExecutionContext::new(false)
        .with_system_supervisor_admission(adm);
    let trace = ctx.execute_dc_document_default_with_trace(
        ordinary_doc,
        program_input,
        default_resolver.as_ref(),
    )?;
    ensure!(
        !trace.result.broke_out_early,
        "Dc document execution terminated early due to resource or permission denial: {:?}",
        trace.events
    );

    let mut combined_output = DcMstring::new();
    for frame in &trace.result.frames {
        for chunk in frame.chunks() {
            combined_output.push_chunk(&chunk)?;
        }
    }
    for chunk in trace.result.output.chunks() {
        combined_output.push_chunk(&chunk)?;
    }

    let opt_pin = SystemSupervisorAdmission::scan_first_pin_description(
        ordinary_doc,
        default_resolver.as_ref(),
    )?;
    let pinned_holder = if let Some((_, pin_id)) = opt_pin {
        Some(ctb_formats_dcstring::syntax::syntax_resolver_at_most_recent(
            pin_id,
        )?)
    } else {
        None
    };
    let render_resolver: &dyn SyntaxRuleResolver =
        if let Some(ref pinned) = pinned_holder {
            pinned.as_ref()
        } else {
            default_resolver.as_ref()
        };

    render_dc_document_with_resolver(&combined_output, settings, render_resolver)
}

/// Executes the `ctoolbox run` CLI subcommand.
///
/// Returns `Ok(Some(bytes))` when output should be written to stdout, or
/// `Ok(None)` when output was written directly to `-o`/`--output`.
///
/// # Errors
/// Returns an error if reading inputs, parsing the document, executing via
/// `Dc 1936`, rendering, or writing output fails.
pub fn execute_cli_run<FRead>(
    args: RunArgs,
    read_data: FRead,
) -> Result<Option<Vec<u8>>>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let input_bytes = args.input_source().read_bytes(&read_data)?;
    let filename_hint = args.file.as_deref().and_then(|p| {
        if p == Path::new("-") {
            None
        } else {
            p.file_name().and_then(|n| n.to_str())
        }
    });
    let permissive = args.permissive && !args.strict;
    let ordinary_doc = load_executable_dc_document(
        &input_bytes,
        filename_hint,
        args.from,
        permissive,
    )?;

    let mut program_input_doc = DcMstring::new();
    if let Some(ref prog_in_str) = args.program_input {
        for ch in prog_in_str.chars() {
            program_input_doc.push_char(DcChar::from(ch));
        }
    } else if let Some(ref prog_in_path) = args.program_input_file {
        let raw_in = read_data(prog_in_path)?;
        let (parsed_in, _) =
            ctb_formats_dcstring::utf8::utf8_to_dcmstring_with_options(
                &raw_in, true,
            )?;
        program_input_doc = parsed_in;
    }

    let render_settings = RenderSettings {
        mode: RenderMode::Immediate,
        target: RenderTarget::from(args.target),
    };
    let rendered = run_dc_document_to_rendered_bytes(
        &ordinary_doc,
        &program_input_doc,
        &render_settings,
    )?;
    args.output_dest().write_or_stdout(rendered)
}

#[cfg(test)]
#[allow(
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

    fn workspace_fixture_path(rel: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel)
    }

    #[crate::ctb_test]
    fn runs_hello_world_emd_and_ems_fixtures_to_teletype_utf8() -> Result<()> {
        let emd_path = workspace_fixture_path(
            "src/formats/dct_el/fixtures/examples/example-documents/hello-world/hello-world.emd",
        );
        let out_emd = execute_cli_run(
            RunArgs {
                file: Some(emd_path),
                ..Default::default()
            },
            |p| Ok(std::fs::read(p)?),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out_emd)?, "Hello, World!\n");

        let idiom_emd_path = workspace_fixture_path(
            "src/formats/dct_el/fixtures/examples/example-documents/hello-world/idiomatic-hello-world.emd",
        );
        let out_idiom_emd = execute_cli_run(
            RunArgs {
                file: Some(idiom_emd_path),
                ..Default::default()
            },
            |p| Ok(std::fs::read(p)?),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out_idiom_emd)?, "Hello, World!\n\n");

        let idiom_ems_path = workspace_fixture_path(
            "src/formats/dct_el/fixtures/examples/example-documents/hello-world/idiomatic-hello-world.ems",
        );
        let out_idiom_ems = execute_cli_run(
            RunArgs {
                file: Some(idiom_ems_path),
                ..Default::default()
            },
            |p| Ok(std::fs::read(p)?),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out_idiom_ems)?, "Hello, World!\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn runs_lexical_scope_assignment_and_equiv_rep_dc_document() -> Result<()> {
        // Document:
        // [greeting] = "Outer"
        // 541 (Enter scope)
        //   [greeting] = 397 [list * "Inner", 6 f199 128 7]
        //   256 (lang.say) 258 (540 [greeting]) 259
        // 542 (Exit scope)
        // 256 (lang.say) 258 (540 [greeting]) 259
        let dcsh = concat!(
            "235 ",
            "270 u67 u72 u65 u65 u74 271 269 260 262 264 263 u4f u75 u74 u65 u72 261 ",
            "541 ",
            "270 u67 u72 u65 u65 u74 271 269 397 310 275 260 262 264 263 u49 u6e u6e u65 u72 261 312 6 f199 128 7 311 ",
            "256 258 540 270 u67 u72 u65 u65 u74 271 259 ",
            "542 ",
            "256 258 540 270 u67 u72 u65 u65 u74 271 259 ",
            "2239 u4f u4b u0a ",
            "236"
        );
        let out = execute_cli_run(
            RunArgs {
                input: Some(dcsh.to_string()),
                from: Some(RunInputFormat::Format(FormatId::Dcsh)),
                ..Default::default()
            },
            |_| Ok(Vec::new()),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out)?, "Inner\nOuter\nOK\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn runs_historical_dc_641_pinned_lang_say_document() -> Result<()> {
        // Historical pin l8589958834 (predates StageR 3 Dc 2608 and f19) + Dc 235 .. Dc 256 .. Dc 236
        // using short Dcs for "Hi!" (57, 90, 19) + Dc 255 escape test.
        let dcsh = concat!(
            "641 l8589958834 ",
            "235 ",
            "256 258 260 262 264 263 57 90 19 261 259 ",
            "255 u4f u4b u0a ",
            "236"
        );
        let out = execute_cli_run(
            RunArgs {
                input: Some(dcsh.to_string()),
                from: Some(RunInputFormat::Format(FormatId::Dcsh)),
                ..Default::default()
            },
            |_| Ok(Vec::new()),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out)?, "Hi!\nOK\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn runs_stager3_and_stagel3_programs_with_program_input() -> Result<()> {
        let sr3_src = concat!(
            "ns app/cli_hello.\n",
            "pub v/l/u8/step l/u8/input\n",
            "    concat concat b\"Hello, \" /input b\"!/n\".\n",
        );
        let out_sr3 = execute_cli_run(
            RunArgs {
                input: Some(sr3_src.to_string()),
                from: Some(RunInputFormat::Format(FormatId::StageR3)),
                program_input: Some("CLI".to_string()),
                ..Default::default()
            },
            |_| Ok(Vec::new()),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out_sr3)?, "Hello, CLI!\n");

        let sl3_src =
            "v/l/u8/step l/u8/input :: concat b\"sl3:\" concat /input b\"/n\".\n";
        let out_sl3 = execute_cli_run(
            RunArgs {
                input: Some(sl3_src.to_string()),
                from: Some(RunInputFormat::Format(FormatId::StageL3)),
                program_input: Some("ok".to_string()),
                ..Default::default()
            },
            |_| Ok(Vec::new()),
        )?
        .unwrap();
        assert_eq!(String::from_utf8(out_sl3)?, "sl3:ok\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn rejects_unimplemented_videoterminal_target() {
        let err = execute_cli_run(
            RunArgs {
                input: Some("Hello".to_string()),
                from: Some(RunInputFormat::Format(FormatId::Utf8)),
                target: RunTarget::Videoterminal,
                ..Default::default()
            },
            |_| Ok(Vec::new()),
        )
        .unwrap_err();
        assert!(err.to_string().contains("Videoterminal"));
    }
}
