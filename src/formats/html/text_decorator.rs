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

//! HTML parsing, plain text rendering, and HTML table extraction utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// A plain-text decorator for html2text that renders links and text without
/// Markdown-isms such as bracketed links.
#[derive(Clone, Debug, Default)]
pub struct PlainTextDecorator;

impl PlainTextDecorator {
    /// Create a new `PlainTextDecorator`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl html2text::render::TextDecorator for PlainTextDecorator {
    type Annotation = ();

    fn decorate_link_start(
        &mut self,
        _url: &str,
    ) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_link_end(&mut self) -> String {
        String::new()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_em_end(&self) -> String {
        String::new()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strong_end(&self) -> String {
        String::new()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_strikeout_end(&self) -> String {
        String::new()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        (String::new(), ())
    }

    fn decorate_code_end(&self) -> String {
        String::new()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {}

    fn decorate_preformat_cont(&self) -> Self::Annotation {}

    fn decorate_image(
        &mut self,
        _src: &str,
        title: &str,
    ) -> (String, Self::Annotation) {
        (title.to_string(), ())
    }

    fn header_prefix(&self, level: usize) -> String {
        let mut s = String::new();
        for _ in 0..level {
            s.push('#');
        }
        s.push(' ');
        s
    }

    fn quote_prefix(&self) -> String {
        "> ".to_string()
    }

    fn unordered_item_prefix(&self) -> String {
        "* ".to_string()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        format!("{i}. ")
    }

    fn make_subblock_decorator(&self) -> Self {
        self.clone()
    }
}
