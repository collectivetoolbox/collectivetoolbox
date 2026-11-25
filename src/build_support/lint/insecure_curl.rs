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

//! Linter rule prohibiting `curl -k` and `curl --insecure` across shell
//! scripts, Dockerfiles, CI workflows, and Python subprocess invocations.
//!
//! Disabling TLS certificate verification with `-k` or `--insecure` exposes
//! downloads, health checks, and authenticated API calls to active
//! interception and is never permitted in repository scripts or tooling.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::headers;

/// A violation representing a prohibited `curl -k` or `curl --insecure` usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsecureCurlViolation {
    /// Path to the file containing the prohibited invocation.
    pub file: PathBuf,
    /// 1-based line number of the offending flag.
    pub line: usize,
    /// Human-readable explanation of the violation.
    pub message: String,
}

#[derive(Debug, Clone)]
struct ShellToken {
    line: usize,
    text: String,
}

#[derive(Debug, Clone)]
enum ShellItem {
    Token(ShellToken),
    Separator,
}

fn is_shell_word_break(ch: char) -> bool {
    ch.is_ascii_whitespace()
        || matches!(ch, ';' | '|' | '&' | '(' | ')' | '{' | '}' | '`')
}

fn is_env_assignment(word: &str) -> bool {
    let Some(eq_pos) = word.find('=') else {
        return false;
    };
    let Some(name) = word.get(..eq_pos) else {
        return false;
    };
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_curl_program(word: &str) -> bool {
    let normalized = word.trim();
    normalized == "curl"
        || normalized == "curl.exe"
        || normalized.ends_with("/curl")
        || normalized.ends_with("/curl.exe")
        || normalized.ends_with("\\curl")
        || normalized.ends_with("\\curl.exe")
}

fn curl_short_opt_takes_value(ch: char) -> bool {
    matches!(
        ch,
        'o' | 'H'
            | 'X'
            | 'd'
            | 'u'
            | 'w'
            | 'A'
            | 'e'
            | 'b'
            | 'c'
            | 'F'
            | 'T'
            | 'K'
            | 'x'
            | 'U'
            | 'E'
            | 'm'
            | 'y'
            | 'Y'
            | 'C'
            | 'D'
            | 'P'
            | 'Q'
            | 'r'
            | 'z'
    )
}

fn curl_long_opt_takes_value(opt: &str) -> bool {
    matches!(
        opt,
        "--Abstract-unix-socket"
            | "--abstract-unix-socket"
            | "--alt-svc"
            | "--aws-sigv4"
            | "--cacert"
            | "--capath"
            | "--cert"
            | "--cert-type"
            | "--ciphers"
            | "--config"
            | "--connect-timeout"
            | "--connect-to"
            | "--continue-at"
            | "--cookie"
            | "--cookie-jar"
            | "--curves"
            | "--data"
            | "--data-ascii"
            | "--data-binary"
            | "--data-raw"
            | "--data-urlencode"
            | "--delegation"
            | "--dns-interface"
            | "--dns-ipv4-addr"
            | "--dns-ipv6-addr"
            | "--dns-servers"
            | "--doh-url"
            | "--dump-header"
            | "--ech"
            | "--egd-file"
            | "--engine"
            | "--etag-compare"
            | "--etag-save"
            | "--expect100-timeout"
            | "--form"
            | "--form-string"
            | "--ftp-account"
            | "--ftp-alternative-to-user"
            | "--ftp-method"
            | "--ftp-port"
            | "--ftp-ssl-ccc-mode"
            | "--happy-eyeballs-timeout-ms"
            | "--header"
            | "--hostpubmd5"
            | "--hostpubsha256"
            | "--hsts"
            | "--interface"
            | "--ip-tos"
            | "--json"
            | "--keepalive-cnt"
            | "--keepalive-time"
            | "--key"
            | "--key-type"
            | "--krb"
            | "--libcurl"
            | "--limit-rate"
            | "--local-port"
            | "--login-options"
            | "--mail-auth"
            | "--mail-from"
            | "--mail-rcpt"
            | "--max-filesize"
            | "--max-redirs"
            | "--max-time"
            | "--NOPROXY"
            | "--noproxy"
            | "--oauth2-bearer"
            | "--output"
            | "--output-dir"
            | "--pass"
            | "--pinnedpubkey"
            | "--proto"
            | "--proto-default"
            | "--proto-redir"
            | "--proxy"
            | "--proxy-cacert"
            | "--proxy-capath"
            | "--proxy-cert"
            | "--proxy-cert-type"
            | "--proxy-ciphers"
            | "--proxy-header"
            | "--proxy-key"
            | "--proxy-key-type"
            | "--proxy-pass"
            | "--proxy-pinnedpubkey"
            | "--proxy-service-name"
            | "--proxy-tls13-ciphers"
            | "--proxy-tlsauthtype"
            | "--proxy-tlspassword"
            | "--proxy-tlsuser"
            | "--proxy-user"
            | "--proxy1.0"
            | "--pubkey"
            | "--quote"
            | "--random-file"
            | "--range"
            | "--rate"
            | "--referer"
            | "--request"
            | "--request-target"
            | "--resolve"
            | "--retry"
            | "--retry-delay"
            | "--retry-max-time"
            | "--sasl-authzid"
            | "--service-name"
            | "--socks4"
            | "--socks4a"
            | "--socks5"
            | "--socks5-gssapi-service"
            | "--socks5-hostname"
            | "--speed-limit"
            | "--speed-time"
            | "--stderr"
            | "--telnet-option"
            | "--tftp-blksize"
            | "--time-cond"
            | "--tls-max"
            | "--tls13-ciphers"
            | "--tlsauthtype"
            | "--tlspassword"
            | "--tlsuser"
            | "--trace"
            | "--trace-ascii"
            | "--trace-config"
            | "--unix-socket"
            | "--upload-file"
            | "--url"
            | "--url-query"
            | "--user"
            | "--user-agent"
            | "--variable"
            | "--vlan-priority"
            | "--write-out"
    )
}

fn skip_command_wrappers(tokens: &[ShellToken]) -> usize {
    let mut idx = 0usize;
    while let Some(tok) = tokens.get(idx) {
        let word = tok.text.as_str();
        if is_env_assignment(word) {
            idx = idx.saturating_add(1);
            continue;
        }
        match word {
            "nohup" | "time" | "exec" => {
                idx = idx.saturating_add(1);
            }
            "command" => {
                idx = idx.saturating_add(1);
                while let Some(arg) = tokens.get(idx) {
                    if arg.text == "-p" {
                        idx = idx.saturating_add(1);
                    } else if arg.text == "--" {
                        idx = idx.saturating_add(1);
                        break;
                    } else {
                        break;
                    }
                }
            }
            "env" => {
                idx = idx.saturating_add(1);
                while let Some(arg) = tokens.get(idx) {
                    let a = arg.text.as_str();
                    if a == "--" {
                        idx = idx.saturating_add(1);
                        break;
                    } else if a == "-u" || a == "--unset" || a == "-C" {
                        idx = idx.saturating_add(2);
                    } else if a.starts_with('-') || is_env_assignment(a) {
                        idx = idx.saturating_add(1);
                    } else {
                        break;
                    }
                }
            }
            "sudo" => {
                idx = idx.saturating_add(1);
                while let Some(arg) = tokens.get(idx) {
                    let a = arg.text.as_str();
                    if a == "--" {
                        idx = idx.saturating_add(1);
                        break;
                    } else if matches!(
                        a,
                        "-u" | "-g" | "-p" | "-C" | "-D" | "-R" | "-T" | "-h"
                    ) {
                        idx = idx.saturating_add(2);
                    } else if a.starts_with('-') {
                        idx = idx.saturating_add(1);
                    } else {
                        break;
                    }
                }
            }
            "timeout" => {
                idx = idx.saturating_add(1);
                while let Some(arg) = tokens.get(idx) {
                    let a = arg.text.as_str();
                    if a == "--" {
                        idx = idx.saturating_add(1);
                        break;
                    } else if a == "-k" || a == "-s" {
                        idx = idx.saturating_add(2);
                    } else if a.starts_with('-') {
                        idx = idx.saturating_add(1);
                    } else {
                        break;
                    }
                }
                // Skip timeout duration operand
                if tokens.get(idx).is_some() {
                    idx = idx.saturating_add(1);
                }
            }
            "nice" => {
                idx = idx.saturating_add(1);
                if let Some(arg) = tokens.get(idx) {
                    if arg.text == "-n" {
                        idx = idx.saturating_add(2);
                    } else if arg.text.starts_with('-') {
                        idx = idx.saturating_add(1);
                    }
                }
            }
            "xargs" => {
                idx = idx.saturating_add(1);
                while let Some(arg) = tokens.get(idx) {
                    let a = arg.text.as_str();
                    if a == "--" {
                        idx = idx.saturating_add(1);
                        break;
                    } else if matches!(
                        a,
                        "-I" | "-L" | "-n" | "-P" | "-s" | "-d" | "-E" | "-a"
                    ) {
                        idx = idx.saturating_add(2);
                    } else if a.starts_with('-') {
                        idx = idx.saturating_add(1);
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }
    idx
}

fn analyze_command_tokens(
    file_path: &Path,
    tokens: &[ShellToken],
    violations: &mut Vec<InsecureCurlViolation>,
) {
    let cmd_idx = skip_command_wrappers(tokens);
    let Some(cmd_tok) = tokens.get(cmd_idx) else {
        return;
    };
    if !is_curl_program(&cmd_tok.text) {
        return;
    }

    let mut idx = cmd_idx.saturating_add(1);
    'arg_loop: while let Some(tok) = tokens.get(idx) {
        let arg = tok.text.as_str();
        if arg == "--" {
            break;
        }
        if arg == "--insecure" || arg.starts_with("--insecure=") {
            violations.push(InsecureCurlViolation {
                file: file_path.to_path_buf(),
                line: tok.line,
                message: format!(
                    "prohibited insecure curl flag `{arg}` disables TLS certificate verification"
                ),
            });
            idx = idx.saturating_add(1);
            continue;
        }
        if arg.starts_with("--") {
            if !arg.contains('=') && curl_long_opt_takes_value(arg) {
                idx = idx.saturating_add(2);
                continue;
            }
            idx = idx.saturating_add(1);
            continue;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            if let Some(short_chars) = arg.get(1..) {
                let mut chars = short_chars.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == 'k' {
                        violations.push(InsecureCurlViolation {
                            file: file_path.to_path_buf(),
                            line: tok.line,
                            message: format!(
                                "prohibited insecure curl flag `-k` (in `{arg}`) disables TLS certificate verification"
                            ),
                        });
                        break;
                    }
                    if curl_short_opt_takes_value(ch) {
                        if chars.peek().is_none() {
                            idx = idx.saturating_add(2);
                            continue 'arg_loop;
                        }
                        break;
                    }
                }
            }
        }
        idx = idx.saturating_add(1);
    }
}

/// Scans shell command text starting at `base_line` (1-based) for `curl -k`
/// and `curl --insecure`, including inside `$(...)`, backticks, heredocs, and
/// nested quoted command strings (such as `ssh host "curl -sk ..."`).
pub fn check_shell_content_at_line(
    file_path: &Path,
    content: &str,
    base_line: usize,
    depth: usize,
    violations: &mut Vec<InsecureCurlViolation>,
) {
    if depth > 6 {
        return;
    }

    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0usize;
    let mut line = base_line;

    let mut items: Vec<ShellItem> = Vec::new();
    let mut current_token = String::new();
    let mut token_line: Option<usize> = None;
    let mut nested_snippets: Vec<(usize, String)> = Vec::new();

    let flush_token = |items: &mut Vec<ShellItem>,
                       current_token: &mut String,
                       token_line: &mut Option<usize>| {
        if let Some(tl) = token_line.take() {
            items.push(ShellItem::Token(ShellToken {
                line: tl,
                text: std::mem::take(current_token),
            }));
        }
    };

    while i < len {
        let Some(&ch) = chars.get(i) else {
            break;
        };

        // Handle backslash escape or line continuation outside quotes
        if ch == '\\' {
            if chars.get(i.saturating_add(1)) == Some(&'\n') {
                line = line.saturating_add(1);
                i = i.saturating_add(2);
                continue;
            }
            if chars.get(i.saturating_add(1)) == Some(&'\r')
                && chars.get(i.saturating_add(2)) == Some(&'\n')
            {
                line = line.saturating_add(1);
                i = i.saturating_add(3);
                continue;
            }
            if let Some(&escaped) = chars.get(i.saturating_add(1)) {
                if token_line.is_none() {
                    token_line = Some(line);
                }
                current_token.push(escaped);
                if escaped == '\n' {
                    line = line.saturating_add(1);
                }
                i = i.saturating_add(2);
                continue;
            }
        }

        // Comments start at `#` when not mid-word
        if ch == '#' && token_line.is_none() {
            while let Some(&c) = chars.get(i) {
                i = i.saturating_add(1);
                if c == '\n' {
                    line = line.saturating_add(1);
                    items.push(ShellItem::Separator);
                    break;
                }
            }
            continue;
        }

        // Newline separates commands
        if ch == '\n' {
            flush_token(&mut items, &mut current_token, &mut token_line);
            items.push(ShellItem::Separator);
            line = line.saturating_add(1);
            i = i.saturating_add(1);
            continue;
        }

        // Whitespace separates words
        if ch.is_ascii_whitespace() {
            flush_token(&mut items, &mut current_token, &mut token_line);
            i = i.saturating_add(1);
            continue;
        }

        // Command Substitution `$(` outside quotes
        if ch == '$' && chars.get(i.saturating_add(1)) == Some(&'(') {
            flush_token(&mut items, &mut current_token, &mut token_line);
            items.push(ShellItem::Separator);
            i = i.saturating_add(2);
            continue;
        }

        // Shell command separators
        if is_shell_word_break(ch) {
            flush_token(&mut items, &mut current_token, &mut token_line);
            items.push(ShellItem::Separator);
            i = i.saturating_add(1);
            continue;
        }

        // Single-quoted string
        if ch == '\'' {
            if token_line.is_none() {
                token_line = Some(line);
            }
            let quote_start_line = line;
            let mut inner = String::new();
            i = i.saturating_add(1);
            while let Some(&c) = chars.get(i) {
                i = i.saturating_add(1);
                if c == '\'' {
                    break;
                }
                if c == '\n' {
                    line = line.saturating_add(1);
                }
                current_token.push(c);
                inner.push(c);
            }
            if inner.contains("curl") {
                nested_snippets.push((quote_start_line, inner));
            }
            continue;
        }

        // Double-quoted string (with `$(...)` and backtick extraction)
        if ch == '"' {
            if token_line.is_none() {
                token_line = Some(line);
            }
            let quote_start_line = line;
            let mut inner = String::new();
            i = i.saturating_add(1);
            while let Some(&c) = chars.get(i) {
                if c == '\\' {
                    if let Some(&esc) = chars.get(i.saturating_add(1)) {
                        if esc == '\n' {
                            line = line.saturating_add(1);
                        } else {
                            current_token.push(esc);
                            inner.push(esc);
                        }
                        i = i.saturating_add(2);
                        continue;
                    }
                }
                if c == '$' && chars.get(i.saturating_add(1)) == Some(&'(') {
                    let sub_start_line = line;
                    i = i.saturating_add(2);
                    let (sub_text, new_idx, new_line) =
                        consume_command_substitution(&chars, i, line);
                    i = new_idx;
                    line = new_line;
                    if sub_text.contains("curl") {
                        nested_snippets.push((sub_start_line, sub_text));
                    }
                    current_token.push_str("__CMD_SUB__");
                    continue;
                }
                if c == '`' {
                    let sub_start_line = line;
                    i = i.saturating_add(1);
                    let mut backtick_body = String::new();
                    while let Some(&bc) = chars.get(i) {
                        i = i.saturating_add(1);
                        if bc == '\\' {
                            if let Some(&besc) = chars.get(i) {
                                if besc == '\n' {
                                    line = line.saturating_add(1);
                                }
                                backtick_body.push(besc);
                                i = i.saturating_add(1);
                                continue;
                            }
                        }
                        if bc == '`' {
                            break;
                        }
                        if bc == '\n' {
                            line = line.saturating_add(1);
                        }
                        backtick_body.push(bc);
                    }
                    if backtick_body.contains("curl") {
                        nested_snippets.push((sub_start_line, backtick_body));
                    }
                    current_token.push_str("__CMD_SUB__");
                    continue;
                }
                i = i.saturating_add(1);
                if c == '"' {
                    break;
                }
                if c == '\n' {
                    line = line.saturating_add(1);
                }
                current_token.push(c);
                inner.push(c);
            }
            if inner.contains("curl") {
                nested_snippets.push((quote_start_line, inner));
            }
            continue;
        }

        // Ordinary character
        if token_line.is_none() {
            token_line = Some(line);
        }
        current_token.push(ch);
        i = i.saturating_add(1);
    }

    flush_token(&mut items, &mut current_token, &mut token_line);

    let mut cmd_tokens: Vec<ShellToken> = Vec::new();
    for item in items {
        match item {
            ShellItem::Token(tok) => cmd_tokens.push(tok),
            ShellItem::Separator => {
                if !cmd_tokens.is_empty() {
                    analyze_command_tokens(file_path, &cmd_tokens, violations);
                    cmd_tokens.clear();
                }
            }
        }
    }
    if !cmd_tokens.is_empty() {
        analyze_command_tokens(file_path, &cmd_tokens, violations);
    }

    for (sub_line, snippet) in nested_snippets {
        check_shell_content_at_line(
            file_path,
            &snippet,
            sub_line,
            depth.saturating_add(1),
            violations,
        );
    }
}

fn consume_command_substitution(
    chars: &[char],
    start_idx: usize,
    start_line: usize,
) -> (String, usize, usize) {
    let mut out = String::new();
    let mut i = start_idx;
    let mut line = start_line;
    let mut paren_depth = 1usize;
    let mut in_single = false;
    let mut in_double = false;

    while let Some(&c) = chars.get(i) {
        if c == '\n' {
            line = line.saturating_add(1);
        }
        if in_single {
            out.push(c);
            i = i.saturating_add(1);
            if c == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            if c == '\\' {
                out.push(c);
                if let Some(&esc) = chars.get(i.saturating_add(1)) {
                    if esc == '\n' {
                        line = line.saturating_add(1);
                    }
                    out.push(esc);
                    i = i.saturating_add(2);
                    continue;
                }
            }
            if c == '$' && chars.get(i.saturating_add(1)) == Some(&'(') {
                paren_depth = paren_depth.saturating_add(1);
                out.push('$');
                out.push('(');
                i = i.saturating_add(2);
                continue;
            }
            out.push(c);
            i = i.saturating_add(1);
            if c == '"' {
                in_double = false;
            }
            continue;
        }

        if c == '\\' {
            out.push(c);
            if let Some(&esc) = chars.get(i.saturating_add(1)) {
                if esc == '\n' {
                    line = line.saturating_add(1);
                }
                out.push(esc);
                i = i.saturating_add(2);
                continue;
            }
        }
        if c == '\'' {
            in_single = true;
            out.push(c);
            i = i.saturating_add(1);
            continue;
        }
        if c == '"' {
            in_double = true;
            out.push(c);
            i = i.saturating_add(1);
            continue;
        }
        if c == '(' {
            paren_depth = paren_depth.saturating_add(1);
            out.push(c);
            i = i.saturating_add(1);
            continue;
        }
        if c == ')' {
            paren_depth = paren_depth.saturating_sub(1);
            i = i.saturating_add(1);
            if paren_depth == 0 {
                break;
            }
            out.push(c);
            continue;
        }
        out.push(c);
        i = i.saturating_add(1);
    }

    (out, i, line)
}

/// Checks shell script, Dockerfile, or CI YAML content for prohibited `curl`
/// flags.
pub fn check_shell_content(
    file_path: &Path,
    content: &str,
) -> Vec<InsecureCurlViolation> {
    let mut violations = Vec::new();
    check_shell_content_at_line(file_path, content, 1, 0, &mut violations);
    dedup_violations(&mut violations);
    violations
}

#[derive(Debug, Default)]
struct PySeqFrame {
    items: Vec<ShellToken>,
    current_has_string: bool,
    current_string: Option<ShellToken>,
    current_has_other: bool,
    current_other_line: Option<usize>,
}

impl PySeqFrame {
    fn finish_element(&mut self) {
        if let Some(str_tok) = self.current_string.take() {
            if !self.current_has_other {
                self.items.push(str_tok);
            } else {
                self.items.push(ShellToken {
                    line: str_tok.line,
                    text: "__PY_EXPR__".to_string(),
                });
            }
        } else if let Some(other_line) = self.current_other_line.take() {
            self.items.push(ShellToken {
                line: other_line,
                text: "__PY_EXPR__".to_string(),
            });
        }
        self.current_has_string = false;
        self.current_has_other = false;
        self.current_other_line = None;
    }
}

fn strip_fstring_interpolations(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while let Some(&c) = chars.get(i) {
        if c == '{' {
            if chars.get(i.saturating_add(1)) == Some(&'{') {
                out.push('{');
                i = i.saturating_add(2);
                continue;
            }
            let mut depth = 1usize;
            i = i.saturating_add(1);
            while let Some(&ic) = chars.get(i) {
                i = i.saturating_add(1);
                if ic == '{' {
                    depth = depth.saturating_add(1);
                } else if ic == '}' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                } else if ic == '\n' {
                    out.push('\n');
                }
            }
            out.push_str("__PY_FEXPR__");
            continue;
        }
        if c == '}' && chars.get(i.saturating_add(1)) == Some(&'}') {
            out.push('}');
            i = i.saturating_add(2);
            continue;
        }
        out.push(c);
        i = i.saturating_add(1);
    }
    out
}

/// Checks Python source code for prohibited `curl -k` and `curl --insecure`
/// flags in both sequence argument lists (`["curl", "-k", ...]`) and shell
/// command strings (`subprocess.run("curl --insecure ...", shell=True)`).
pub fn check_python_content(
    file_path: &Path,
    content: &str,
) -> Vec<InsecureCurlViolation> {
    let mut violations = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut frames: Vec<PySeqFrame> = Vec::new();

    while i < len {
        let Some(&ch) = chars.get(i) else {
            break;
        };

        if ch == '\n' {
            line = line.saturating_add(1);
            i = i.saturating_add(1);
            continue;
        }

        if ch.is_ascii_whitespace() {
            i = i.saturating_add(1);
            continue;
        }

        // Python comment
        if ch == '#' {
            while let Some(&c) = chars.get(i) {
                i = i.saturating_add(1);
                if c == '\n' {
                    line = line.saturating_add(1);
                    break;
                }
            }
            continue;
        }

        // Check for string prefix (r, u, b, f, rf, fr, rb, br, case-insensitive)
        let mut prefix_len = 0usize;
        let mut is_raw = false;
        let mut is_fstring = false;
        if ch.is_ascii_alphabetic() {
            let prev_is_ident = if i > 0 {
                chars
                    .get(i.saturating_sub(1))
                    .is_some_and(|p| p.is_ascii_alphanumeric() || *p == '_')
            } else {
                false
            };
            if !prev_is_ident {
                let c1 = ch.to_ascii_lowercase();
                if matches!(c1, 'r' | 'u' | 'b' | 'f') {
                    if matches!(chars.get(i.saturating_add(1)), Some('\'' | '"'))
                    {
                        prefix_len = 1;
                        is_raw = c1 == 'r';
                        is_fstring = c1 == 'f';
                    } else if let Some(&c2_raw) = chars.get(i.saturating_add(1))
                    {
                        let c2 = c2_raw.to_ascii_lowercase();
                        if matches!(
                            (c1, c2),
                            ('r', 'f' | 'b') | ('f' | 'b', 'r')
                        ) && matches!(
                            chars.get(i.saturating_add(2)),
                            Some('\'' | '"')
                        ) {
                            prefix_len = 2;
                            is_raw = true;
                            is_fstring = c1 == 'f' || c2 == 'f';
                        }
                    }
                }
            }
        }

        let quote_pos = i.saturating_add(prefix_len);
        if (prefix_len > 0 || ch == '\'' || ch == '"')
            && let Some(&quote_ch) = chars.get(quote_pos)
            && (quote_ch == '\'' || quote_ch == '"')
        {
            let str_start_line = line;
            let is_triple = chars.get(quote_pos.saturating_add(1))
                == Some(&quote_ch)
                && chars.get(quote_pos.saturating_add(2)) == Some(&quote_ch);
            i = if is_triple {
                quote_pos.saturating_add(3)
            } else {
                quote_pos.saturating_add(1)
            };

            let mut str_val = String::new();
            while let Some(&sc) = chars.get(i) {
                if sc == '\\' {
                    if let Some(&esc) = chars.get(i.saturating_add(1)) {
                        if esc == '\n' {
                            line = line.saturating_add(1);
                            if is_raw {
                                str_val.push('\\');
                                str_val.push('\n');
                            }
                            i = i.saturating_add(2);
                            continue;
                        }
                        if is_raw {
                            str_val.push('\\');
                            str_val.push(esc);
                        } else {
                            match esc {
                                'n' => str_val.push('\n'),
                                't' => str_val.push('\t'),
                                'r' => str_val.push('\r'),
                                '\\' => str_val.push('\\'),
                                '\'' => str_val.push('\''),
                                '"' => str_val.push('"'),
                                other => {
                                    str_val.push('\\');
                                    str_val.push(other);
                                }
                            }
                        }
                        i = i.saturating_add(2);
                        continue;
                    }
                }

                if is_triple {
                    if sc == quote_ch
                        && chars.get(i.saturating_add(1)) == Some(&quote_ch)
                        && chars.get(i.saturating_add(2)) == Some(&quote_ch)
                    {
                        i = i.saturating_add(3);
                        break;
                    }
                } else if sc == quote_ch {
                    i = i.saturating_add(1);
                    break;
                }

                if sc == '\n' {
                    line = line.saturating_add(1);
                }
                str_val.push(sc);
                i = i.saturating_add(1);
            }

            let normalized_str = if is_fstring {
                strip_fstring_interpolations(&str_val)
            } else {
                str_val
            };

            // Check shell-string style invocations (`subprocess.run("curl -k ...", shell=True)`)
            if normalized_str.contains("curl") && normalized_str != "curl" {
                check_shell_content_at_line(
                    file_path,
                    &normalized_str,
                    str_start_line,
                    0,
                    &mut violations,
                );
            }

            // Record into enclosing sequence frame (handling implicit string literal concatenation)
            if let Some(frame) = frames.last_mut() {
                if let Some(existing) = frame.current_string.as_mut() {
                    existing.text.push_str(&normalized_str);
                } else {
                    frame.current_has_string = true;
                    frame.current_string = Some(ShellToken {
                        line: str_start_line,
                        text: normalized_str,
                    });
                }
            }
            continue;
        }

        // Sequence open
        if ch == '[' || ch == '(' {
            if let Some(parent) = frames.last_mut() {
                parent.current_has_other = true;
                if parent.current_other_line.is_none() {
                    parent.current_other_line = Some(line);
                }
            }
            frames.push(PySeqFrame::default());
            i = i.saturating_add(1);
            continue;
        }

        // Sequence element separator
        if ch == ',' {
            if let Some(frame) = frames.last_mut() {
                frame.finish_element();
            }
            i = i.saturating_add(1);
            continue;
        }

        // Sequence close
        if ch == ']' || ch == ')' {
            if let Some(mut frame) = frames.pop() {
                frame.finish_element();
                if frame.items.iter().any(|t| is_curl_program(&t.text)) {
                    analyze_command_tokens(
                        file_path,
                        &frame.items,
                        &mut violations,
                    );
                }
            }
            i = i.saturating_add(1);
            continue;
        }

        // Other tokens inside sequence (variables, operators, calls, etc.)
        if let Some(frame) = frames.last_mut() {
            frame.current_has_other = true;
            if frame.current_other_line.is_none() {
                frame.current_other_line = Some(line);
            }
        }
        i = i.saturating_add(1);
    }

    dedup_violations(&mut violations);
    violations
}

fn dedup_violations(violations: &mut Vec<InsecureCurlViolation>) {
    violations
        .sort_by(|a, b| (&a.file, a.line, &a.message).cmp(&(&b.file, b.line, &b.message)));
    violations.dedup();
}

fn collect_extra_script_files(
    workspace_root: &Path,
    extra_shell_like: &mut Vec<PathBuf>,
    extra_python: &mut Vec<PathBuf>,
) -> Result<()> {
    // Include GitHub Actions workflows (`.github/workflows/*.yml`, `*.yaml`)
    let workflows_dir = workspace_root.join(".github").join("workflows");
    if workflows_dir.is_dir() {
        for entry in fs::read_dir(&workflows_dir).with_context(|| {
            format!("failed to read {}", workflows_dir.display())
        })? {
            let path = entry?.path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("yml" | "yaml")
                )
            {
                extra_shell_like.push(path);
            }
        }
    }

    // Include helper scripts placed directly under `ci-logs/` (such as `ci-logs/vm`)
    // without traversing `ci-logs/logs_*` or VM disk directories.
    let ci_logs_dir = workspace_root.join("ci-logs");
    if ci_logs_dir.is_dir() {
        for entry in fs::read_dir(&ci_logs_dir).with_context(|| {
            format!("failed to read {}", ci_logs_dir.display())
        })? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("py") {
                extra_python.push(path);
            } else if ext == Some("sh") || ext == Some("bash") {
                extra_shell_like.push(path);
            } else if ext.is_none()
                && let Ok(content) = fs::read_to_string(&path)
                && content
                    .lines()
                    .next()
                    // Reason for fallback: empty file yields empty first line which is not a shebang
                    .unwrap_or("")
                    .starts_with("#!")
            {
                extra_shell_like.push(path);
            }
        }
    }

    Ok(())
}

/// Checks already-discovered shell, Dockerfile, and Python file lists plus
/// workflow YAML and `ci-logs/` helper scripts.
pub fn check_discovered_files(
    workspace_root: &Path,
    shell_files: &[PathBuf],
    docker_files: &[PathBuf],
    python_files: &[PathBuf],
) -> Result<(usize, Vec<InsecureCurlViolation>)> {
    let mut shell_set: BTreeSet<PathBuf> = shell_files
        .iter()
        .chain(docker_files.iter())
        .cloned()
        .collect();
    let mut py_set: BTreeSet<PathBuf> = python_files.iter().cloned().collect();

    let mut extra_shell = Vec::new();
    let mut extra_py = Vec::new();
    collect_extra_script_files(workspace_root, &mut extra_shell, &mut extra_py)?;
    shell_set.extend(extra_shell);
    py_set.extend(extra_py);

    let mut violations = Vec::new();
    let total_files = shell_set.len().saturating_add(py_set.len());

    for path in &shell_set {
        let content = fs::read_to_string(path).with_context(|| {
            format!("failed to read script {}", path.display())
        })?;
        violations.extend(check_shell_content(path, &content));
    }

    for path in &py_set {
        let content = fs::read_to_string(path).with_context(|| {
            format!("failed to read Python file {}", path.display())
        })?;
        violations.extend(check_python_content(path, &content));
    }

    Ok((total_files, violations))
}

/// Discovers all scripts and Python files in `workspace_root` and checks for
/// prohibited `curl -k` or `curl --insecure` flags.
pub fn check_workspace(
    workspace_root: &Path,
) -> Result<(usize, Vec<InsecureCurlViolation>)> {
    let mut rs_files = Vec::new();
    let mut scm_files = Vec::new();
    let mut docker_files = Vec::new();
    let mut shell_files = Vec::new();
    let mut python_files = Vec::new();
    let mut wat_files = Vec::new();
    let mut stagel_files = Vec::new();
    let mut unused_header_violations = Vec::new();

    headers::find_files(
        workspace_root,
        &mut rs_files,
        &mut scm_files,
        &mut docker_files,
        &mut shell_files,
        &mut python_files,
        &mut wat_files,
        &mut stagel_files,
        &mut unused_header_violations,
    )?;

    check_discovered_files(
        workspace_root,
        &shell_files,
        &docker_files,
        &python_files,
    )
}

/// Runs the `insecure-curl` linter from the CLI and prints results.
pub fn run(workspace_root: &Path) -> Result<()> {
    println!("=== Checking for insecure curl flags (-k / --insecure) ===");
    let (checked_count, violations) = check_workspace(workspace_root)?;
    if violations.is_empty() {
        println!(
            "insecure-curl lint passed ({checked_count} script/Python files checked)."
        );
        Ok(())
    } else {
        eprintln!(
            "insecure-curl lint failed: found {} violation(s):",
            violations.len()
        );
        for v in &violations {
            // Reason for fallback: paths outside workspace_root display full path
            let relative = v.file.strip_prefix(workspace_root).unwrap_or(&v.file);
            eprintln!("  {}:{}: {}", relative.display(), v.line, v.message);
        }
        bail!(
            "`curl -k` and `curl --insecure` are prohibited; validate TLS certificates properly instead."
        );
    }
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

    #[test]
    fn test_shell_detects_curl_k_and_insecure_variants() {
        let script = r#"#!/usr/bin/env bash
# A comment mentioning curl -k and curl --insecure is ignored
curl -s -k "https://example.com/status"
resp="$(curl -s -k "$status_url" 2>/dev/null || true)"
sudo -u deployer env FOO=1 /usr/bin/curl -fsSLk \
    https://example.com/payload
remote_ssh "$server_user" "curl --insecure https://127.0.0.1:8443/api"
"#;
        let violations = check_shell_content(Path::new("test.sh"), script);
        let lines: Vec<usize> = violations.iter().map(|v| v.line).collect();
        assert_eq!(lines, vec![3, 4, 5, 7], "violations: {violations:?}");
    }

    #[test]
    fn test_shell_allows_safe_curl_and_unrelated_k_flags() {
        let script = r#"#!/usr/bin/env bash
curl --proto '=https' --tlsv1.2 -sSfL -o output_k.tar.gz "https://collectivetoolbox.com/releases"
curl -fsS --resolve "collectivetoolbox.com:443:127.0.0.1" "https://collectivetoolbox.com/api/update-status"
wineserver -k || true
freebsd-version -k -u
gzip -k archive.tar
timeout -k 5s 30s curl -fsS "https://example.com"
sudo -k curl -fsS "https://example.com"
curl -fsS -o -k -- "https://example.com/-k"
"#;
        let violations = check_shell_content(Path::new("safe.sh"), script);
        assert!(
            violations.is_empty(),
            "unexpected violations: {violations:?}"
        );
    }

    #[test]
    fn test_python_detects_subprocess_list_and_shell_strings() {
        let py_src = r#"
import subprocess

# Comment with curl -k should be ignored
subprocess.run(["curl", "-s", "-k", url], check=True)
subprocess.Popen(("/usr/bin/curl", "-o", dest_path, "--insecure", url))
subprocess.check_output(["env", "A=1", "curl", "-fsSLk", url])
subprocess.run(f"curl -sk {endpoint}", shell=True)
"#;
        let violations = check_python_content(Path::new("script.py"), py_src);
        let lines: Vec<usize> = violations.iter().map(|v| v.line).collect();
        assert_eq!(lines, vec![5, 6, 7, 8], "violations: {violations:?}");
    }

    #[test]
    fn test_python_allows_safe_curl_and_other_commands() {
        let py_src = r#"
import subprocess

subprocess.run(["curl", "-fsSL", "-o", "pkg_k.tar.gz", url], check=True)
subprocess.run(["gzip", "-k", "pkg.tar"], check=True)
msg = "Error: curl request failed"
"#;
        let violations = check_python_content(Path::new("safe.py"), py_src);
        assert!(
            violations.is_empty(),
            "unexpected violations: {violations:?}"
        );
    }

    #[test]
    fn test_live_repository_has_no_insecure_curl() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let (checked_count, violations) = check_workspace(root).unwrap();
        assert!(checked_count > 0, "expected repository scripts to be scanned");
        assert!(
            violations.is_empty(),
            "repository contains prohibited insecure curl usages: {violations:?}"
        );
    }
}
