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

//! HTML to Markdown conversion engine with table and formatting support.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{
    ElementFlags, NodeOrText, QuirksMode, TreeBuilderOpts, TreeSink,
};
use html5ever::{Attribute, ExpandedName, QualName};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::io::Cursor;
use std::rc::{Rc, Weak};

/// A DOM node representation for HTML parsing.
struct Node {
    parent: Cell<Option<Weak<Node>>>,
    children: RefCell<Vec<Rc<Node>>>,
    data: NodeData,
}

enum NodeData {
    Document,
    Doctype,
    Text(RefCell<String>),
    Comment(()),
    Element {
        name: QualName,
        attrs: RefCell<Vec<Attribute>>,
    },
}

impl Node {
    fn new(data: NodeData) -> Rc<Self> {
        Rc::new(Node {
            parent: Cell::new(None),
            children: RefCell::new(Vec::new()),
            data,
        })
    }
}

struct DomSink {
    document: Rc<Node>,
    errors: RefCell<Vec<Cow<'static, str>>>,
    quirks_mode: Cell<QuirksMode>,
}

impl DomSink {
    fn new() -> Self {
        DomSink {
            document: Node::new(NodeData::Document),
            errors: RefCell::new(Vec::new()),
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }
    }
}

impl TreeSink for DomSink {
    type Handle = Rc<Node>;
    type Output = Rc<Node>;
    type ElemName<'a> = ExpandedName<'a>;

    fn finish(self) -> Self::Output {
        self.document
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        self.errors.borrow_mut().push(msg);
    }

    fn get_document(&self) -> Self::Handle {
        self.document.clone()
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        Rc::ptr_eq(x, y)
    }

    #[expect(
        clippy::unreachable,
        reason = "TreeSink contract guarantees elem_name is only invoked on elements"
    )]
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        match &target.data {
            NodeData::Element { name, .. } => name.expanded(),
            _ => unreachable!(
                "html5ever TreeSink contract guarantees elem_name is only called on element nodes"
            ),
        }
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        Node::new(NodeData::Element {
            name,
            attrs: RefCell::new(attrs),
        })
    }

    fn create_comment(&self, _text: StrTendril) -> Self::Handle {
        Node::new(NodeData::Comment(()))
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        Node::new(NodeData::Comment(()))
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(node) => {
                node.parent.set(Some(Rc::downgrade(parent)));
                parent.children.borrow_mut().push(node);
            }
            NodeOrText::AppendText(text) => {
                let mut children = parent.children.borrow_mut();
                if let Some(last_child) = children.last() {
                    if let NodeData::Text(ref contents) = last_child.data {
                        contents.borrow_mut().push_str(&text);
                        return;
                    }
                }
                let new_node =
                    Node::new(NodeData::Text(RefCell::new(text.to_string())));
                new_node.parent.set(Some(Rc::downgrade(parent)));
                children.push(new_node);
            }
        }
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let Some(weak_parent) = sibling.parent.take() else {
            return;
        };
        let Some(parent) = weak_parent.upgrade() else {
            return;
        };
        sibling.parent.set(Some(weak_parent));

        let mut children = parent.children.borrow_mut();
        let Some(pos) = children.iter().position(|n| Rc::ptr_eq(n, sibling))
        else {
            return;
        };

        match child {
            NodeOrText::AppendNode(node) => {
                node.parent.set(Some(Rc::downgrade(&parent)));
                children.insert(pos, node);
            }
            NodeOrText::AppendText(text) => {
                if pos > 0 {
                    let prev_idx = pos.saturating_sub(1);
                    if let Some(prev) = children.get(prev_idx) {
                        if let NodeData::Text(ref contents) = prev.data {
                            contents.borrow_mut().push_str(&text);
                            return;
                        }
                    }
                }
                let new_node =
                    Node::new(NodeData::Text(RefCell::new(text.to_string())));
                new_node.parent.set(Some(Rc::downgrade(&parent)));
                children.insert(pos, new_node);
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if element.parent.take().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        let doctype = Node::new(NodeData::Doctype);
        doctype.parent.set(Some(Rc::downgrade(&self.document)));
        self.document.children.borrow_mut().push(doctype);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        target.clone()
    }

    fn add_attrs_if_missing(
        &self,
        target: &Self::Handle,
        attrs: Vec<Attribute>,
    ) {
        if let NodeData::Element {
            attrs: ref current_attrs,
            ..
        } = target.data
        {
            let mut current = current_attrs.borrow_mut();
            for attr in attrs {
                if !current.iter().any(|a| a.name == attr.name) {
                    current.push(attr);
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let Some(weak_parent) = target.parent.take() else {
            return;
        };
        let Some(parent) = weak_parent.upgrade() else {
            return;
        };
        let mut children = parent.children.borrow_mut();
        if let Some(pos) = children.iter().position(|n| Rc::ptr_eq(n, target)) {
            children.remove(pos);
        }
    }

    fn reparent_children(
        &self,
        node: &Self::Handle,
        new_parent: &Self::Handle,
    ) {
        let mut node_children = node.children.borrow_mut();
        let mut new_children = new_parent.children.borrow_mut();
        for child in node_children.drain(..) {
            child.parent.set(Some(Rc::downgrade(new_parent)));
            new_children.push(child);
        }
    }
}

/// Column alignment for Markdown tables.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ColumnAlign {
    Left,
    Center,
    Right,
    None,
}

/// Cell structure for HTML tables.
struct TableCell {
    content: String,
    is_header: bool,
    align: ColumnAlign,
}

/// Row structure for HTML tables.
struct TableRow {
    cells: Vec<TableCell>,
}

/// Parse HTML byte slice into DOM tree.
fn parse_html(html: &[u8]) -> Result<Rc<Node>> {
    let cursor = Cursor::new(html);
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    parse_document(DomSink::new(), opts)
        .from_utf8()
        .read_from(&mut { cursor })
        .map_err(|e| anyhow::anyhow!("Failed to parse HTML document: {e}"))
}

/// Convert an HTML document to Markdown formatted string.
pub fn html_to_markdown(html: &[u8]) -> Result<String> {
    let doc = parse_html(html)?;
    let mut out = String::new();
    render_children(&doc, &mut out, &RenderContext::default())?;
    Ok(clean_output(&out))
}

#[derive(Clone, Default)]
struct RenderContext {
    list_depth: usize,
    ordered_index: Option<usize>,
    in_pre: bool,
    in_blockquote: bool,
    blockquote_depth: usize,
    in_table_cell: bool,
}

fn get_attr(attrs: &[Attribute], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|a| a.name.local.as_ref() == name)
        .map(|a| a.value.to_string())
}

fn get_align_from_attrs(attrs: &[Attribute]) -> ColumnAlign {
    if let Some(align) = get_attr(attrs, "align") {
        match align.to_ascii_lowercase().as_str() {
            "left" => return ColumnAlign::Left,
            "center" => return ColumnAlign::Center,
            "right" => return ColumnAlign::Right,
            _ => {}
        }
    }
    if let Some(style) = get_attr(attrs, "style") {
        let style_lower = style.to_ascii_lowercase();
        if style_lower.contains("text-align: center")
            || style_lower.contains("text-align:center")
        {
            return ColumnAlign::Center;
        }
        if style_lower.contains("text-align: right")
            || style_lower.contains("text-align:right")
        {
            return ColumnAlign::Right;
        }
        if style_lower.contains("text-align: left")
            || style_lower.contains("text-align:left")
        {
            return ColumnAlign::Left;
        }
    }
    ColumnAlign::None
}

fn render_children(
    node: &Rc<Node>,
    out: &mut String,
    ctx: &RenderContext,
) -> Result<()> {
    let children = node.children.borrow();
    for child in children.iter() {
        render_node(child, out, ctx)?;
    }
    Ok(())
}

fn render_node(
    node: &Rc<Node>,
    out: &mut String,
    ctx: &RenderContext,
) -> Result<()> {
    match &node.data {
        NodeData::Document => render_children(node, out, ctx)?,
        NodeData::Doctype | NodeData::Comment(()) => {}
        NodeData::Text(text) => {
            let s = text.borrow();
            if ctx.in_pre {
                out.push_str(&s);
            } else {
                let normalized = normalize_whitespace(&s);
                out.push_str(&normalized);
            }
        }
        NodeData::Element { name, attrs } => {
            let tag = name.local.as_ref();
            let attrs_borrow = attrs.borrow();
            render_element(node, tag, &attrs_borrow, out, ctx)?;
        }
    }
    Ok(())
}

fn normalize_whitespace(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut prev_is_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_is_ws {
                res.push(' ');
                prev_is_ws = true;
            }
        } else {
            res.push(ch);
            prev_is_ws = false;
        }
    }
    res
}

fn ensure_blank_lines(out: &mut String, count: usize) {
    let mut trailing_newlines = 0usize;
    for ch in out.chars().rev() {
        if ch == '\n' {
            trailing_newlines = trailing_newlines.saturating_add(1);
        } else {
            break;
        }
    }
    if !out.is_empty() {
        while trailing_newlines < count {
            out.push('\n');
            trailing_newlines = trailing_newlines.saturating_add(1);
        }
    }
}

fn render_element(
    node: &Rc<Node>,
    tag: &str,
    attrs: &[Attribute],
    out: &mut String,
    ctx: &RenderContext,
) -> Result<()> {
    match tag {
        "head" | "script" | "style" | "meta" | "link" | "noscript"
        | "template" => {
            // Ignored elements
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            ensure_blank_lines(out, 2);
            let level = match tag {
                "h1" => 1,
                "h2" => 2,
                "h3" => 3,
                "h4" => 4,
                "h5" => 5,
                "h6" => 6,
                _ => 1,
            };
            for _ in 0..level {
                out.push('#');
            }
            out.push(' ');
            let mut content = String::new();
            render_children(node, &mut content, ctx)?;
            out.push_str(content.trim());
            ensure_blank_lines(out, 2);
        }
        "p" => {
            ensure_blank_lines(out, 2);
            render_children(node, out, ctx)?;
            ensure_blank_lines(out, 2);
        }
        "br" => {
            if ctx.in_table_cell {
                out.push_str("<br>");
            } else {
                out.push('\n');
            }
        }
        "hr" => {
            ensure_blank_lines(out, 2);
            out.push_str("---");
            ensure_blank_lines(out, 2);
        }
        "strong" | "b" => {
            let mut content = String::new();
            render_children(node, &mut content, ctx)?;
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                if content.starts_with(' ') {
                    out.push(' ');
                }
                out.push_str("**");
                out.push_str(trimmed);
                out.push_str("**");
                if content.ends_with(' ') {
                    out.push(' ');
                }
            }
        }
        "em" | "i" => {
            let mut content = String::new();
            render_children(node, &mut content, ctx)?;
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                if content.starts_with(' ') {
                    out.push(' ');
                }
                out.push('*');
                out.push_str(trimmed);
                out.push('*');
                if content.ends_with(' ') {
                    out.push(' ');
                }
            }
        }
        "s" | "del" | "strike" => {
            let mut content = String::new();
            render_children(node, &mut content, ctx)?;
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                if content.starts_with(' ') {
                    out.push(' ');
                }
                out.push_str("~~");
                out.push_str(trimmed);
                out.push_str("~~");
                if content.ends_with(' ') {
                    out.push(' ');
                }
            }
        }
        "code" => {
            if ctx.in_pre {
                render_children(node, out, ctx)?;
            } else {
                let mut content = String::new();
                let mut inner_ctx = ctx.clone();
                inner_ctx.in_pre = true;
                render_children(node, &mut content, &inner_ctx)?;
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    out.push('`');
                    out.push_str(trimmed);
                    out.push('`');
                }
            }
        }
        "pre" => {
            ensure_blank_lines(out, 2);
            let mut lang = String::new();
            // Look for <code class="language-xyz"> inside <pre>
            for child in node.children.borrow().iter() {
                if let NodeData::Element {
                    ref name,
                    ref attrs,
                } = child.data
                {
                    if name.local.as_ref() == "code" {
                        if let Some(class) = get_attr(&attrs.borrow(), "class")
                        {
                            for part in class.split_whitespace() {
                                if let Some(l) = part.strip_prefix("language-")
                                {
                                    lang = l.to_string();
                                    break;
                                }
                                if let Some(l) = part.strip_prefix("lang-") {
                                    lang = l.to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            out.push_str("```");
            out.push_str(&lang);
            out.push('\n');

            let mut code_content = String::new();
            let mut pre_ctx = ctx.clone();
            pre_ctx.in_pre = true;
            render_children(node, &mut code_content, &pre_ctx)?;

            // Strip single leading/trailing newline if any
            let trimmed_code = code_content.trim_matches('\n');
            out.push_str(trimmed_code);
            out.push('\n');
            out.push_str("```");
            ensure_blank_lines(out, 2);
        }
        "blockquote" => {
            ensure_blank_lines(out, 2);
            let mut inner = String::new();
            let mut bq_ctx = ctx.clone();
            bq_ctx.in_blockquote = true;
            bq_ctx.blockquote_depth =
                bq_ctx.blockquote_depth.saturating_add(1);
            render_children(node, &mut inner, &bq_ctx)?;

            let trimmed_inner = inner.trim();
            for line in trimmed_inner.lines() {
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
            ensure_blank_lines(out, 2);
        }
        "ul" => {
            ensure_blank_lines(out, 1);
            let mut list_ctx = ctx.clone();
            list_ctx.list_depth = ctx.list_depth.saturating_add(1);
            list_ctx.ordered_index = None;
            render_children(node, out, &list_ctx)?;
            ensure_blank_lines(out, 1);
        }
        "ol" => {
            ensure_blank_lines(out, 1);
            let mut list_ctx = ctx.clone();
            list_ctx.list_depth = ctx.list_depth.saturating_add(1);
            let start: usize = match get_attr(attrs, "start").and_then(|s| s.parse::<usize>().ok()) {
                Some(s) => s,
                None => 1,
            };
            list_ctx.ordered_index = Some(start);
            render_children(node, out, &list_ctx)?;
            ensure_blank_lines(out, 1);
        }
        "li" => {
            ensure_blank_lines(out, 1);
            let indent_spaces = ctx.list_depth.saturating_sub(1).saturating_mul(2);
            for _ in 0..indent_spaces {
                out.push(' ');
            }
            if let Some(idx) = ctx.ordered_index {
                out.push_str(&format!("{idx}. "));
            } else {
                out.push_str("* ");
            }
            let mut item_content = String::new();
            render_children(node, &mut item_content, ctx)?;
            let trimmed = item_content.trim();
            out.push_str(trimmed);
            out.push('\n');
        }
        "a" => {
            let href = get_attr(attrs, "href");
            let title = get_attr(attrs, "title");
            let mut link_text = String::new();
            render_children(node, &mut link_text, ctx)?;
            let trimmed_text = link_text.trim();

            if let Some(url) = href {
                if trimmed_text.is_empty() {
                    out.push_str(&format!("<{url}>"));
                } else if let Some(t) = title {
                    out.push_str(&format!("[{trimmed_text}]({url} \"{t}\")"));
                } else {
                    out.push_str(&format!("[{trimmed_text}]({url})"));
                }
            } else {
                out.push_str(trimmed_text);
            }
        }
        "img" => {
            let src = match get_attr(attrs, "src") {
                Some(s) => s,
                None => "",
            };
            let alt = match get_attr(attrs, "alt") {
                Some(s) => s,
                None => "",
            };
            let title = get_attr(attrs, "title");
            if let Some(t) = title {
                out.push_str(&format!("![{alt}]({src} \"{t}\")"));
            } else {
                out.push_str(&format!("![{alt}]({src})"));
            }
        }
        "table" => {
            ensure_blank_lines(out, 2);
            render_table(node, out)?;
            ensure_blank_lines(out, 2);
        }
        "div" | "article" | "section" | "main" | "header" | "footer" | "nav" => {
            ensure_blank_lines(out, 1);
            render_children(node, out, ctx)?;
            ensure_blank_lines(out, 1);
        }
        _ => {
            render_children(node, out, ctx)?;
        }
    }
    Ok(())
}

fn collect_table_rows(node: &Rc<Node>, rows: &mut Vec<TableRow>) -> Result<()> {
    for child in node.children.borrow().iter() {
        if let NodeData::Element {
            ref name,
            ..
        } = child.data
        {
            let tag = name.local.as_ref();
            match tag {
                "tr" => {
                    let mut cells = Vec::new();
                    for cell_node in child.children.borrow().iter() {
                        if let NodeData::Element {
                            ref name,
                            ref attrs,
                        } = cell_node.data
                        {
                            let cell_tag = name.local.as_ref();
                            if cell_tag == "td" || cell_tag == "th" {
                                let is_header = cell_tag == "th";
                                let align =
                                    get_align_from_attrs(&attrs.borrow());
                                let mut cell_content = String::new();
                                let cell_ctx = RenderContext {
                                    in_table_cell: true,
                                    ..Default::default()
                                };
                                render_children(
                                    cell_node,
                                    &mut cell_content,
                                    &cell_ctx,
                                )?;
                                // Clean cell content: replace newlines with space, escape pipes
                                let cleaned = sanitize_cell_content(
                                    cell_content.trim(),
                                );
                                cells.push(TableCell {
                                    content: cleaned,
                                    is_header,
                                    align,
                                });
                            }
                        }
                    }
                    if !cells.is_empty() {
                        rows.push(TableRow { cells });
                    }
                }
                "thead" | "tbody" | "tfoot" => {
                    collect_table_rows(child, rows)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn sanitize_cell_content(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut prev_is_ws = false;
    for ch in content.chars() {
        match ch {
            '|' => {
                out.push_str("\\|");
                prev_is_ws = false;
            }
            '\n' | '\r' | '\t' => {
                if !prev_is_ws {
                    out.push(' ');
                    prev_is_ws = true;
                }
            }
            c => {
                if c.is_whitespace() {
                    if !prev_is_ws {
                        out.push(' ');
                        prev_is_ws = true;
                    }
                } else {
                    out.push(c);
                    prev_is_ws = false;
                }
            }
        }
    }
    out
}

fn render_table(table_node: &Rc<Node>, out: &mut String) -> Result<()> {
    let mut rows = Vec::new();
    collect_table_rows(table_node, &mut rows)?;

    if rows.is_empty() {
        return Ok(());
    }

    let col_count = match rows.iter().map(|r| r.cells.len()).max() {
        Some(c) => c,
        None => 0,
    };
    if col_count == 0 {
        return Ok(());
    }

    // Determine column alignments
    let mut aligns = vec![ColumnAlign::None; col_count];
    for row in &rows {
        for (i, cell) in row.cells.iter().enumerate() {
            if i < col_count && cell.align != ColumnAlign::None {
                if let Some(align_slot) = aligns.get_mut(i) {
                    *align_slot = cell.align;
                }
            }
        }
    }

    let first_row_is_header =
        rows.first().is_some_and(|r| r.cells.iter().any(|c| c.is_header));

    let (header_cells, data_rows_start) = if first_row_is_header {
        (
            rows.first().map(|r| &r.cells),
            1usize,
        )
    } else {
        // If no explicit <th> in the table, use first row as header
        (
            rows.first().map(|r| &r.cells),
            1usize,
        )
    };

    // Header row
    out.push('|');
    for col_idx in 0..col_count {
        out.push(' ');
        if let Some(cells) = header_cells {
            if let Some(cell) = cells.get(col_idx) {
                out.push_str(&cell.content);
            }
        }
        out.push_str(" |");
    }
    out.push('\n');

    // Delimiter row
    out.push('|');
    for col_idx in 0..col_count {
        let align = match aligns.get(col_idx) {
            Some(&a) => a,
            None => ColumnAlign::None,
        };
        match align {
            ColumnAlign::Left => out.push_str(":---"),
            ColumnAlign::Center => out.push_str(":---:"),
            ColumnAlign::Right => out.push_str("---:"),
            ColumnAlign::None => out.push_str("---"),
        }
        out.push_str(" |");
    }
    out.push('\n');

    // Data rows
    for row in rows.iter().skip(data_rows_start) {
        out.push('|');
        for col_idx in 0..col_count {
            out.push(' ');
            if let Some(cell) = row.cells.get(col_idx) {
                out.push_str(&cell.content);
            }
            out.push_str(" |");
        }
        out.push('\n');
    }

    Ok(())
}

fn clean_output(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut consecutive_newlines = 0usize;

    for line in s.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            consecutive_newlines = consecutive_newlines.saturating_add(1);
            if consecutive_newlines <= 2 {
                result.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(trimmed_end);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

/// Convert HTML bytes to Markdown.
pub fn html2md(html: Vec<u8>) -> Result<Vec<u8>> {
    let md = html_to_markdown(html.as_slice())?;
    Ok(md.into_bytes())
}

/// Convert HTML bytes to Markdown with width specification.
pub fn html2md_with_width(html: Vec<u8>, _width: u16) -> Result<Vec<u8>> {
    html2md(html)
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
    use super::*;

    #[crate::ctb_test]
    fn test_html2md_links_and_images() -> Result<()> {
        let html = b"<p>Check <a href=\"https://example.com/\">this link</a> and <img src=\"img.png\" alt=\"An image\" />.</p>";
        let md = html2md(html.to_vec())?;
        let md_str = String::from_utf8(md)?;
        assert_eq!(
            md_str.trim(),
            "Check [this link](https://example.com/) and ![An image](img.png)."
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html2md_formatting_and_code() -> Result<()> {
        let html = b"<p><b>Bold</b>, <i>Italic</i>, <code>inline code</code>, <del>deleted</del>.</p>";
        let md = html2md(html.to_vec())?;
        let md_str = String::from_utf8(md)?;
        assert_eq!(
            md_str.trim(),
            "**Bold**, *Italic*, `inline code`, ~~deleted~~."
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html2md_headings_and_pre() -> Result<()> {
        let html = b"<h1>Heading 1</h1><h2>Heading 2</h2><pre><code class=\"language-rust\">fn main() {\n    println!(\"Hello\");\n}</code></pre>";
        let md = html2md(html.to_vec())?;
        let md_str = String::from_utf8(md)?;
        assert!(md_str.contains("# Heading 1"));
        assert!(md_str.contains("## Heading 2"));
        assert!(md_str.contains("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html2md_table_with_alignment_and_escaping() -> Result<()> {
        let html = b"<table>\
            <thead>\
                <tr><th align=\"left\">Item</th><th align=\"center\">Qty</th><th align=\"right\">Price</th></tr>\
            </thead>\
            <tbody>\
                <tr><td>Widget A | Standard</td><td>10</td><td>$5.00</td></tr>\
                <tr><td>Widget B</td><td>2</td><td>$12.50</td></tr>\
            </tbody>\
        </table>";
        let md = html2md(html.to_vec())?;
        let md_str = String::from_utf8(md)?;
        assert!(md_str.contains("| Item | Qty | Price |"));
        assert!(md_str.contains("| :--- | :---: | ---: |"));
        assert!(md_str.contains("| Widget A \\| Standard | 10 | $5.00 |"));
        assert!(md_str.contains("| Widget B | 2 | $12.50 |"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_html2md_table_without_thead() -> Result<()> {
        let html = b"<table>\
            <tr><td>First</td><td>Second</td></tr>\
            <tr><td>1</td><td>2</td></tr>\
        </table>";
        let md = html2md(html.to_vec())?;
        let md_str = String::from_utf8(md)?;
        assert!(md_str.contains("| First | Second |"));
        assert!(md_str.contains("| --- | --- |"));
        assert!(md_str.contains("| 1 | 2 |"));
        Ok(())
    }
}