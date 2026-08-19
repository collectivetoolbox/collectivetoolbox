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

//! Tracing-based logging setup.
//! Useful documentation: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html#per-layer-filtering>

use std::{env, fmt, io, sync::OnceLock, time::SystemTime};

use crate::{is_in_test, pc_settings};
use anyhow::{Context, Result, anyhow};
use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
};
use tracing_appender::rolling::{self, Builder};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{
        FmtContext, FormatEvent, FormatFields, FormattedFields, format::Writer,
    },
    prelude::*,
    registry::LookupSpan,
};

use crate::{
    COLUMN_UUID_DELIM, pc_settings::get_bool_setting, storage::get_storage_dir,
};

/// Custom event formatter. Creates a fresh timestamp per event.
/// Avoids panics if span extensions are missing.
#[derive(Clone)]
struct LogEventFormatter {
    pid: u64,
    sub_index: Option<String>,
    service_name: Option<String>,
}

impl LogEventFormatter {
    fn format_timestamp() -> String {
        humantime::format_rfc3339_micros(SystemTime::now()).to_string()
    }
}

struct MyVisitor {
    file: Option<String>,
    line: Option<u32>,
}

impl Visit for MyVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "log.file" {
            self.file = Some(value.to_string());
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "log.line" {
            self.line = u32::try_from(value).ok();
        }
    }
    fn record_debug(&mut self, _field: &Field, _value: &dyn fmt::Debug) {
        // Ignore other fields
    }
}

fn get_workspace_filter(level: &str) -> String {
    workspace_filter::workspace_filter!(level).replace('-', "_")
}

fn extract_file_line(event: &Event) -> (Option<String>, Option<u32>) {
    let mut visitor = MyVisitor {
        file: None,
        line: None,
    };
    event.record(&mut visitor);
    (visitor.file, visitor.line)
}

impl<S, N> FormatEvent<S, N> for LogEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        // Module path (preferred) falls back to target, then "unknown"
        // Reason for fallback: synthetic log events missing a module path or target display as "unknown"
        let target = meta
            .module_path()
            .or_else(|| Some(meta.target()))
            .unwrap_or("unknown");

        let (file, line) = extract_file_line(event);

        // File/line (fallback "unknown:0")
        // Reason for fallback: log events from macro callers without file metadata default to "unknown"
        let mut file = file
            .or_else(|| meta.file().map(std::string::ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        if file.starts_with("src/") {
            // Reason for fallback: empty or non-path file metadata string retains full name when trailing segment is absent
            file = file.rsplit('/').next().unwrap_or(&file).to_string();
        } else if meta.level() == &Level::DEBUG {
            return Ok(());
        }
        // Reason for fallback: log events without line metadata default to line 0
        let line = line.or_else(|| meta.line()).unwrap_or(0);

        let mut column = "";
        // Try to serialize the event into a string
        let mut buf = String::new();
        let column_writer = Writer::new(&mut buf);
        if ctx
            .field_format()
            .format_fields(column_writer, event)
            .is_ok()
        {
            if let Some((_, after)) = buf.split_once(COLUMN_UUID_DELIM) {
                // Reason for fallback: formatted log event without column text leaves column empty
                let col_text = after.split_whitespace().next().unwrap_or("");
                column = col_text;
            }
        }
        let column_formatted;
        if !column.is_empty() {
            column_formatted = format!(":{column}");
            column = &column_formatted;
        }

        let pid = self.pid;
        // Reason for fallback: main workspace process does not have a subprocess index
        let sub_idx = self.sub_index.as_deref().unwrap_or("");
        // Reason for fallback: main service context defaults to user service target
        let svc_name = self.service_name.as_deref().unwrap_or("user");
        let ts = Self::format_timestamp();

        write!(
            &mut writer,
            "[{ts} {} {file}:{line}{column} {target} {sub_idx}:{pid}/{svc_name}] ",
            meta.level()
        )?;

        // Span chain (root -> current)
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;
                if let Some(fields) = span
                    .extensions()
                    .get::<FormattedFields<N>>()
                    .filter(|f| !f.is_empty())
                {
                    write!(writer, "{{{fields}}}")?;
                }
                write!(writer, ": ")?;
            }
        }

        // Event fields
        let mut event_buf = String::new();
        {
            let mut tmp_writer = Writer::new(&mut event_buf);
            ctx.field_format()
                .format_fields(tmp_writer.by_ref(), event)?;
        }
        if let Some((before, after)) = event_buf.split_once(COLUMN_UUID_DELIM) {
            // Remove column number: split at first whitespace after delimiter
            let after_trimmed = after
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            write!(writer, "{before}{after_trimmed}")?;
        } else {
            write!(writer, "{event_buf}")?;
        }
        writeln!(writer)
    }
}

/// Builds the filter directive string based on build configuration.
fn build_default_filter_directives() -> String {
    // let crate_name = crate_name!();

    if cfg!(debug_assertions) {
        // Use an explicit "warn" root to keep noise down, override interesting crates to debug
        let crate_filter = get_workspace_filter("debug");

        format!(
            "warn,{crate_filter},tower_http=debug,hyper=warn,axum::rejection=trace"
        )
    } else {
        // For some reason using "error" root hides web log warnings (404 etc)
        let crate_filter = get_workspace_filter("warn");

        format!(
            "warn,{crate_filter},tower_http=debug,hyper=error,axum::rejection=warn"
        )
    }
}

/// Creates an `EnvFilter` from either `RUST_LOG` or the default directives.
/// Returns the filter and the directive string used.
/// FIXME: It looks like only the directives variable here is actually used.
fn create_env_filter() -> (EnvFilter, String) {
    let default_directives = build_default_filter_directives();

    // Check if RUST_LOG is set and non-empty
    let mut directives = match env::var("RUST_LOG") {
        Ok(val) if !val.trim().is_empty() => val,
        _ => default_directives.clone(),
    };

    if !directives.contains("turso_core=") {
        directives = format!("{directives},turso_core=info");
    }

    // Use the builder pattern to properly parse directives
    // Reason for fallback: invalid custom log directives fall back to default level directives parse_lossy
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .parse(&directives)
        .unwrap_or_else(|_| {
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .parse_lossy(&default_directives)
        });

    (filter, directives)
}

#[tracing::instrument]
pub fn setup_logger(
    subprocess_index: String,
    service_name: String,
) -> Result<()> {
    let pid = u64::from(std::process::id());

    /*
    - When running tests or in a debug build (when running tests, output should only be shown in the terminal for failed tests, as with normal test output.):
        - Warnings from dependencies should be logged to the appropriate files and printed to the terminal.
        - Debug logging from my own application should be logged to the appropriate files and printed to the terminal. For this I'm using my utility module's wrappers around log::debug, etc. which seems to be working when running tests, but not when running it as a build.
        - HTTP(s) requests should be logged to the appropriate file and printed to the terminal.
    - When running a release build:
        - Errors from dependencies should be printed to the terminal.
        - Warnings from my own application should be printed to the terminal.
        - HTTP(s) requests should be printed to the terminal.
    */

    // Get the directive string once, then create separate filters for each layer
    let (_, directives) = create_env_filter();

    // File logging only in debug (including tests). Release: only stderr (per specification).
    let stderr_formatter = LogEventFormatter {
        pid,
        sub_index: Some(subprocess_index.clone()),
        service_name: Some(service_name.clone()),
    };

    // Optional JSON structured logging
    // let json_layer = tracing_subscriber::fmt::layer()
    //     .json()
    //     .with_current_span(true)
    //     .with_span_list(true)
    //     .with_filter(env_filter.clone());

    let mut layers = Vec::new();

    // Some tests spawn real subprocesses and need to assert stderr/stdout
    // output. Allow forcing "real" stderr logging even when invoked from a
    // test harness.
    let force_stderr_logs = std::env::var("CTB_FORCE_STDERR_LOGS")
        .ok()
        .is_some_and(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        });

    // Build subscriber
    if cfg!(debug_assertions)
        || get_bool_setting(pc_settings::PcSettingBoolKey::LogStackFile)
    {
        layers.push(get_file_writer_layer(
            service_name,
            stderr_formatter.clone(),
            &directives,
        )?);
    }
    if is_in_test() && !force_stderr_logs {
        layers.push(get_mock_writer_layer(stderr_formatter.clone()));
    }
    let stderr_layer = get_stderr_layer(stderr_formatter, &directives)?;
    layers.push(stderr_layer);

    let result = tracing_subscriber::registry().with(layers).try_init();
    match result {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow!(
            "Failed to initialize logging in release mode: {err}"
        )),
    }
}

fn get_stderr_layer<S>(
    formatter: LogEventFormatter,
    directives: &str,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .parse(directives)
        .with_context(|| {
            format!("Failed to parse filter directives: {directives}")
        })?;

    let force_stderr_logs = std::env::var("CTB_FORCE_STDERR_LOGS")
        .ok()
        .is_some_and(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        });

    if is_in_test() && !force_stderr_logs {
        Ok(tracing_subscriber::fmt::layer()
            // work around https://github.com/rust-lang/rust/issues/90785
            .with_test_writer()
            .with_ansi(false)
            .event_format(formatter)
            .with_filter(env_filter)
            .boxed())
    } else {
        Ok(tracing_subscriber::fmt::layer()
            .with_writer(io::stderr)
            .with_ansi(false)
            .event_format(formatter)
            .with_filter(env_filter)
            .boxed())
    }
}

fn get_mock_writer_layer<S>(
    formatter: LogEventFormatter,
) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // FIXME
    /* let filter_directives = if cfg!(feature = "tracing-test-no-env-filter") {
        "trace".to_string()
    } else {
        get_workspace_filter("trace")
    }; */
    // println!("Using filter directives for mock writer: {filter_directives}");
    let filter_directives = "trace";
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::TRACE.into())
        .parse_lossy(filter_directives);

    let mock_writer = crate::testing::logging_test_internal::MockWriter::new(
        crate::testing::logging_test_internal::global_buf(),
    );

    tracing_subscriber::fmt::layer()
        .with_writer(mock_writer)
        .with_level(true)
        .with_ansi(false)
        .event_format(formatter)
        .with_filter(env_filter)
        .boxed()
}

fn get_file_writer_layer<S>(
    filename_base: String,
    formatter: LogEventFormatter,
    directives: &str,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let log_dir = get_storage_dir()
        .context("failed to resolve storage dir for logs")?
        .join("logs");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("creating log dir: {}", log_dir.display()))?;

    // if running tests, add "test" to filename
    let filename_base = if is_in_test() {
        format!("test-{filename_base}")
    } else {
        filename_base
    };

    let file_appender = Builder::new()
        .rotation(rolling::Rotation::DAILY)
        .max_log_files(7)
        .filename_prefix(filename_base.clone())
        .filename_suffix("log")
        .build(&log_dir)?;
    let (nb_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Keep guards alive for the process lifetime.
    static APPENDER_GUARDS: OnceLock<
        std::sync::Mutex<Vec<tracing_appender::non_blocking::WorkerGuard>>,
    > = OnceLock::new();
    let guards =
        APPENDER_GUARDS.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(mut guards) = guards.lock() {
        guards.push(guard);
    }

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .parse(directives)
        .with_context(|| {
            format!("Failed to parse filter directives: {directives}")
        })?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(nb_writer)
        .event_format(formatter)
        .with_ansi(false)
        .with_filter(env_filter)
        .boxed();

    Ok(file_layer)
}

/* -------------- OPTIONAL: AXUM / TOWER_HTTP TRACE CONFIG EXAMPLE -----------------

In your server builder (where you currently do:
    .layer(TraceLayer::new_for_http())
You can do something more explicit so you get clearer request lifecycle events:

use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;

pub fn make_app(state: AppState) -> Router {
    build_routes(state)
        .layer(
            TraceLayer::new_for_http()
                // Span created per request (INFO so it shows under tower_http=info)
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(false)
                )
                // Log a line when the request starts
                .on_request(
                    DefaultOnRequest::new()
                        .level(Level::INFO)
                )
                // Log a line when the response finishes (includes latency)
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .include_headers(false)
                )
        )
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
}

---------------------------------------------------------------------------------- */

struct ElfRollingWriter {
    inner: rolling::RollingFileAppender,
    last_date: Option<chrono::NaiveDate>,
    fields_str: String,
}

impl io::Write for ElfRollingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let today = chrono::Utc::now().date_naive();
        if self.last_date != Some(today) {
            self.last_date = Some(today);
            let _ = writeln!(self.inner, "#Version: 1.0");
            let _ = writeln!(self.inner, "#Fields: {}", self.fields_str);
        }
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

static ACCESS_LOG_WRITER: OnceLock<
    tracing_appender::non_blocking::NonBlocking,
> = OnceLock::new();

pub fn write_access_log_line(line: &str) {
    let writer = ACCESS_LOG_WRITER.get_or_init(|| {
        // Reason for fallback: storage dir lookup failure defaults to relative "logs" directory
        let log_dir = get_storage_dir().map_or_else(|_| std::path::PathBuf::from("logs"), |d| d.join("logs"));
        let _ = std::fs::create_dir_all(&log_dir);
        let filename_base = if is_in_test() { "test-webui-access" } else { "webui-access" };
        let file_appender = Builder::new()
            .rotation(rolling::Rotation::DAILY)
            .max_log_files(7)
            .filename_prefix(filename_base)
            .filename_suffix("log")
            .build(&log_dir);

        if let Ok(appender) = file_appender {
            let elf_writer = ElfRollingWriter {
                inner: appender,
                last_date: None,
                fields_str: "c-ip c-ident c-user date time cs-method cs-uri cs-version sc-status sc-bytes cs(User-Agent)".to_string(),
            };
            let (nb_writer, guard) = tracing_appender::non_blocking(elf_writer);
            // Keep guard alive
            static ACCESS_GUARDS: OnceLock<std::sync::Mutex<Vec<tracing_appender::non_blocking::WorkerGuard>>> = OnceLock::new();
            let guards = ACCESS_GUARDS.get_or_init(|| std::sync::Mutex::new(Vec::new()));
            if let Ok(mut g) = guards.lock() {
                g.push(guard);
            }
            nb_writer
        } else {
            let (sink_writer, _guard) = tracing_appender::non_blocking(std::io::sink());
            sink_writer
        }
    });

    use std::io::Write;
    let mut w = writer.clone();
    let _ = writeln!(w, "{line}");
}
