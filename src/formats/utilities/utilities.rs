pub mod detection;
pub mod extension;
pub mod extension_data;
pub mod format_id;
pub mod magic;
pub mod magic_data;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

// Trait to extend char with a as_utf8_bytes() convenience method
pub trait CharUtfBytesExt {
    fn as_utf8_bytes(&self) -> Vec<u8>;
}

impl CharUtfBytesExt for char {
    /// Similar to `encode_utf8` - more convenient, but slower and copies.
    fn as_utf8_bytes(&self) -> Vec<u8> {
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        s.as_bytes().to_vec()
    }
}

/// Output wrapper for format conversion operations, returning the converted result
/// alongside any accumulated conversion log warnings/errors.
#[derive(Debug)]
pub struct ConversionOutput<T> {
    pub result: T,
    pub log: FormatLog,
}

impl<T> ConversionOutput<T> {
    pub fn new(result: T, log: FormatLog) -> Self {
        Self { result, log }
    }
}

#[derive(Default, Debug)]
pub struct FormatLog {
    errors: Vec<String>,
    warnings: Vec<String>,
    debug_messages: Vec<String>,
    /// Stores the order and type of all log entries, so that formatting can preserve log order and type.
    log_order: Vec<(LogType, usize)>,
}

/// Tracks the type of log entry.
#[derive(Copy, Clone, Debug)]
enum LogType {
    Error,
    Warning,
    Debug,
}

impl FormatLog {
    /// Record a serious error that may indicate the document could not be fully processed.
    pub fn error(&mut self, message: &str) {
        #[cfg(debug_assertions)]
        crate::debug!("FormatLog error: {}", message);
        self.errors.push(message.to_string());
        self.log_order
            .push((LogType::Error, self.errors.len().saturating_sub(1)));
    }

    pub fn warn(&mut self, message: &str) {
        #[cfg(debug_assertions)]
        crate::debug!("FormatLog warn: {}", message);
        self.warnings.push(message.to_string());
        self.log_order
            .push((LogType::Warning, self.warnings.len().saturating_sub(1)));
    }

    #[cfg(not(debug_assertions))]
    pub fn debug(&mut self, message: &str) {}

    #[cfg(debug_assertions)]
    pub fn debug(&mut self, message: &str) {
        #[cfg(debug_assertions)]
        crate::debug!("FormatLog debug: {}", message);
        self.debug_messages.push(message.to_string());
        self.log_order.push((
            LogType::Debug,
            self.debug_messages.len().saturating_sub(1),
        ));
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn get_warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_no_errors(&self) -> bool {
        !self.has_errors()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn has_debug_messages(&self) -> bool {
        !self.debug_messages.is_empty()
    }

    pub fn has_any(&self) -> bool {
        self.has_errors() || self.has_warnings() || self.has_debug_messages()
    }

    pub fn has_warnings_or_errors(&self) -> bool {
        self.has_warnings() || self.has_errors()
    }

    pub fn has_no_warnings_or_errors(&self) -> bool {
        !self.has_warnings_or_errors()
    }

    /// Add an import error at a specific character index and problem description.
    pub fn import_error(&mut self, index: u64, problem: &str) {
        let error = format!(
            "An unrecoverable problem was encountered while importing at character {index}: {problem}"
        );
        self.error(error.as_str());
    }

    /// Add an import warning for a specific character index and problem description.
    pub fn import_warning(&mut self, index: u64, problem: &str) {
        let warn = format!(
            "A problem was encountered while importing at character {index}: {problem}"
        );
        self.warn(warn.as_str());
    }

    /// Add an export error at a specific character index and problem description.
    pub fn export_error(&mut self, index: u64, problem: &str) {
        let error = format!(
            "An unrecoverable problem was encountered while exporting at character {index}: {problem}"
        );
        self.error(error.as_str());
    }

    /// Add an export warning for a specific character index and problem description.
    pub fn export_warning(&mut self, index: u64, problem: &str) {
        let warn = format!(
            "A problem was encountered while exporting at character {index}: {problem}"
        );
        self.warn(warn.as_str());
    }

    pub fn export_warning_unmappable(
        &mut self,
        index: u64,
        problem_dc: u32,
        format: &str,
    ) {
        self.export_warning(index, format!("The character {problem_dc} could not be represented in the chosen export format ({format}).").as_str());
    }

    pub fn merge(&mut self, other: &FormatLog) {
        let error_offset = self.errors.len();
        let warning_offset = self.warnings.len();
        let debug_offset = self.debug_messages.len();

        self.errors.extend(other.errors.clone());
        self.warnings.extend(other.warnings.clone());
        self.debug_messages.extend(other.debug_messages.clone());

        for &(typ, idx) in &other.log_order {
            let adjusted_idx = match typ {
                LogType::Error => idx.saturating_add(error_offset),
                LogType::Warning => idx.saturating_add(warning_offset),
                LogType::Debug => idx.saturating_add(debug_offset),
            };
            self.log_order.push((typ, adjusted_idx));
        }
    }

    /// Formats all log messages in the order they were logged, with proper prefixing.
    pub fn format_all(&self) -> String {
        if !self.has_any() {
            return String::new();
        }
        let mut output = String::new();
        output.push_str("Messages during format processing:\n");
        // idx are not consecutive if printed, this uses them to encode the sort
        // order, I think. They're consecutive w/i each message type.
        for &(typ, idx) in &self.log_order {
            match typ {
                LogType::Error => {
                    output.push_str("* [ERROR] ");
                    if let Some(msg) = self.errors.get(idx) {
                        output.push_str(msg);
                    }
                }
                LogType::Warning => {
                    output.push_str("- [WARNING] ");
                    if let Some(msg) = self.warnings.get(idx) {
                        output.push_str(msg);
                    }
                }
                LogType::Debug => {
                    output.push_str("- [DEBUG] ");
                    if let Some(msg) = self.debug_messages.get(idx) {
                        output.push_str(msg);
                    }
                }
            }
            output.push('\n');
        }
        output
    }

    pub fn format_errors(&self) -> String {
        let mut errors = String::new();
        if self.has_errors() {
            for e in &self.errors {
                errors.push_str("- ");
                errors.push_str(e);
                errors.push('\n');
            }
            format!("Errors during format processing:\n{errors}")
        } else {
            String::new()
        }
    }

    pub fn format_warnings(&self) -> String {
        let mut warnings = String::new();
        if self.has_warnings() {
            for w in &self.warnings {
                warnings.push_str("- ");
                warnings.push_str(w);
                warnings.push('\n');
            }
            format!("Warnings during format processing:\n{warnings}")
        } else {
            String::new()
        }
    }

    pub fn format_debug(&self) -> String {
        let mut debug = String::new();
        if self.debug_messages.is_empty() {
            String::new()
        } else {
            for d in &self.debug_messages {
                debug.push_str("- ");
                debug.push_str(d);
                debug.push('\n');
            }
            format!("Debug messages during format processing:\n{debug}")
        }
    }

    pub fn auto_log(&self) {
        if self.has_any() {
            debug!(self.format_errors());
            debug!(self.format_warnings());
            debug!(self.format_debug());
        } else {
            debug!("No errors or warnings during format processing.");
        }
    }
}

impl std::fmt::Display for FormatLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.has_any() {
            writeln!(f, "{}", self.format_all())?;
        } else {
            writeln!(f, "No errors or warnings during format processing.")?;
        }
        Ok(())
    }
}

// Test helpers
pub fn assert_vec_u8_ok_eq_no_warnings(
    expected: &[u8],
    actual: Result<(Vec<u8>, FormatLog)>,
) -> Vec<u8> {
    let (actual_bytes, log) = actual.unwrap();
    assert!(
        log.has_no_warnings_or_errors(),
        "Warnings or errors found:\n{}",
        log.format_all()
    );
    assert_vec_u8_eq_log(expected, &actual_bytes, &log);
    actual_bytes
}

pub fn assert_vec_u8_ok_eq_no_errors(
    expected: &[u8],
    actual: Result<(Vec<u8>, FormatLog)>,
) -> (Vec<u8>, FormatLog) {
    let (actual_bytes, log) = actual.unwrap();

    assert!(log.has_no_errors(), "Errors found:\n{}", log.format_all());
    assert_vec_u8_eq_log(expected, &actual_bytes, &log);
    (actual_bytes, log)
}

fn _assert_vec_u32_ok_eq_log(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
    disallow_warnings: bool,
) -> (Vec<u32>, FormatLog) {
    let (actual_vec, log) = actual.unwrap();

    let mut log_problem_type = "Errors";
    if disallow_warnings {
        log_problem_type = "Warnings or errors";
    }
    let message = format!("{log_problem_type} found:\n{}", log.format_all());

    if disallow_warnings {
        assert!(log.has_no_warnings_or_errors(), "{message}");
    } else {
        assert!(log.has_no_errors(), "{message}");
    }

    assert_vec_u32_eq_log(expected, &actual_vec, &log);

    (actual_vec, log)
}

pub fn assert_vec_u32_ok_eq_no_warnings(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
) -> (Vec<u32>, FormatLog) {
    _assert_vec_u32_ok_eq_log(expected, actual, true)
}

pub fn assert_vec_u32_ok_eq_no_errors(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
) -> (Vec<u32>, FormatLog) {
    _assert_vec_u32_ok_eq_log(expected, actual, false)
}

/// Equivalent to `assert_vec_u32_eq`, but prints the provided log on failure
pub fn assert_vec_u32_eq_log(
    expected: &[u32],
    actual: &[u32],
    log: &FormatLog,
) {
    _assert_vec_u32_eq_log(expected, actual, log);
}

fn _assert_vec_u32_eq_log(expected: &[u32], actual: &[u32], log: &FormatLog) {
    let message = format!(
        "Vectors (u32) differ.\n{}\nLog:      {}",
        fmt_mismatch_vec_u32(expected, actual),
        log.format_all()
    );

    assert_eq!(expected, actual, "{message}");
}

/// Equivalent to `assert_vec_u8_eq`, but prints the provided log on failure
pub fn assert_vec_u8_eq_log(expected: &[u8], actual: &[u8], log: &FormatLog) {
    assert_eq!(
        expected,
        actual,
        "Vectors (u8) differ.\n{}\nLog:      {}",
        fmt_mismatch_vec_u8(expected, actual),
        log.format_all()
    );
}

pub fn assert_string_eq(expected: &str, actual: String) -> String {
    let actual_string = actual;

    assert_eq!(
        expected,
        &actual_string,
        "Strings differ.\n{}",
        fmt_mismatch_string(expected, &actual_string),
    );
    actual_string
}

pub fn assert_string_ok_eq(expected: &str, actual: Result<String>) -> String {
    let actual_string = actual.unwrap();

    assert_eq!(
        expected,
        &actual_string,
        "Strings differ.\n{}",
        fmt_mismatch_string(expected, &actual_string),
    );
    actual_string
}

pub fn assert_string_ok_eq_no_warnings(
    expected: &str,
    actual: Result<(String, FormatLog)>,
) -> (String, FormatLog) {
    let (actual_string, log) = actual.unwrap();

    assert!(
        log.has_no_warnings_or_errors(),
        "Warnings or errors found:\n{}",
        log.format_all()
    );
    assert_eq!(
        expected,
        &actual_string,
        "Strings differ.\n{}\nLog:      {}",
        fmt_mismatch_string(expected, &actual_string),
        log.format_all()
    );
    (actual_string, log)
}

pub fn assert_string_ok_eq_no_errors(
    expected: &str,
    actual: Result<(String, FormatLog)>,
) -> (String, FormatLog) {
    let (actual_string, log) = actual.unwrap();

    assert!(log.has_no_errors(), "Errors found:\n{}", log.format_all());
    assert_eq!(
        expected,
        &actual_string,
        "Strings differ.\n{}\nLog:      {}",
        fmt_mismatch_string(expected, &actual_string),
        log.format_all()
    );
    (actual_string, log)
}
