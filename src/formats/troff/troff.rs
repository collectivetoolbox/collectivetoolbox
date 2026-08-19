// SPDX-License-Identifier: AGPL-3.0-or-later AND Apache-2.0
// SPDX-License-Identifier for parts derived from mantohtml: Apache-2.0
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

// Parts derived from mantohtml:
// Copyright © 2022-2023 by Michael R Sweet.
//
// Licensed under Apache License v2.0.
// <https://opensource.org/licenses/Apache-2.0>

// See additional licensing details at end of file.

//! Man page to HTML conversion program.
//! Based on <https://github.com/michaelrsweet/mantohtml>
//! Not 100% equivalent to the above; there are a couple intentional
//! differences where there appeared to be bugs (.in 17 gives 17em not 1em, and
//! doubled `</a>\n</a>` only has one `</a>`); and it uses its own name/version.
//!
//! Usage:
//!
//!    mantohtml \[OPTIONS\] MAN-FILE \[... MAN-FILE\] >HTML-FILE
//!
//! Options:
//!
//!    --author 'AUTHOR'        Set author metadata
//!    --chapter 'CHAPTER'      Set chapter (H1 heading)
//!    --copyright 'COPYRIGHT'  Set copyright metadata
//!    --css CSS-FILE-OR-URL    Use named stylesheet
//!    --help                   Show help
//!    --subject 'SUBJECT'      Set subject metadata
//!    --title 'TITLE'          Set output title
//!    --version                Show version
//!
//! Converted to Rust with AI assistance
//! Limitations: Supports a subset of macros, not all of roff and its packages.
//!
//! Other implementations:
//! <https://github.com/plan9foundation/plan9/blob/main/sys/src/cmd/troff2html/troff2html.c>
//! <https://github.com/plan9foundation/plan9/tree/main/sys/src/cmd/troff>

#[expect(
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use num_enum::IntoPrimitive;

use include_dir::{Dir, include_dir};

use ctb_formats_utilities::FormatLog;

static TROFF_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_troff_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&TROFF_DATA_DIR, key)
}

// -----------------------------------------------------------------------------
// Constants / Types
// -----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Font {
    Regular,
    Bold,
    Italic,
    Small,
    SmallBold,
    Monospace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive)]
#[repr(u8)]
enum Heading {
    Topic,
    Section,
    Subsection,
}

struct State {
    wrote_header: bool,
    in_block: Option<&'static str>,
    in_link: bool,
    indent: usize,
    author: Option<String>,
    chapter: Option<String>,
    copyright: Option<String>,
    css: Option<String>,
    subject: Option<String>,
    title: Option<String>,

    atopic: String,
    asection: String,
    font: Font,
}

impl State {
    fn new() -> Self {
        State {
            wrote_header: false,
            in_block: None,
            in_link: false,
            indent: 0,
            author: None,
            chapter: None,
            copyright: None,
            css: None,
            subject: None,
            title: None,
            atopic: String::new(),
            asection: String::new(),
            font: Font::Regular,
        }
    }
}

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.is_empty() {
        args.push("mantohtml".into());
    }

    let mut state = State::new();

    let mut i = 1usize;
    let mut end_of_options = false;

    if args.len() == 1 {
        usage(None);
        std::process::exit(1);
    }

    while i < args.len() {
        let Some(arg) = args.get(i) else {
            break;
        };
        if !end_of_options && arg == "--author" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!("mantohtml: Missing author after --author.");
                std::process::exit(1);
            }
            state.author = args.get(i).cloned();
        } else if !end_of_options && arg == "--chapter" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!("mantohtml: Missing chapter after --chapter.");
                std::process::exit(1);
            }
            state.chapter = args.get(i).cloned();
        } else if !end_of_options && arg == "--copyright" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!("mantohtml: Missing copyright after --copyright.");
                std::process::exit(1);
            }
            state.copyright = args.get(i).cloned();
        } else if !end_of_options && arg == "--css" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!(
                    "mantohtml: Missing CSS filename or URL after --css."
                );
                std::process::exit(1);
            }
            state.css = args.get(i).cloned();
        } else if !end_of_options && arg == "--help" {
            usage(None);
            return;
        } else if !end_of_options && arg == "--subject" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!("mantohtml: Missing subject after --subject.");
                std::process::exit(1);
            }
            state.subject = args.get(i).cloned();
        } else if !end_of_options && arg == "--title" {
            i = i.saturating_add(1);
            if i >= args.len() {
                eprintln!("mantohtml: Missing title after --title.");
                std::process::exit(1);
            }
            state.title = args.get(i).cloned();
        } else if !end_of_options && arg == "--version" {
            println!("{}", package_version());
            return;
        } else if !end_of_options && arg == "--" {
            end_of_options = true;
        } else if !end_of_options && arg.starts_with('-') {
            usage(Some(arg));
            std::process::exit(1);
        } else {
            // Treat as filename
            if let Err(err) = convert_man_to_stdout(&mut state, arg) {
                eprintln!("{err}");
            }
        }
        i = i.saturating_add(1);
    }

    if !state.wrote_header {
        usage(None);
        std::process::exit(1);
    }
}

// -----------------------------------------------------------------------------
// Conversion
// -----------------------------------------------------------------------------

fn convert_man_to_stdout(
    state: &mut State,
    filename: &str,
) -> Result<(), String> {
    let data = fs::read(filename).map_err(|e| format!("{filename}: {e}"))?;

    let basepath;
    if let Some(parent) = Path::new(filename).parent() {
        basepath = parent.to_string_lossy().to_string();
    } else {
        basepath = ".".into();
    }

    convert_man_from_data(
        state,
        data,
        Some(filename),
        Some(basepath.as_str()),
        io::stdout(),
    )
}

pub fn convert_man_troff_to_html(
    data: Vec<u8>,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut state = State::new();
    let mut buffer: Vec<u8> = Vec::new();
    let mut log: FormatLog = FormatLog::default();
    let result =
        convert_man_from_data(&mut state, data, None, None, &mut buffer);

    if let Err(err) = result {
        log.error(&err);
        return Ok((buffer, log));
    }

    Ok((buffer, log))
}

#[expect(
    clippy::too_many_lines,
    clippy::arithmetic_side_effects,
    reason = "Legacy macro conversion dispatcher"
)]
fn convert_man_from_data<W: Write>(
    state: &mut State,
    data: Vec<u8>,
    filename: Option<&str>,
    basepath: Option<&str>,
    out: W,
) -> Result<(), String> {
    let mut reader = ManReader::new(&data);

    let mut th_seen = false;
    let mut warning = false;
    let mut break_text = String::new();

    let mut stdout = io::BufWriter::new(out);

    // Reason for fallback: document title defaults to "document" if filename parameter is None.
    let filename = filename.unwrap_or("document");

    while let Some(line) = reader.man_gets() {
        // Handle macro lines starting with '.'
        if line.starts_with('.') {
            let mut lp: &str = &line;
            // Reason for fallback: macro_name defaults to empty string if line contains no token.
            let macro_name = parse_value(&mut lp).unwrap_or_default();

            if macro_name == "." {
                continue;
            } else if macro_name == ".TH" {
                // .TH title section ...
                // Reason for fallback: title_val defaults to empty string if missing in .TH.
                let title_val = parse_value(&mut lp).unwrap_or_default();
                if title_val.is_empty() {
                    return Err(format!(
                        "mantohtml: Missing title in '.TH' on line {} of '{}'.",
                        reader.line_number(),
                        filename
                    ));
                }
                // Reason for fallback: section defaults to empty string if missing in .TH.
                let section = parse_value(&mut lp).unwrap_or_default();
                if section.is_empty()
                    // Reason for fallback: defaults to space character if section string is empty.
                    || !section.chars().next().unwrap_or(' ').is_ascii_digit()
                {
                    return Err(format!(
                        "mantohtml: Missing section in '.TH' on line {} of '{}'.",
                        reader.line_number(),
                        filename
                    ));
                }
                let topic = format!("{title_val}({section})");

                if state.wrote_header {
                    if state.in_link {
                        writeln!(stdout, "</a>").ok();
                        state.in_link = false;
                    }
                    if let Some(block) = state.in_block.take() {
                        writeln!(stdout, "</{block}>").ok();
                    }
                } else {
                    html_header(state, &topic, &mut stdout);
                }

                html_heading(state, Heading::Topic, &topic, &mut stdout);
                th_seen = true;
            } else if !th_seen {
                if !warning {
                    eprintln!(
                        "mantohtml: Need '.TH' before '{}' macro on line {} of '{}'.",
                        macro_name,
                        reader.line_number(),
                        filename
                    );
                    warning = true;
                }
                continue;
            } else if macro_name == ".B" {
                let mut remainder = lp.to_owned();
                if remainder.trim().is_empty()
                    && let Some(next_line) = reader.man_gets()
                {
                    remainder = next_line;
                }
                push_font(state, Font::Bold, &mut stdout);
                man_puts(state, &remainder, &mut stdout);
                pop_font(state, &mut stdout);
                writeln!(stdout, "{break_text}").ok();
                break_text.clear();
            } else if macro_name == ".BI" {
                handle_xx(
                    state,
                    Font::Bold,
                    Font::Italic,
                    &mut reader,
                    lp,
                    &mut break_text,
                    &mut stdout,
                    basepath,
                );
            } else if macro_name == ".BR" {
                handle_xx(
                    state,
                    Font::Bold,
                    Font::Regular,
                    &mut reader,
                    lp,
                    &mut break_text,
                    &mut stdout,
                    basepath,
                );
            } else if macro_name == ".EE" || macro_name == ".fi" {
                if state.in_block == Some("pre") {
                    writeln!(stdout, "</pre>").ok();
                    state.in_block = None;
                } else {
                    eprintln!(
                        "mantohtml: '{}' with no '.EX' or '.nf' on line {} of '{}'.",
                        macro_name,
                        reader.line_number(),
                        filename
                    );
                }
            } else if macro_name == ".EX" || macro_name == ".nf" {
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                write!(stdout, "    <pre>").ok();
                state.in_block = Some("pre");
            } else if macro_name == ".HP" {
                let mut lp2 = lp;
                // Reason for fallback: HP macro indent defaults to 2.5em per groff man specification.
                let indent = parse_measurement(&mut lp2, 'n')
                    .unwrap_or_else(|| "2.5em".into());
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                write!(
                    stdout,
                    "    <p style=\"margin-left: {indent}; text-indent: -{indent};\">"
                )
                .ok();
                state.in_block = Some("p");
            } else if macro_name == ".I" {
                let mut remainder = lp.to_owned();
                if remainder.trim().is_empty()
                    && let Some(next_line) = reader.man_gets()
                {
                    remainder = next_line;
                }
                push_font(state, Font::Italic, &mut stdout);
                man_puts(state, &remainder, &mut stdout);
                pop_font(state, &mut stdout);
                writeln!(stdout, "{break_text}").ok();
                break_text.clear();
            } else if macro_name == ".IB" {
                handle_xx(
                    state,
                    Font::Italic,
                    Font::Bold,
                    &mut reader,
                    lp,
                    &mut break_text,
                    &mut stdout,
                    basepath,
                );
            } else if macro_name == ".IP" {
                let mut lp2 = lp;
                // Reason for fallback: IP macro tag defaults to empty string if not supplied.
                let tag = parse_value(&mut lp2).unwrap_or_default();
                // Reason for fallback: IP macro indent defaults to 2.5em per groff man specification.
                let indent = parse_measurement(&mut lp2, 'n')
                    .unwrap_or_else(|| "2.5em".into());
                close_link_if(state, &mut stdout);
                if let Some(block) = state.in_block
                    && block != "ul"
                {
                    writeln!(stdout, "</{block}>").ok();
                    state.in_block = None;
                }
                if state.in_block.is_none() {
                    writeln!(stdout, "    <ul>").ok();
                    state.in_block = Some("ul");
                }
                let list_style = if tag == "\\(bu" || tag == "-" || tag == "*" {
                    ""
                } else {
                    "list-style-type: none; "
                };
                write!(
                    stdout,
                    "    <li style=\"{list_style}margin-left: {indent};\">"
                )
                .ok();
            } else if macro_name == ".IR" {
                handle_xx(
                    state,
                    Font::Italic,
                    Font::Regular,
                    &mut reader,
                    lp,
                    &mut break_text,
                    &mut stdout,
                    basepath,
                );
            } else if macro_name == ".LP"
                || macro_name == ".P"
                || macro_name == ".PP"
            {
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                write!(stdout, "    <p>").ok();
                state.in_block = Some("p");
            } else if macro_name == ".ME" || macro_name == ".UE" {
                if state.in_link {
                    writeln!(stdout, "</a>").ok();
                    state.in_link = false;
                }
            } else if macro_name == ".MT" {
                let mut lp2 = lp;
                // Reason for fallback: MT macro email defaults to empty string if not supplied.
                let email = parse_value(&mut lp2).unwrap_or_default();
                if !email.is_empty() {
                    write!(
                        stdout,
                        "<a href=\"mailto:{}\">",
                        html_escape(&email)
                    )
                    .ok();
                    state.in_link = true;
                }
            } else if macro_name == ".RB" {
                handle_xx(
                    state,
                    Font::Regular,
                    Font::Bold,
                    &mut reader,
                    lp,
                    &mut break_text,
                    &mut stdout,
                    basepath,
                );
            } else if macro_name == ".RE" {
                if state.indent > 0 {
                    writeln!(stdout, "    </div>").ok();
                    state.indent -= 1;
                } else {
                    eprintln!(
                        "mantohtml: Unbalanced '.RE' on line {} of '{}'.",
                        reader.line_number(),
                        filename
                    );
                }
            } else if macro_name == ".RS" {
                let mut lp2 = lp;
                // Reason for fallback: RS macro indent defaults to 0.5in per groff man specification.
                let indent = parse_measurement(&mut lp2, 'n')
                    .unwrap_or_else(|| "0.5in".into());
                writeln!(stdout, "    <div style=\"margin-left: {indent};\">")
                    .ok();
                state.indent += 1;
            } else if macro_name == ".SB" {
                let mut remainder = lp.to_owned();
                if remainder.trim().is_empty()
                    && let Some(next_line) = reader.man_gets()
                {
                    remainder = next_line;
                }
                push_font(state, Font::SmallBold, &mut stdout);
                man_puts(state, &remainder, &mut stdout);
                pop_font(state, &mut stdout);
                writeln!(stdout, "{break_text}").ok();
                break_text.clear();
            } else if macro_name == ".SH" {
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                html_heading(state, Heading::Section, lp, &mut stdout);
            } else if macro_name == ".SM" {
                let mut remainder = lp.to_owned();
                if remainder.trim().is_empty()
                    && let Some(next_line) = reader.man_gets()
                {
                    remainder = next_line;
                }
                push_font(state, Font::Small, &mut stdout);
                man_puts(state, &remainder, &mut stdout);
                pop_font(state, &mut stdout);
                writeln!(stdout, "{break_text}").ok();
                break_text.clear();
            } else if macro_name == ".SS" {
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                html_heading(state, Heading::Subsection, lp, &mut stdout);
            } else if macro_name == ".SY" {
                if let Some(block) = state.in_block.take() {
                    writeln!(stdout, "</{block}>").ok();
                }
                write!(stdout, "    <p style=\"font-family: monospace;\">")
                    .ok();
                state.in_block = Some("p");
            } else if macro_name == ".TP" {
                let mut lp2 = lp;
                // Reason for fallback: TP macro indent defaults to 2.5em per groff man specification.
                let indent = parse_measurement(&mut lp2, 'n')
                    .unwrap_or_else(|| "2.5em".into());
                close_link_if(state, &mut stdout);
                close_block_if(state, &mut stdout);
                write!(
                    stdout,
                    "    <p style=\"margin-left: {indent}; text-indent: -{indent};\">"
                )
                .ok();
                state.in_block = Some("p");
                break_text = "<br>".into();
            } else if macro_name == ".UR" {
                let mut lp2 = lp;
                // Reason for fallback: UR macro url defaults to empty string if not supplied.
                let url = parse_value(&mut lp2).unwrap_or_default();
                if !url.is_empty() {
                    write!(stdout, "<a href=\"{}\">", html_escape(&url)).ok();
                    state.in_link = true;
                }
            } else if macro_name == ".YS" {
                if state.in_block == Some("p") {
                    writeln!(stdout, "</p>").ok();
                    state.in_block = None;
                } else {
                    eprintln!(
                        "mantohtml: '.YS' seen without prior '.SY' on line {} of '{}'.",
                        reader.line_number(),
                        filename
                    );
                }
            } else if macro_name == ".br" {
                writeln!(stdout, "<br>").ok();
            } else if macro_name == ".in" {
                let mut lp2 = lp;
                if let Some(indent) = parse_measurement(&mut lp2, 'm') {
                    writeln!(
                        stdout,
                        "    <div style=\"margin-left: {};\">",
                        html_escape(&indent)
                    )
                    .ok();
                    state.indent += 1;
                } else if state.indent > 0 {
                    writeln!(stdout, "    </div>").ok();
                    state.indent -= 1;
                } else {
                    eprintln!(
                        "mantohtml: '.in' seen without prior '.in INDENT' on line {} of '{}'.",
                        reader.line_number(),
                        filename
                    );
                }
            } else if macro_name == ".sp" {
                writeln!(stdout, "<br>&nbsp;<br>").ok();
            } else {
                eprintln!(
                    "mantohtml: Unsupported command/macro '{}' on line {} of '{}'.",
                    macro_name,
                    reader.line_number(),
                    filename
                );
            }
        } else if th_seen {
            // Text
            if state.in_block.is_none() {
                write!(stdout, "<p>").ok();
                state.in_block = Some("p");
            }
            man_puts(state, &line, &mut stdout);
            writeln!(stdout, "{break_text}").ok();
            break_text.clear();
        } else if !line.is_empty() && !warning {
            eprintln!(
                "mantohtml: Ignoring text before '.TH' on line {} of '{}'.",
                reader.line_number(),
                filename
            );
            warning = true;
        }
    }

    if state.wrote_header {
        html_footer(state, &mut stdout);
    }

    stdout.flush().ok();
    Ok(())
}

// -----------------------------------------------------------------------------
// ManReader replicating C man_gets
// -----------------------------------------------------------------------------

struct ManReader<'a> {
    data: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> ManReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            line: 0,
        }
    }

    fn line_number(&self) -> usize {
        self.line
    }

    fn getc(&mut self) -> Option<u8> {
        let c = self.data.get(self.pos).copied()?;
        self.pos = self.pos.saturating_add(1);
        Some(c)
    }

    fn ungetc(&mut self) {
        if self.pos > 0 {
            self.pos = self.pos.saturating_sub(1);
        }
    }

    fn man_gets(&mut self) -> Option<String> {
        let mut out = Vec::new();
        while let Some(c) = self.getc() {
            if c == b'\n' {
                self.line = self.line.saturating_add(1);
                break;
            } else if c == b'\\' {
                if let Some(nc) = self.getc() {
                    if nc == b'\n' {
                        // Continuation
                        self.line = self.line.saturating_add(1);
                        continue;
                    } else if nc == b'"' {
                        // Comment until end of line
                        while let Some(cc) = self.getc() {
                            if cc == b'\n' {
                                self.line = self.line.saturating_add(1);
                                break;
                            }
                        }
                        break;
                    }
                    out.push(b'\\');
                    out.push(nc);
                } else {
                    break;
                }
            } else {
                out.push(c);
            }
        }

        if out.is_empty() && self.pos >= self.data.len() {
            None
        } else {
            Some(String::from_utf8_lossy(&out).to_string())
        }
    }
}

// -----------------------------------------------------------------------------
// Helpers: HTML output
// -----------------------------------------------------------------------------

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            // The original doesn't escape a couple things that I usually would
            // '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

fn html_anchor(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while let Some(&b) = bytes.get(i) {
        let c = char::from(b);
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
            out.push(c.to_ascii_lowercase());
        } else if (c == '(' || c == ' ' || c == '\t')
            && i.saturating_add(1) < bytes.len()
            && !out.ends_with('-')
        {
            out.push('-');
        }
        i = i.saturating_add(1);
        if out.len() >= 255 {
            break;
        }
    }
    out
}

fn html_header(state: &mut State, topic: &str, w: &mut impl Write) {
    if state.wrote_header {
        return;
    }
    state.wrote_header = true;
    let _ = writeln!(w, "<!DOCTYPE html>");
    let _ = writeln!(w, "<html>");
    let _ = writeln!(w, "  <head>");

    if let Some(css) = &state.css {
        if css.starts_with("http://") || css.starts_with("https://") {
            let _ = writeln!(
                w,
                "    <link rel=\"stylesheet\" type=\"text/css\" href=\"{}\">",
                html_escape(css)
            );
        } else {
            let _ = writeln!(w, "    <style><!--");
            if let Ok(text) = fs::read_to_string(css) {
                let _ = write!(w, "{text}");
            }
            let _ = writeln!(w, "--></style>");
        }
    }
    if let Some(author) = &state.author {
        let _ = writeln!(
            w,
            "    <meta name=\"author\" content=\"{}\">",
            html_escape(author)
        );
    }
    if let Some(c) = &state.copyright {
        let _ = writeln!(
            w,
            "    <meta name=\"copyright\" content=\"{}\">",
            html_escape(c)
        );
    }
    let _ = writeln!(
        w,
        "    <meta name=\"creator\" content=\"{} v{} convert_man_troff_to_html\">",
        package_short_name(),
        package_version()
    );
    if let Some(subject) = &state.subject {
        let _ = writeln!(
            w,
            "    <meta name=\"subject\" content=\"{}\">",
            html_escape(subject)
        );
    }
    // Reason for fallback: header title defaults to topic parameter or generic "Documentation" if title is unassigned.
    let title = state.title.as_deref().unwrap_or({
        if topic.is_empty() {
            "Documentation"
        } else {
            topic
        }
    });
    let _ = writeln!(w, "    <title>{}</title>", html_escape(title));
    let _ = writeln!(w, "  </head>");
    let _ = writeln!(w, "  <body>");

    if let Some(chapter) = &state.chapter {
        let anchor = html_anchor(chapter);
        let _ = writeln!(
            w,
            "    <h1 id=\"{}\">{}</h1>",
            anchor,
            html_escape(chapter)
        );
    }
}

fn html_footer(state: &mut State, w: &mut impl Write) {
    if state.wrote_header {
        let _ = writeln!(w, "  </body>");
        let _ = writeln!(w, "</html>");
        state.wrote_header = false;
    }
}

fn push_font(state: &mut State, font: Font, w: &mut impl Write) {
    if state.font == font && state.in_block.is_some() {
        return;
    }
    // Close previous
    if state.font != Font::Regular {
        match state.font {
            Font::Bold => {
                let _ = write!(w, "</strong>");
            }
            Font::Italic => {
                let _ = write!(w, "</em>");
            }
            Font::Small => {
                let _ = write!(w, "</small>");
            }
            Font::SmallBold => {
                let _ = write!(w, "</small>");
            }
            Font::Monospace => {
                let _ = write!(w, "</pre>");
            }
            Font::Regular => (),
        }
    }
    if state.in_block.is_none() {
        let _ = write!(w, "<p>");
        state.in_block = Some("p");
    }
    match font {
        Font::Regular => (),
        Font::Bold => {
            let _ = write!(w, "<strong>");
        }
        Font::Italic => {
            let _ = write!(w, "<em>");
        }
        Font::Small => {
            let _ = write!(w, "<small>");
        }
        Font::SmallBold => {
            let _ = write!(w, "<small style=\"font-weight: bold;\">");
        }
        Font::Monospace => {
            let _ = write!(w, "<pre>");
        }
    }
    state.font = font;
}

fn pop_font(state: &mut State, w: &mut impl Write) {
    push_font(state, Font::Regular, w);
}

fn html_heading(
    state: &mut State,
    heading: Heading,
    raw: &str,
    w: &mut impl Write,
) {
    // Regenerate capitalized heading for section/subsection
    let mut title = raw.trim().to_string();
    if heading != Heading::Topic {
        title = capitalize_heading_words(&title);
    }

    if state.in_link {
        let _ = writeln!(w, "</a>");
        state.in_link = false;
    }
    if let Some(block) = state.in_block.take() {
        let _ = writeln!(w, "</{block}>");
    }

    let heading_u8: u8 = heading.into();
    let hlevel = if state.chapter.is_some() {
        heading_u8.saturating_add(2)
    } else {
        heading_u8.saturating_add(1)
    };

    match heading {
        Heading::Topic => {
            state.atopic = html_anchor(raw);
            let _ = write!(
                w,
                "    <h{} id=\"{}\">",
                hlevel,
                html_escape(&state.atopic)
            );
        }
        Heading::Section => {
            state.asection = html_anchor(raw);
            let _ = write!(
                w,
                "    <h{} id=\"{}.{}\">",
                hlevel,
                html_escape(&state.atopic),
                html_escape(&state.asection)
            );
        }
        Heading::Subsection => {
            let subsection = html_anchor(raw);
            let _ = write!(
                w,
                "    <h{} id=\"{}.{}.{}\">",
                hlevel,
                html_escape(&state.atopic),
                html_escape(&state.asection),
                html_escape(&subsection)
            );
        }
    }

    man_puts(state, &title, w);
    let _ = writeln!(w, "</h{hlevel}>");
}

fn capitalize_heading_words(s: &str) -> String {
    let mut title = s.as_bytes().to_vec();
    // Capitalize first letter of each word except "a", "and", "or", "the"
    let mut i = 0usize;
    while i < title.len() {
        let Some(&ch) = title.get(i) else {
            break;
        };
        if (char::from(ch)).is_ascii_alphabetic() {
            // Check if this is the start of a word (first letter or preceded by non-alpha)
            let is_start = i == 0;
            // Check for exception words: "a ", "and ", "or ", "the "
            let is_exception = title.get(i..i.saturating_add(2)) == Some(b"a ")
                || title.get(i..i.saturating_add(4)) == Some(b"and ")
                || title.get(i..i.saturating_add(3)) == Some(b"or ")
                || title.get(i..i.saturating_add(4)) == Some(b"the ");

            if is_start || !is_exception {
                // Capitalize first letter
                if let Some(elem) = title.get_mut(i) {
                    *elem = ch.to_ascii_uppercase();
                }
            }

            // Lowercase the rest of the word
            let mut j = i.saturating_add(1);
            while j < title.len()
                && title
                    .get(j)
                    .copied()
                    .map(char::from)
                    .is_some_and(|c| c.is_ascii_alphabetic())
            {
                if let Some(elem) = title.get_mut(j) {
                    *elem = elem.to_ascii_lowercase();
                }
                j = j.saturating_add(1);
            }
            i = j;
        } else {
            i = i.saturating_add(1);
        }
    }
    // Reason for fallback: invalid UTF-8 title bytes default to empty string.
    String::from_utf8(title).unwrap_or_default()
}

// -----------------------------------------------------------------------------
// Macro handling similar to man_xx
// -----------------------------------------------------------------------------

#[expect(clippy::too_many_arguments, reason = "Legacy macro conversion helper")]
fn handle_xx(
    state: &mut State,
    a: Font,
    b: Font,
    reader: &mut ManReader,
    lp: &str,
    break_text: &mut String,
    w: &mut impl Write,
    basepath: Option<&str>,
) {
    let mut line_rest = lp.to_owned();
    if line_rest.trim().is_empty()
        && let Some(next_line_r) = reader.man_gets()
    {
        line_rest = next_line_r; // move the owned String
    }
    let mut line_rest = line_rest.as_str();
    let mut words = Vec::new();
    {
        while let Some(val) = parse_value(&mut line_rest) {
            words.push(val);
        }
    }

    let have_basepath = basepath.is_some();
    let basepath = basepath.unwrap_or("");
    let original_font = state.font;
    let mut use_a = true;
    let mut idx = 0usize;
    while idx < words.len() {
        let Some(word) = words.get(idx) else {
            break;
        };
        // Attempt link detection for ".BR name (section)"
        let mut have_link = false;
        if a == Font::Bold
            && b == Font::Regular
            && use_a
            && idx.saturating_add(1) < words.len()
            && words
                .get(idx.saturating_add(1))
                .is_some_and(|w| w.starts_with('(') && w.contains(')'))
        {
            if let Some(sec) = words.get(idx.saturating_add(1)) {
                if let Some(endp) = sec.find(')') {
                    let section = sec.get(1..endp).unwrap_or("");
                    if section
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_digit())
                        && have_basepath
                    {
                        let manfile = format!("{basepath}/{word}.{section}");
                        if Path::new(&manfile).exists() {
                            let _ = write!(
                                w,
                                "<a href=\"{}.html\">",
                                html_escape(word)
                            );
                            have_link = true;
                        }
                    }
                }
            }
        }

        push_font(state, if use_a { a } else { b }, w);
        man_puts(state, word, w);

        if have_link {
            if idx.saturating_add(1) < words.len() {
                idx = idx.saturating_add(1);
                push_font(state, b, w);
                if let Some(w_next) = words.get(idx) {
                    man_puts(state, w_next, w);
                }
            }
            let _ = write!(w, "</a>");
        } else {
            use_a = !use_a;
        }
        idx = idx.saturating_add(1);
    }

    push_font(state, original_font, w);
    let _ = writeln!(w);
    if !break_text.is_empty() {
        let _ = writeln!(w, "{break_text}");
        break_text.clear();
    }
    let _ = writeln!(w);
}

// -----------------------------------------------------------------------------
// man_puts + escape/sequence handling (subset of original logic)
// -----------------------------------------------------------------------------

fn flush_fragment(start: usize, end: usize, w: &mut impl Write, bytes: &[u8]) {
    if end > start {
        if let Some(slice) = bytes.get(start..end) {
            let text = String::from_utf8_lossy(slice);
            let _ = write!(w, "{}", html_escape(&text));
        }
    }
}

fn man_puts(state: &mut State, s: &str, w: &mut impl Write) {
    let mut i = 0usize;
    let bytes = s.as_bytes();
    let mut fragment_start = 0usize;

    while let Some(&b) = bytes.get(i) {
        if b == b'\\' {
            if bytes.get(i.saturating_add(1)).is_some() {
                flush_fragment(fragment_start, i, w, bytes);
                i = i.saturating_add(1);

                #[allow(
                    clippy::expect_used,
                    reason = "i < bytes.len() guaranteed by preceding is_some check"
                )]
                let c = char::from(*bytes.get(i).expect("i < bytes.len() guaranteed by is_some check"));
                match c {
                    'f' => {
                        if let Some(&b2) = bytes.get(i.saturating_add(1)) {
                            i = i.saturating_add(1);
                            let fch = char::from(b2);
                            match fch {
                                'R' | 'P' => push_font(state, Font::Regular, w),
                                'b' | 'B' => push_font(state, Font::Bold, w),
                                'i' | 'I' => push_font(state, Font::Italic, w),
                                _ => {
                                    eprintln!(
                                        "mantohtml: Unknown font '\\f{fch}' ignored."
                                    );
                                }
                            }
                            i = i.saturating_add(1);
                        } else {
                            i = i.saturating_add(1);
                        }
                    }
                    '*' => {
                        i = i.saturating_add(1);
                        if let Some(&bch) = bytes.get(i) {
                            let ch = char::from(bch);
                            if ch == '(' {
                                i = i.saturating_add(1);
                                if let (Some(&bm1), Some(&bm2)) = (
                                    bytes.get(i),
                                    bytes.get(i.saturating_add(1)),
                                ) {
                                    let m1 = char::from(bm1);
                                    let m2 = char::from(bm2);
                                    let macro_id = format!("{m1}{m2}");
                                    match macro_id.as_str() {
                                        "aq" => {
                                            let _ = write!(w, "'");
                                        }
                                        "dq" => {
                                            let _ = write!(w, "&quot;");
                                        }
                                        "lq" => {
                                            let _ = write!(w, "&ldquo;");
                                        }
                                        "rq" => {
                                            let _ = write!(w, "&rdquo;");
                                        }
                                        "Tm" => {
                                            let _ = write!(w, "<sup>TM</sup>");
                                        }
                                        _ => {
                                            eprintln!(
                                                "mantohtml: Unknown macro '\\*({m1}{m2})' ignored."
                                            );
                                        }
                                    }
                                    i = i.saturating_add(2);
                                }
                            } else {
                                match ch {
                                    'R' => {
                                        let _ = write!(w, "&reg;");
                                    }
                                    _ => {
                                        eprintln!(
                                            "mantohtml: Unknown macro '\\*{ch}' ignored."
                                        );
                                    }
                                }
                                i = i.saturating_add(1);
                            }
                        }
                    }
                    '(' => {
                        // \(... style
                        if let Some(seq) = bytes.get(i..i.saturating_add(3)) {
                            let token = String::from_utf8_lossy(seq);
                            match token.as_ref() {
                                "(bu" => {
                                    let _ = write!(w, "&middot;");
                                }
                                "(em" => {
                                    let _ = write!(w, "&mdash;");
                                }
                                "(en" => {
                                    let _ = write!(w, "&ndash;");
                                }
                                "(ga" => {
                                    let _ = write!(w, "`");
                                }
                                "(ha" => {
                                    let _ = write!(w, "^");
                                }
                                "(ti" => {
                                    let _ = write!(w, "~");
                                }
                                _ => {
                                    let _ =
                                        write!(w, "{}", html_escape(&token));
                                }
                            }
                            i = i.saturating_add(3);
                        } else {
                            // Just treat literally
                            let _ = write!(w, "(");
                            i = i.saturating_add(1);
                        }
                    }
                    '[' => {
                        // \[xx] style
                        let mut j = i;
                        while let Some(&bj) = bytes.get(j) {
                            if bj == b']' {
                                break;
                            }
                            j = j.saturating_add(1);
                        }

                        if matches!(bytes.get(j), Some(b']')) {
                            #[allow(
                                clippy::expect_used,
                                reason = "i + 1 <= j < bytes.len() established by bracket scan"
                            )]
                            let token_bytes = bytes
                                .get(i.saturating_add(1)..j)
                                .expect("i + 1 <= j < bytes.len() in bracket scan");
                            match token_bytes {
                                b"aq" => {
                                    let _ = write!(w, "'");
                                }
                                b"co" => {
                                    let _ = write!(w, "&copy;");
                                }
                                b"cq" => {
                                    let _ = write!(w, "&rsquo;");
                                }
                                b"de" => {
                                    let _ = write!(w, "&deg;");
                                }
                                b"dq" => {
                                    let _ = write!(w, "&quot;");
                                }
                                b"lq" => {
                                    let _ = write!(w, "&ldquo;");
                                }
                                b"mc" => {
                                    let _ = write!(w, "&mu;");
                                }
                                b"oq" => {
                                    let _ = write!(w, "&lsquo;");
                                }
                                b"rg" => {
                                    let _ = write!(w, "&reg;");
                                }
                                b"rq" => {
                                    let _ = write!(w, "&rdquo;");
                                }
                                b"tm" => {
                                    let _ = write!(w, "<sup>TM</sup>");
                                }
                                // Unknown, output literally
                                _ => {
                                    let token =
                                        String::from_utf8_lossy(token_bytes);
                                    let _ = write!(
                                        w,
                                        "\\[{}]",
                                        html_escape(&token)
                                    );
                                }
                            }
                            i = j.saturating_add(1);
                        } else {
                            // Unterminated, output literally
                            let _ = write!(w, "\\[");
                            i = i.saturating_add(1);
                        }
                    }
                    'e' => {
                        // Escaped backslash
                        let _ = write!(w, "\\");
                        i = i.saturating_add(1);
                    }
                    // Numeric escape like \123 (octal-ish in original)
                    d if d.is_ascii_digit() => {
                        let mut num = String::new();
                        num.push(d);
                        i = i.saturating_add(1);

                        if let Some(&b2) = bytes.get(i) {
                            let c2 = char::from(b2);
                            if c2.is_ascii_digit() {
                                num.push(c2);
                                i = i.saturating_add(1);
                            }
                        }
                        if let Some(&b3) = bytes.get(i) {
                            let c3 = char::from(b3);
                            if c3.is_ascii_digit() {
                                num.push(c3);
                                i = i.saturating_add(1);
                            }
                        }

                        if let Ok(v) = i32::from_str_radix(&num, 8) {
                            let _ = write!(w, "&#{v};");
                        }
                    }
                    other => {
                        // Pass through certain escapes; warn on unknown
                        match other {
                            '\\' | '"' | '\'' | '-' | ' ' => {
                                let _ = write!(w, "{other}");
                            }
                            _ => {
                                eprintln!(
                                    "mantohtml: Unrecognized escape '\\{other}' ignored."
                                );
                                let _ = write!(w, "\\{other}");
                            }
                        }
                        i = i.saturating_add(1);
                    }
                }

                fragment_start = i;
            } else {
                i = i.saturating_add(1);
            }
        } else if ({
            #[allow(
                clippy::expect_used,
                reason = "i < bytes.len() guaranteed by loop condition"
            )]
            let rem = bytes.get(i..).expect("i < bytes.len() in loop");
            starts_with_url(rem)
        }) {
            flush_fragment(fragment_start, i, w, bytes);
            #[allow(
                clippy::expect_used,
                reason = "i < bytes.len() guaranteed by loop condition"
            )]
            let rem = bytes.get(i..).expect("i < bytes.len() in loop");
            let (url, consumed) = extract_url(rem);
            let _ = write!(
                w,
                "<a href=\"{}\">{}</a>",
                html_escape(&url),
                html_escape(&url)
            );
            i = i.saturating_add(consumed);
            fragment_start = i;
        } else if matches!(b, b'<' | b'&' | b'"') {
            flush_fragment(fragment_start, i, w, bytes);
            let ch = char::from(b);
            let _ = write!(w, "{}", html_escape(&ch.to_string()));
            i = i.saturating_add(1);
            fragment_start = i;
        } else {
            i = i.saturating_add(1);
        }
    }

    flush_fragment(fragment_start, i, w, bytes);
}

fn starts_with_url(slice: &[u8]) -> bool {
    // Reason for fallback: invalid UTF-8 slice defaults to empty string for URL detection.
    let s = std::str::from_utf8(slice).unwrap_or("");
    s.starts_with("http://") || s.starts_with("https://")
}

/// Get URL from a chunk of bytes. If it is not valid UTF-8, returns empty string.
fn extract_url(slice: &[u8]) -> (String, usize) {
    // Reason for fallback: invalid UTF-8 slice defaults to empty string for URL extraction.
    let s = std::str::from_utf8(slice).unwrap_or("");
    let mut end = 0usize;
    let chars: Vec<char> = s.chars().collect();
    while end < chars.len() {
        let Some(&c) = chars.get(end) else {
            break;
        };
        if c.is_whitespace() {
            break;
        }
        if ",.)".contains(c)
            && (end.saturating_add(1) == chars.len()
                || chars
                    .get(end.saturating_add(1))
                    .copied()
                    .is_none_or(|c| c.is_whitespace() || c == '\0'))
        {
            break;
        }
        end = end.saturating_add(1);
    }
    #[allow(
        clippy::expect_used,
        reason = "end <= chars.len() guaranteed by while end < chars.len() loop"
    )]
    let url: String = chars
        .get(..end)
        .expect("end <= chars.len() in extract_url")
        .iter()
        .collect();
    let len = url.len();
    (url, len)
}

// -----------------------------------------------------------------------------
// Parsing helpers (parse_value / parse_measurement)
// -----------------------------------------------------------------------------

#[expect(
    clippy::expect_used,
    clippy::unwrap_in_result,
    reason = "String slices are guaranteed to be valid char boundaries by preceding starts_with or chars().next() checks"
)]
fn parse_value(line: &mut &str) -> Option<String> {
    let mut l = line.trim_start();
    if l.is_empty() {
        return None;
    }
    let mut out = String::new();
    if l.starts_with('"') {
        l = l.get(1..).expect("starts with quote");
        while !l.is_empty() {
            let c = l.chars().next()?;
            if c == '"' {
                l = l.get(1..).expect("c == quote");
                break;
            } else if c == '\\' {
                l = l.get(1..).expect("c == backslash");
                if let Some(nextc) = l.chars().next() {
                    out.push(nextc);
                    l = l.get(nextc.len_utf8()..).expect("valid char boundary");
                }
            } else {
                out.push(c);
                l = l.get(c.len_utf8()..).expect("valid char boundary");
            }
        }
    } else {
        while !l.is_empty() {
            let c = l.chars().next()?;
            if c.is_whitespace() {
                break;
            }
            out.push(c);
            l = l.get(c.len_utf8()..).expect("valid char boundary");
            if c == '\\' && !l.is_empty() {
                // Keep escaped char
                let c2 = l.chars().next()?;
                out.push(c2);
                l = l.get(c2.len_utf8()..).expect("valid char boundary");
            }
        }
    }
    *line = l.trim_start();
    Some(out)
}

fn parse_measurement(line: &mut &str, defunit: char) -> Option<String> {
    let val = parse_value(line)?;
    if val.is_empty() {
        return None;
    }
    let s = val.clone();
    let unit = s
        .chars()
        .next_back()
        .filter(char::is_ascii_alphabetic)
        // Reason for fallback: unit char defaults to specified defunit (e.g. 'n') if last char is not alphabetic.
        .unwrap_or(defunit);

    let number_part =
        if s.chars().last().is_some_and(|c| c.is_ascii_alphabetic()) {
            let mut chars = s.chars();
            chars.next_back();
            chars.as_str().to_string()
        } else {
            s.clone()
        };

    // Reason for fallback: number parsing defaults to 0.0 if number_part is non-numeric.
    let parsed = number_part.parse::<f64>().unwrap_or(0.0);

    let converted = match unit {
        'c' => format!("{number_part}cm"),
        'f' => format!("{:.1}%", 100.0 * parsed / 65536.0),
        'i' => format!("{number_part}in"),
        'm' => format!("{number_part}em"),
        'M' => format!("{:.2}em", parsed * 0.01),
        'n' => format!("{}em", parsed * 0.5),
        'P' => format!("{number_part}pc"),
        'p' => format!("{number_part}pt"),
        's' => format!("{:.1}%", 100.0 * parsed),
        'u' => format!("{number_part}px"),
        'v' => number_part,
        _ => return None,
    };

    Some(converted)
}

// -----------------------------------------------------------------------------
// Utility
// -----------------------------------------------------------------------------

fn close_block_if(state: &mut State, w: &mut impl Write) {
    if let Some(block) = state.in_block.take() {
        let _ = writeln!(w, "</{block}>");
    }
}

fn close_link_if(state: &mut State, w: &mut impl Write) {
    if state.in_link {
        let _ = writeln!(w, "</a>");
        state.in_link = false;
    }
}

fn usage(bad: Option<&str>) {
    if let Some(opt) = bad {
        eprintln!("Unknown option: {opt}");
    }
    println!("Usage: mantohtml [OPTIONS] MAN-FILE [... MAN-FILE] >HTML-FILE");
    println!("Options:");
    println!("   --author 'AUTHOR'        Set author metadata");
    println!("   --chapter 'CHAPTER'      Set chapter (H1 heading)");
    println!("   --copyright 'COPYRIGHT'  Set copyright metadata");
    println!("   --css CSS-FILE-OR-URL    Use named stylesheet");
    println!("   --help                   Show help");
    println!("   --subject 'SUBJECT'      Set subject metadata");
    println!("   --title 'TITLE'          Set output title");
    println!("   --version                Show version");
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
    use ctb_formats_utilities::assert_vec_u8_ok_eq_no_warnings;

    use super::*;

    #[crate::ctb_test]
    fn test_html_escape() {
        assert_eq!(html_escape("Hello & World"), "Hello &amp; World");
        // > is not escaped by this function
        assert_eq!(html_escape("<tag>"), "&lt;tag>");
        assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
        assert_eq!(html_escape("No special chars"), "No special chars");
    }

    #[crate::ctb_test]
    fn test_html_anchor() {
        assert_eq!(html_anchor("Section 1.2"), "section-1.2");
        assert_eq!(html_anchor("My-Function()"), "my-function-");
        assert_eq!(html_anchor("A B C"), "a-b-c");
        assert_eq!(html_anchor("!@#$%^&*()"), "-");
    }

    #[crate::ctb_test]
    fn test_capitalize_heading_words() {
        assert_eq!(
            capitalize_heading_words("the quick brown fox"),
            "The Quick Brown Fox"
        );
        assert_eq!(
            capitalize_heading_words("a tale of two cities"),
            "A Tale Of Two Cities"
        );
        assert_eq!(
            capitalize_heading_words("a tale or two cities"),
            "A Tale or Two Cities"
        );
        assert_eq!(
            capitalize_heading_words("and then there were none"),
            "And Then There Were None"
        );
    }

    #[crate::ctb_test]
    fn test_starts_with_url() {
        assert!(starts_with_url(b"http://example.com"));
        assert!(starts_with_url(b"https://example.com"));
        assert!(!starts_with_url(b"ftp://example.com"));
        assert!(!starts_with_url(b"Just some text"));
    }

    #[crate::ctb_test]
    fn test_extract_url() {
        let (url, len) =
            extract_url(b"http://example.com/path?query=1 more text");
        assert_eq!(url, "http://example.com/path?query=1");
        assert_eq!(len, url.len());

        let (url2, len2) = extract_url(b"https://example.com.");
        assert_eq!(url2, "https://example.com");
        assert_eq!(len2, url2.len());

        let (url3, len3) = extract_url(b"https://example.com)");
        assert_eq!(url3, "https://example.com");
        assert_eq!(len3, url3.len());
    }

    #[crate::ctb_test]
    fn test_convert_from_fixture() {
        let fixture_expected =
            get_troff_data("fixtures/out.html").expect("Could not get fixture");
        // replace bytes `ctoolbox v0.1.0 convert_man_troff_to_html` with the actual version
        let fixture_expected = string::bytes_str_replace(
            &fixture_expected,
            b"ctoolbox v0.1.0 convert_man_troff_to_html",
            format!(
                "ctoolbox v{} convert_man_troff_to_html",
                package_version()
            )
            .as_bytes(),
        );
        assert_vec_u8_ok_eq_no_warnings(
            &fixture_expected,
            convert_man_troff_to_html(
                get_troff_data("fixtures/in.1").expect("Could not get fixture"),
            ),
        );
    }
}

/*


                                Apache License
                          Version 2.0, January 2004
                       http://www.apache.org/licenses/

  TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

  1. Definitions.

     "License" shall mean the terms and conditions for use, reproduction,
     and distribution as defined by Sections 1 through 9 of this document.

     "Licensor" shall mean the copyright owner or entity authorized by
     the copyright owner that is granting the License.

     "Legal Entity" shall mean the union of the acting entity and all
     other entities that control, are controlled by, or are under common
     control with that entity. For the purposes of this definition,
     "control" means (i) the power, direct or indirect, to cause the
     direction or management of such entity, whether by contract or
     otherwise, or (ii) ownership of fifty percent (50%) or more of the
     outstanding shares, or (iii) beneficial ownership of such entity.

     "You" (or "Your") shall mean an individual or Legal Entity
     exercising permissions granted by this License.

     "Source" form shall mean the preferred form for making modifications,
     including but not limited to software source code, documentation
     source, and configuration files.

     "Object" form shall mean any form resulting from mechanical
     transformation or translation of a Source form, including but
     not limited to compiled object code, generated documentation,
     and conversions to other media types.

     "Work" shall mean the work of authorship, whether in Source or
     Object form, made available under the License, as indicated by a
     copyright notice that is included in or attached to the work
     (an example is provided in the Appendix below).

     "Derivative Works" shall mean any work, whether in Source or Object
     form, that is based on (or derived from) the Work and for which the
     editorial revisions, annotations, elaborations, or other modifications
     represent, as a whole, an original work of authorship. For the purposes
     of this License, Derivative Works shall not include works that remain
     separable from, or merely link (or bind by name) to the interfaces of,
     the Work and Derivative Works thereof.

     "Contribution" shall mean any work of authorship, including
     the original version of the Work and any modifications or additions
     to that Work or Derivative Works thereof, that is intentionally
     submitted to Licensor for inclusion in the Work by the copyright owner
     or by an individual or Legal Entity authorized to submit on behalf of
     the copyright owner. For the purposes of this definition, "submitted"
     means any form of electronic, verbal, or written communication sent
     to the Licensor or its representatives, including but not limited to
     communication on electronic mailing lists, source code control systems,
     and issue tracking systems that are managed by, or on behalf of, the
     Licensor for the purpose of discussing and improving the Work, but
     excluding communication that is conspicuously marked or otherwise
     designated in writing by the copyright owner as "Not a Contribution."

     "Contributor" shall mean Licensor and any individual or Legal Entity
     on behalf of whom a Contribution has been received by Licensor and
     subsequently incorporated within the Work.

  2. Grant of Copyright License. Subject to the terms and conditions of
     this License, each Contributor hereby grants to You a perpetual,
     worldwide, non-exclusive, no-charge, royalty-free, irrevocable
     copyright license to reproduce, prepare Derivative Works of,
     publicly display, publicly perform, sublicense, and distribute the
     Work and such Derivative Works in Source or Object form.

  3. Grant of Patent License. Subject to the terms and conditions of
     this License, each Contributor hereby grants to You a perpetual,
     worldwide, non-exclusive, no-charge, royalty-free, irrevocable
     (except as stated in this section) patent license to make, have made,
     use, offer to sell, sell, import, and otherwise transfer the Work,
     where such license applies only to those patent claims licensable
     by such Contributor that are necessarily infringed by their
     Contribution(s) alone or by combination of their Contribution(s)
     with the Work to which such Contribution(s) was submitted. If You
     institute patent litigation against any entity (including a
     cross-claim or counterclaim in a lawsuit) alleging that the Work
     or a Contribution incorporated within the Work constitutes direct
     or contributory patent infringement, then any patent licenses
     granted to You under this License for that Work shall terminate
     as of the date such litigation is filed.

  4. Redistribution. You may reproduce and distribute copies of the
     Work or Derivative Works thereof in any medium, with or without
     modifications, and in Source or Object form, provided that You
     meet the following conditions:

     (a) You must give any other recipients of the Work or
         Derivative Works a copy of this License; and

     (b) You must cause any modified files to carry prominent notices
         stating that You changed the files; and

     (c) You must retain, in the Source form of any Derivative Works
         that You distribute, all copyright, patent, trademark, and
         attribution notices from the Source form of the Work,
         excluding those notices that do not pertain to any part of
         the Derivative Works; and

     (d) If the Work includes a "NOTICE" text file as part of its
         distribution, then any Derivative Works that You distribute must
         include a readable copy of the attribution notices contained
         within such NOTICE file, excluding those notices that do not
         pertain to any part of the Derivative Works, in at least one
         of the following places: within a NOTICE text file distributed
         as part of the Derivative Works; within the Source form or
         documentation, if provided along with the Derivative Works; or,
         within a display generated by the Derivative Works, if and
         wherever such third-party notices normally appear. The contents
         of the NOTICE file are for informational purposes only and
         do not modify the License. You may add Your own attribution
         notices within Derivative Works that You distribute, alongside
         or as an addendum to the NOTICE text from the Work, provided
         that such additional attribution notices cannot be construed
         as modifying the License.

     You may add Your own copyright statement to Your modifications and
     may provide additional or different license terms and conditions
     for use, reproduction, or distribution of Your modifications, or
     for any such Derivative Works as a whole, provided Your use,
     reproduction, and distribution of the Work otherwise complies with
     the conditions stated in this License.

  5. Submission of Contributions. Unless You explicitly state otherwise,
     any Contribution intentionally submitted for inclusion in the Work
     by You to the Licensor shall be under the terms and conditions of
     this License, without any additional terms or conditions.
     Notwithstanding the above, nothing herein shall supersede or modify
     the terms of any separate license agreement you may have executed
     with Licensor regarding such Contributions.

  6. Trademarks. This License does not grant permission to use the trade
     names, trademarks, service marks, or product names of the Licensor,
     except as required for reasonable and customary use in describing the
     origin of the Work and reproducing the content of the NOTICE file.

  7. Disclaimer of Warranty. Unless required by applicable law or
     agreed to in writing, Licensor provides the Work (and each
     Contributor provides its Contributions) on an "AS IS" BASIS,
     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
     implied, including, without limitation, any warranties or conditions
     of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
     PARTICULAR PURPOSE. You are solely responsible for determining the
     appropriateness of using or redistributing the Work and assume any
     risks associated with Your exercise of permissions under this License.

  8. Limitation of Liability. In no event and under no legal theory,
     whether in tort (including negligence), contract, or otherwise,
     unless required by applicable law (such as deliberate and grossly
     negligent acts) or agreed to in writing, shall any Contributor be
     liable to You for damages, including any direct, indirect, special,
     incidental, or consequential damages of any character arising as a
     result of this License or out of the use or inability to use the
     Work (including but not limited to damages for loss of goodwill,
     work stoppage, computer failure or malfunction, or any and all
     other commercial damages or losses), even if such Contributor
     has been advised of the possibility of such damages.

  9. Accepting Warranty or Additional Liability. While redistributing
     the Work or Derivative Works thereof, You may choose to offer,
     and charge a fee for, acceptance of support, warranty, indemnity,
     or other liability obligations and/or rights consistent with this
     License. However, in accepting such obligations, You may act only
     on Your own behalf and on Your sole responsibility, not on behalf
     of any other Contributor, and only if You agree to indemnify,
     defend, and hold each Contributor harmless for any liability
     incurred by, or claims asserted against, such Contributor by reason
     of your accepting any such warranty or additional liability.

  END OF TERMS AND CONDITIONS

  APPENDIX: How to apply the Apache License to your work.

     To apply the Apache License to your work, attach the following
     boilerplate notice, with the fields enclosed by brackets "[]"
     replaced with your own identifying information. (Don't include
     the brackets!)  The text should be enclosed in the appropriate
     comment syntax for the file format. We also recommend that a
     file or class name and description of purpose be included on the
     same "printed page" as the copyright notice for easier
     identification within third-party archives.

  Copyright [yyyy] [name of copyright owner]

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.

*/
