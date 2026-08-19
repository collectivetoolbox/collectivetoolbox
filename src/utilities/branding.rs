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

//! Functions for providing branding details. Whether to use branded or generic branding is determined by the --branding/--no-branding flags in the build script.

pub const fn is_branded_build() -> bool {
    match option_env!("CTB_BRANDING") {
        Some(val) => crate::string::const_str_eq(val, "true"),
        None => false,
    }
}

pub const fn official_domain() -> &'static str {
    "CollectiveToolbox.com"
}

pub const fn official_url() -> &'static str {
    constcat::concat!("https://", official_domain())
}

pub const fn official_email() -> &'static str {
    constcat::concat!("info@", official_domain())
}

pub const fn official_application_name() -> &'static str {
    "Collective Toolbox"
}

pub const fn default_domain() -> &'static str {
    if is_branded_build() {
        official_domain()
    } else {
        "example.com"
    }
}

pub const fn default_email() -> &'static str {
    constcat::concat!("info@", default_domain())
}

pub const fn default_url() -> &'static str {
    constcat::concat!("https://", default_domain())
}

pub const fn newsletter_url() -> &'static str {
    if is_branded_build() {
        "https://collectivetoolbox.eo.page/"
    } else {
        "https://example.com/"
    }
}

pub fn application_name() -> &'static str {
    if is_branded_build() {
        official_application_name()
    } else {
        "CTUnofficialBuild"
    }
}

pub fn user_agent_name() -> String {
    let string = application_name().replace(' ', "");
    if crate::environment::is_official_signed_build() {
        string
    } else if is_branded_build() {
        format!("{string}-UnofficialBuild")
    } else {
        string
    }
}

pub fn replace_magic_strings(markdown: &str) -> String {
    markdown
        .replace("$SiteName", application_name())
        .replace("$DomainName", default_domain())
        .replace("$SiteURL", default_url())
        .replace("$ContactEmail", default_email())
}
