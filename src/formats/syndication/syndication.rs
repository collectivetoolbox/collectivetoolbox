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

//! Syndication data model.
//!
//! This module defines in-memory structures (`Feed`, `Entry`) that can later be
//! rendered into various syndication formats (e.g., JSON Feed, Atom, RSS).
//!
//! Note that some formats have special limitations:
//!
//! - RSS 0.90 only allows HTTP, HTTPS, FTP, and mailto URLs
//! - RSS 0.92 and both RSS 0.91 variants only allow HTTP or FTP URLs
//! - RSS 2.0 only allows IANA-registered URI schemes

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use chrono::{DateTime, Utc};
use dateparser::parse;

pub mod atom;
pub mod hatom;
pub mod json_feed;
pub mod rss;

/// Output formats supported by [`Feed::to`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeedFormat {
    /// JSON Feed 1.1.
    JsonFeed,
    /// Atom
    Atom,
    HAtom,
    /// RSS
    Rss,
    Rss1,
    Rss092,
    Rss091Netscape,
    Rss091UserLand,
    Rss09,
    ScriptingNews10,
    ScriptingNews20,
}

/// A single item in a syndication feed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    id: String,
    title: String,
    body_html: String,
    author: Option<String>,
    date_published: String,
    date_updated: Option<String>,
    url: Option<String>,
}

/// Optional channel image metadata.
///
/// Some output formats (e.g. RSS 0.91 UserLand) expect a channel `<image>`.
/// Since the syndication data model is shared across multiple formats, all
/// fields are optional and individual renderers can enforce/derive defaults.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Image {
    url: String,
    alt: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    description: Option<String>,
}

impl Image {
    /// Create an empty image descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set or clear the image URL.
    #[must_use]
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    /// Set or clear the image alt text.
    #[must_use]
    pub fn with_alt_opt(mut self, alt: Option<String>) -> Self {
        self.alt = alt;
        self
    }

    /// Set or clear the image width.
    #[must_use]
    pub fn with_width_opt(mut self, width: Option<u32>) -> Self {
        self.width = width;
        self
    }

    /// Set or clear the image height.
    #[must_use]
    pub fn with_height_opt(mut self, height: Option<u32>) -> Self {
        self.height = height;
        self
    }

    /// Set or clear the image description.
    #[must_use]
    pub fn with_description_opt(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Borrow the image URL, if present.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Borrow the image alt text, if present.
    pub fn alt(&self) -> Option<&str> {
        self.alt.as_deref()
    }

    /// Borrow the image width, if present.
    pub fn width(&self) -> Option<u32> {
        self.width
    }

    /// Borrow the image height, if present.
    pub fn height(&self) -> Option<u32> {
        self.height
    }

    /// Borrow the image description, if present.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl Entry {
    /// Create a new entry.
    ///
    /// The `date_published` is intentionally stored as a string for now; format
    /// and parsing will be decided when output formats are implemented.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        body_html: impl Into<String>,
        date_published: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body_html: body_html.into(),
            author: None,
            date_published: date_published.into(),
            date_updated: None,
            url: None,
        }
    }

    /// Set the author for this entry.
    #[must_use]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set or clear the author for this entry.
    #[must_use]
    pub fn with_author_opt(mut self, author: Option<String>) -> Self {
        self.author = author;
        self
    }

    /// Set or clear the canonical URL for this entry.
    #[must_use]
    pub fn with_url_opt(mut self, url: Option<String>) -> Self {
        self.url = url;
        self
    }

    /// Set or clear the canonical URL for this entry.
    #[must_use]
    pub fn with_updated_opt(mut self, updated: Option<String>) -> Self {
        self.date_updated = updated;
        self
    }

    /// Borrow the entry id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Borrow the entry title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the entry body.
    pub fn body(&self) -> &str {
        &self.body_html
    }

    /// Borrow the entry author, if present.
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Borrow the entry published date string.
    pub fn date(&self) -> &str {
        &self.date_published
    }

    pub fn date_parsed(&self) -> Result<DateTime<Utc>> {
        // FIXME: This uses the dateparser crate to parse various date formats.
        // It doesn't look very maintained and has some bugs/what appear to be
        // limitations particularly around timezone. Look for something better
        // later.
        parse(&self.date_published)
    }

    pub fn date_try_format(&self, fmt: &str) -> String {
        match &self.date_parsed() {
            Ok(dt) => dt.with_timezone(&Utc).format(fmt).to_string(),
            Err(_) => self.date_published.clone(),
        }
    }

    /// Borrow the entry updated date string, if present.
    pub fn updated(&self) -> Option<&str> {
        self.date_updated.as_deref()
    }

    pub fn updated_parsed(&self) -> Result<DateTime<Utc>> {
        parse(bail_if_none!(self.date_updated.as_ref()))
    }

    pub fn updated_try_format(&self, fmt: &str) -> Option<String> {
        match &self.updated_parsed() {
            Ok(dt) => Some(dt.with_timezone(&Utc).format(fmt).to_string()),
            Err(_) => Some(self.date().to_string()),
        }
    }

    pub fn try_updated(&self) -> String {
        match &self.updated() {
            Some(dt) => (*dt).to_string(),
            None => self.date().to_string(),
        }
    }

    pub fn try_updated_parsed(&self) -> Result<DateTime<Utc>> {
        parse(&self.try_updated())
    }

    pub fn try_updated_try_format(&self, fmt: &str) -> String {
        match &self.try_updated_parsed() {
            Ok(dt) => dt.with_timezone(&Utc).format(fmt).to_string(),
            Err(_) => self.try_updated().clone(),
        }
    }

    /// Borrow the entry URL, if present.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

/// A collection of entries intended to be rendered into different formats.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Feed {
    title: String,
    home_page_url: Option<String>,
    #[expect(
        clippy::struct_field_names,
        reason = "feed_url is standard RSS/Atom nomenclature"
    )]
    feed_url: Option<String>,
    image: Option<Image>,
    entries: Vec<Entry>,
}

impl Feed {
    /// Create a new feed from an iterator of entries.
    pub fn new(
        title: impl Into<String>,
        entries: impl IntoIterator<Item = Entry>,
    ) -> Self {
        Self {
            title: title.into(),
            home_page_url: None,
            feed_url: None,
            image: None,
            entries: entries.into_iter().collect(),
        }
    }

    /// Create an empty feed.
    pub fn empty(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            home_page_url: None,
            feed_url: None,
            image: None,
            entries: Vec::new(),
        }
    }

    /// Set or clear the home page URL for this feed.
    #[must_use]
    pub fn with_home_page_url_opt(
        mut self,
        home_page_url: Option<String>,
    ) -> Self {
        self.home_page_url = home_page_url;
        self
    }

    /// Set or clear the feed URL for this feed.
    #[must_use]
    pub fn with_feed_url_opt(mut self, feed_url: Option<String>) -> Self {
        self.feed_url = feed_url;
        self
    }

    /// Provide an image for formats that support channel images.
    #[must_use]
    pub fn with_image(mut self, image: Image) -> Self {
        self.image = Some(image);
        self
    }

    /// Borrow the feed title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrow the feed home page URL, if present.
    pub fn home_page_url(&self) -> Option<&str> {
        self.home_page_url.as_deref()
    }

    /// Borrow the feed URL, if present.
    pub fn feed_url(&self) -> Option<&str> {
        self.feed_url.as_deref()
    }

    /// Borrow the feed channel image, if present.
    pub fn image(&self) -> Option<&Image> {
        self.image.as_ref()
    }

    /// Add an entry to the end of the feed.
    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Add an entry to the end of the feed, returning `self` for chaining.
    #[must_use]
    pub fn with_entry(mut self, entry: Entry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Borrow all entries in order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Consume the feed and return its entries.
    pub fn into_entries(self) -> Vec<Entry> {
        self.entries
    }

    /// Render the feed in the requested format.
    pub fn to(&self, format: FeedFormat) -> Result<String> {
        match format {
            FeedFormat::JsonFeed => json_feed::render(self),
            FeedFormat::Atom => atom::render(self),
            FeedFormat::Rss => rss::render(self),
            FeedFormat::HAtom => hatom::render(self),
            FeedFormat::Rss1 => rss::rss_10::render_rss_10(self),
            FeedFormat::Rss092 => rss::rss_092::render_rss_092(self),
            FeedFormat::Rss091Netscape => {
                rss::rss_091::render_rss_091_netscape(self)
            }
            FeedFormat::Rss091UserLand => {
                rss::rss_091::render_rss_091_userland(self)
            }
            FeedFormat::Rss09 => rss::rss_09::render_rss_09(self),
            FeedFormat::ScriptingNews10 => {
                rss::scripting_news::render_scripting_news_10(self)
            }
            FeedFormat::ScriptingNews20 => {
                rss::scripting_news::render_scripting_news_20(self)
            }
        }
    }

    /// Render as JSON Feed.
    pub fn to_json_feed(&self) -> Result<String> {
        self.to(FeedFormat::JsonFeed)
    }

    /// Render as Atom.
    pub fn to_atom(&self) -> Result<String> {
        self.to(FeedFormat::Atom)
    }

    /// Render as RSS.
    pub fn to_rss(&self) -> Result<String> {
        self.to(FeedFormat::Rss)
    }

    /// Render as hAtom (HTML with microformat markup).
    pub fn to_hatom(&self) -> Result<String> {
        self.to(FeedFormat::HAtom)
    }
}
