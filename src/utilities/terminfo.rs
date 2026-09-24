// SPDX-License-Identifier: AGPL-3.0-or-later AND WTFPL
// SPDX-License-Identifier for parts derived from `terminfo` crate: WTFPL
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

// Parts derived from `terminfo` crate:
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co

// See full license details at end of this file.

//! Embedded terminfo database parser and query interface.
//!
//! Parses bundled ncurses and xterm terminfo source files
//! (`terminfo.src.ncurses` and `terminfo.xterm`), resolves entry inheritance
//! (`use=`), unescapes string capabilities, and provides capability queries
//! matching standard terminfo conventions without requiring the `tic` compiler.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::{Mutex, OnceLock};

/// Parsed terminal capabilities description from the terminfo database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminfo {
    /// Primary terminal name (e.g. `"xterm-256color"`).
    pub name: String,
    /// Terminal aliases (e.g. `["xterm", "xterm-color"]`).
    pub aliases: Vec<String>,
    /// Human-readable terminal description.
    pub description: String,
    /// Boolean capabilities (e.g. `"am"`, `"bce"`, `"RGB"`).
    pub booleans: HashSet<String>,
    /// Numeric capabilities (e.g. `"colors"`, `"cols"`, `"lines"`).
    pub numbers: HashMap<String, i32>,
    /// String capabilities unescaped as raw bytes (e.g. `"cup"`, `"kmous"`).
    pub strings: HashMap<String, Vec<u8>>,
}

impl Terminfo {
    /// Look up a terminal profile by the `$TERM` environment variable.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let term = env::var("TERM").ok()?;
        if term.is_empty() {
            return None;
        }
        Self::from_name(&term)
    }

    /// Look up a terminal profile by name or alias from the bundled database.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        get_cached_or_parse_entry(name)
    }

    /// Return true if the given boolean capability is present and true.
    #[must_use]
    pub fn get_bool(&self, name: &str) -> bool {
        self.booleans.contains(name)
    }

    /// Return the numeric value of the given capability if present.
    #[must_use]
    pub fn get_num(&self, name: &str) -> Option<i32> {
        self.numbers.get(name).copied()
    }

    /// Return the raw byte sequence of the given string capability.
    #[must_use]
    pub fn get_str(&self, name: &str) -> Option<&[u8]> {
        self.strings.get(name).map(Vec::as_slice)
    }

    /// Return the string capability decoded as lossy UTF-8 if present.
    #[must_use]
    pub fn get_str_lossy(&self, name: &str) -> Option<String> {
        self.get_str(name)
            .map(|s| String::from_utf8_lossy(s).into_owned())
    }

    /// Maximum number of colors supported by the terminal (`colors`).
    #[must_use]
    pub fn max_colors(&self) -> i32 {
        self.get_num("colors").unwrap_or(0)
    }

    /// Whether the terminal supports mouse input reporting.
    #[must_use]
    pub fn has_mouse(&self) -> bool {
        self.get_str("kmous").is_some()
            || self.get_bool("XM")
            || self.get_str("reqmp").is_some()
            || self.get_str("minfo").is_some()
    }

    /// Whether the terminal supports arbitrary cursor addressing (`cup`).
    #[must_use]
    pub fn can_cursor_address(&self) -> bool {
        self.get_str("cup").is_some()
    }

    /// Whether the terminal supports editing or clearing on the current line.
    #[must_use]
    pub fn can_edit_line(&self) -> bool {
        self.get_str("el").is_some()
            || self.get_str("cub1").is_some()
            || self.get_str("cub").is_some()
    }

    /// Whether the terminal supports direct truecolor / 24-bit color.
    #[must_use]
    pub fn has_truecolor(&self) -> bool {
        self.get_bool("RGB")
            || self.get_bool("Tc")
            || self.max_colors() >= 16_777_216
            || self
                .get_str("setaf")
                .is_some_and(|s| s.windows(4).any(|w| w == b":2::"))
    }

    /// Whether the terminal description matches standard VT100 / ANSI.
    #[must_use]
    pub fn is_vt100_compatible(&self) -> bool {
        let is_vt_name = |s: &str| {
            s.starts_with("vt100")
                || s.starts_with("vt102")
                || s.starts_with("vt220")
                || s.starts_with("xterm")
                || s.starts_with("rxvt")
                || s.starts_with("screen")
                || s.starts_with("tmux")
                || s.starts_with("linux")
                || s.starts_with("alacritty")
                || s.starts_with("kitty")
                || s.starts_with("ms-terminal")
                || s == "ansi"
        };
        if is_vt_name(&self.name) || self.aliases.iter().any(|a| is_vt_name(a)) {
            return true;
        }
        if let Some(cup) = self.get_str("cup") {
            if cup.starts_with(b"\x1b[") {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Copy)]
struct EntryLocation {
    source_id: u8,
    start: usize,
    end: usize,
}

static INDEX: OnceLock<HashMap<String, EntryLocation>> = OnceLock::new();
static CACHE: OnceLock<Mutex<HashMap<String, Terminfo>>> = OnceLock::new();

fn get_source(id: u8) -> Option<&'static str> {
    match id {
        0 => get_utilities_data_str("terminfo/terminfo.xterm"),
        1 => get_utilities_data_str("terminfo/terminfo.src.ncurses"),
        _ => None,
    }
}

fn build_index() -> HashMap<String, EntryLocation> {
    let mut map = HashMap::new();
    // Index ncurses (1) first, then xterm (0) so xterm definitions take precedence
    for &id in &[1u8, 0u8] {
        let Some(text) = get_source(id) else {
            continue;
        };
        index_source(text, id, &mut map);
    }
    map
}

fn index_source(
    text: &'static str,
    source_id: u8,
    map: &mut HashMap<String, EntryLocation>,
) {
    let mut current_start: Option<usize> = None;
    let mut current_names: Vec<String> = Vec::new();
    let mut offset = 0usize;

    for line in text.split_inclusive('\n') {
        let line_len = line.len();
        let next_offset = offset.saturating_add(line_len);

        let trimmed = line.trim();
        let is_comment = line.starts_with('#');
        let is_empty = trimmed.is_empty();
        let is_indented = line.starts_with(' ') || line.starts_with('\t');

        if !is_comment && !is_empty && !is_indented {
            if let Some(start) = current_start {
                let loc = EntryLocation {
                    source_id,
                    start,
                    end: offset,
                };
                for name in current_names.drain(..) {
                    map.insert(name, loc);
                }
            }

            let mut header = line.trim_end();
            if let Some(stripped) = header.strip_suffix(',') {
                header = stripped;
            }
            let parts: Vec<&str> = header.split('|').map(str::trim).collect();
            let mut names = Vec::new();
            if parts.len() <= 1 {
                if let Some(&first) = parts.first() {
                    if !first.is_empty() {
                        names.push(first.to_ascii_lowercase());
                    }
                }
            } else {
                let names_count = parts.len().saturating_sub(1);
                for &part in parts.iter().take(names_count) {
                    if !part.is_empty() {
                        names.push(part.to_ascii_lowercase());
                    }
                }
            }

            current_start = Some(offset);
            current_names = names;
        }

        offset = next_offset;
    }

    if let Some(start) = current_start {
        let loc = EntryLocation {
            source_id,
            start,
            end: offset,
        };
        for name in current_names {
            map.insert(name, loc);
        }
    }
}

struct RawParsedEntry {
    primary_name: String,
    aliases: Vec<String>,
    description: String,
    booleans: HashSet<String>,
    numbers: HashMap<String, i32>,
    strings: HashMap<String, Vec<u8>>,
    cancelled: HashSet<String>,
    uses: Vec<String>,
}

fn parse_raw_entry(entry_slice: &str) -> Option<RawParsedEntry> {
    let (header_line, body) = match entry_slice.split_once('\n') {
        Some((h, b)) => (h, b),
        None => (entry_slice, ""),
    };

    let mut header = header_line.trim_end();
    if let Some(stripped) = header.strip_suffix(',') {
        header = stripped;
    }
    let parts: Vec<&str> = header.split('|').map(str::trim).collect();
    if parts.is_empty() {
        return None;
    }

    let primary_name = (*parts.first()?).to_string();
    let mut aliases = Vec::new();
    let mut description = String::new();

    if parts.len() > 1 {
        let names_count = parts.len().saturating_sub(1);
        for &alias in parts.iter().skip(1).take(names_count.saturating_sub(1)) {
            if !alias.is_empty() {
                aliases.push(alias.to_string());
            }
        }
        if let Some(&desc) = parts.last() {
            description = desc.to_string();
        }
    }

    // Strip comments within the entry body lines
    let mut clean_body = String::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        clean_body.push_str(line);
        clean_body.push(' ');
    }

    let mut booleans = HashSet::new();
    let mut numbers = HashMap::new();
    let mut strings = HashMap::new();
    let mut cancelled = HashSet::new();
    let mut uses = Vec::new();

    let chunks = split_capabilities(&clean_body);
    for chunk in chunks {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }

        if chunk.starts_with('@') || chunk.ends_with('@') {
            let name = chunk.trim_matches('@').trim();
            if !name.is_empty() {
                cancelled.insert(name.to_string());
            }
        } else if let Some((k, v)) = chunk.split_once('#') {
            let k = k.trim();
            if !cancelled.contains(k) {
                if let Some(val) = parse_number(v) {
                    numbers.insert(k.to_string(), val);
                }
            }
        } else if let Some((k, v)) = chunk.split_once('=') {
            let k = k.trim();
            let v = v.trim();
            if k == "use" {
                uses.push(v.to_ascii_lowercase());
            } else if !cancelled.contains(k) {
                strings.insert(k.to_string(), unescape_terminfo_string(v));
            }
        } else {
            let k = chunk.trim();
            if !k.is_empty() && !cancelled.contains(k) {
                booleans.insert(k.to_string());
            }
        }
    }

    Some(RawParsedEntry {
        primary_name,
        aliases,
        description,
        booleans,
        numbers,
        strings,
        cancelled,
        uses,
    })
}

fn split_capabilities(body: &str) -> Vec<String> {
    let mut caps = Vec::new();
    let mut current = String::new();
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            current.push('\\');
            if let Some(next_ch) = chars.next() {
                current.push(next_ch);
            }
        } else if ch == ',' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                caps.push(trimmed.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        caps.push(trimmed.to_string());
    }
    caps
}

fn parse_number(s: &str) -> Option<i32> {
    let s = s.trim();
    if let Some(hex_str) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i32::from_str_radix(hex_str, 16).ok()
    } else {
        s.parse::<i32>().ok()
    }
}

fn unescape_terminfo_string(raw: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('E' | 'e') => out.push(0x1B),
                Some('a') => out.push(0x07),
                Some('b') => out.push(0x08),
                Some('f') => out.push(0x0C),
                Some('n' | 'l') => out.push(b'\n'),
                Some('r') => out.push(b'\r'),
                Some('t') => out.push(b'\t'),
                Some('s') => out.push(b' '),
                Some('^') => out.push(b'^'),
                Some('\\') => out.push(b'\\'),
                Some(',') => out.push(b','),
                Some(':') => out.push(b':'),
                Some(d @ '0'..='7') => {
                    let mut oct_val = match d {
                        '0' => 0u8,
                        '1' => 1,
                        '2' => 2,
                        '3' => 3,
                        '4' => 4,
                        '5' => 5,
                        '6' => 6,
                        '7' => 7,
                        _ => 0,
                    };
                    for _ in 0..2 {
                        if let Some(&next_d @ '0'..='7') = chars.peek() {
                            let _ = chars.next();
                            let digit = match next_d {
                                '0' => 0u8,
                                '1' => 1,
                                '2' => 2,
                                '3' => 3,
                                '4' => 4,
                                '5' => 5,
                                '6' => 6,
                                '7' => 7,
                                _ => 0,
                            };
                            if let Some(next_val) = oct_val
                                .checked_mul(8)
                                .and_then(|v| v.checked_add(digit))
                            {
                                oct_val = next_val;
                            }
                        } else {
                            break;
                        }
                    }
                    out.push(oct_val);
                }
                Some(other) => {
                    if let Ok(b) = u8::try_from(other) {
                        out.push(b);
                    }
                }
                None => out.push(b'\\'),
            }
        } else if ch == '^' {
            if let Some(&next_ch) = chars.peek() {
                if let Ok(b) = u8::try_from(next_ch) {
                    let _ = chars.next();
                    if (b'@'..=b'_').contains(&b) {
                        if let Some(ctrl) = b.checked_sub(b'@') {
                            out.push(ctrl);
                        }
                    } else if (b'a'..=b'z').contains(&b) {
                        if let Some(ctrl) =
                            b.checked_sub(b'a').and_then(|v| v.checked_add(1))
                        {
                            out.push(ctrl);
                        }
                    } else if b == b'?' {
                        out.push(127);
                    } else {
                        out.push(b'^');
                        out.push(b);
                    }
                } else {
                    out.push(b'^');
                }
            } else {
                out.push(b'^');
            }
        } else if let Ok(b) = u8::try_from(ch) {
            out.push(b);
        }
    }

    out
}

fn resolve_entry(
    name: &str,
    index: &HashMap<String, EntryLocation>,
    visited: &mut HashSet<String>,
) -> Option<Terminfo> {
    let lower_name = name.to_ascii_lowercase();
    if !visited.insert(lower_name.clone()) {
        return None;
    }

    let loc = index.get(&lower_name)?;
    let text = get_source(loc.source_id)?;
    let entry_slice = text.get(loc.start..loc.end)?;

    let raw = parse_raw_entry(entry_slice)?;

    let mut booleans = raw.booleans;
    let mut numbers = raw.numbers;
    let mut strings = raw.strings;
    let cancelled = raw.cancelled;

    for use_name in &raw.uses {
        if let Some(parent) = resolve_entry(use_name, index, visited) {
            for b in parent.booleans {
                if !cancelled.contains(&b) {
                    booleans.insert(b);
                }
            }
            for (k, v) in parent.numbers {
                if !cancelled.contains(&k) {
                    numbers.entry(k).or_insert(v);
                }
            }
            for (k, v) in parent.strings {
                if !cancelled.contains(&k) {
                    strings.entry(k).or_insert(v);
                }
            }
        }
    }

    Some(Terminfo {
        name: raw.primary_name,
        aliases: raw.aliases,
        description: raw.description,
        booleans,
        numbers,
        strings,
    })
}

fn get_cached_or_parse_entry(name: &str) -> Option<Terminfo> {
    let lower = name.to_ascii_lowercase();
    let cache_lock = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache_lock.lock() {
        if let Some(cached) = guard.get(&lower) {
            return Some(cached.clone());
        }
    }

    let index = INDEX.get_or_init(build_index);
    let mut visited = HashSet::new();
    let resolved = resolve_entry(&lower, index, &mut visited)?;

    if let Ok(mut guard) = cache_lock.lock() {
        guard.insert(lower, resolved.clone());
    }
    Some(resolved)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Standard test boilerplate"
)]
mod tests {
    use super::{Terminfo, unescape_terminfo_string};
    use std::panic;

    #[crate::ctb_test]
    fn test_xterm_256color_detection() {
        let info = Terminfo::from_name("xterm-256color")
            .expect("xterm-256color must exist in bundled terminfo");

        assert_eq!(info.max_colors(), 256);
        assert!(info.has_mouse(), "xterm-256color must support mouse");
        assert!(info.can_cursor_address(), "xterm-256color must have cup");
        assert!(info.can_edit_line(), "xterm-256color must have el/cub1");
        assert!(info.is_vt100_compatible(), "xterm is vt100 compatible");
    }

    #[crate::ctb_test]
    fn test_ms_terminal_detection() {
        let info = Terminfo::from_name("ms-terminal")
            .expect("ms-terminal must exist in bundled terminfo");

        assert_eq!(info.max_colors(), 256);
        assert!(info.has_mouse(), "ms-terminal must support mouse");
        assert!(info.can_cursor_address(), "ms-terminal must have cup");
        assert!(info.is_vt100_compatible(), "ms-terminal is vt100 compatible");
    }

    #[crate::ctb_test]
    fn test_vt100_detection() {
        let info = Terminfo::from_name("vt100")
            .expect("vt100 must exist in bundled terminfo");

        assert_eq!(info.max_colors(), 0);
        assert!(!info.has_mouse(), "Standard vt100 does not support mouse");
        assert!(info.can_cursor_address(), "vt100 has cursor address (cup)");
        assert!(info.is_vt100_compatible(), "vt100 is vt100 compatible");
    }

    #[crate::ctb_test]
    fn test_dumb_terminal() {
        let info = Terminfo::from_name("dumb")
            .expect("dumb must exist in bundled terminfo");

        assert_eq!(info.max_colors(), 0);
        assert!(!info.can_cursor_address(), "dumb terminal lacks cup");
        assert!(!info.has_mouse(), "dumb terminal lacks mouse");
    }

    #[crate::ctb_test]
    fn test_xterm_direct_color() {
        let info = Terminfo::from_name("xterm-direct")
            .expect("xterm-direct must exist in bundled terminfo");

        assert!(info.has_truecolor(), "xterm-direct must have truecolor");
        assert!(info.max_colors() >= 16_777_216);
    }

    #[crate::ctb_test]
    fn test_unescape_sequences() {
        let esc_str = unescape_terminfo_string(r"\E[H\E[2J");
        assert_eq!(esc_str, b"\x1b[H\x1b[2J");

        let cr_str = unescape_terminfo_string(r"^M^H");
        assert_eq!(cr_str, b"\r\x08");

        let oct_str = unescape_terminfo_string(r"\033");
        assert_eq!(oct_str, b"\x1b");
    }
}

/*

LICENSE from `terminfo-0.9.0`:

        DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
                    Version 2, December 2004

 Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co

 Everyone is permitted to copy and distribute verbatim or modified
 copies of this license document, and changing it is allowed as long
 as the name is changed.

            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

  0. You just DO WHAT THE FUCK YOU WANT TO.

*/