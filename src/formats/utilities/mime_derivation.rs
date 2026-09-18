/*
 * Copyright (c) Ian F. Darwin 1986-1995.
 * Software written by Ian F. Darwin and others;
 * maintained 1995-present by Christos Zoulas and others.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice immediately at the beginning of the file, without modification,
 *    this list of conditions, and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE FOR
 * ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

// SPDX-License-Identifier: AGPL-3.0-or-later AND BSD-2-Clause-Darwin AND Apache-2.0 AND MIT AND BSD-3-Clause
// SPDX-License-Identifier for parts derived from `file` (libmagic): BSD-2-Clause-Darwin
// SPDX-License-Identifier for parts derived from polyfile: Apache-2.0
// SPDX-License-Identifier for parts derived from fileid and binwalk: MIT
// SPDX-License-Identifier for parts derived from DROID: BSD-3-Clause
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

// See license text at the beginning of this file for full license details for parts derived from `file` <https://www.darwinsys.com/file/>.
// See the full license details for parts derived from polyfile <https://github.com/trailofbits/polyfile>, binwalk <https://github.com/ReFirmLabs/binwalk>, fileid <https://github.com/DBHeise/fileid>, and DROID <https://github.com/digital-preservation/droid> at the end of this file.

//! Build-time and runtime derivation of MIME type and extension mappings to
//! authoritative Document Characters (Dcs) and Format IDs.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::format_id::FormatId;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Mapping record associating an external MIME type or extension with an
/// authoritative Document Character format definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatMapping {
    /// Authoritative workspace `FormatId` if a matching variant exists.
    pub format_id: Option<FormatId>,
    /// Global graph Document Character ID (offset by 2,228,224 for format Dcs).
    pub dc_id: u128,
    /// Short format ID (column 2 in format CSVs).
    pub short_id: usize,
    /// Identifier string (Rust-friendly ident from column 3).
    pub ident: String,
    /// Human-readable label (column 4).
    pub label: String,
    /// Category string (column 5).
    pub category: String,
    /// Associated primary and alternative file extensions.
    pub extensions: Vec<String>,
    /// Primary and alternative MIME types.
    pub mime_types: Vec<String>,
    /// Associated operating systems extracted from `@os(...)` format shorthands.
    pub os_associations: Vec<FormatId>,
}

impl FormatMapping {
    /// Checks whether this format is associated with or compatible with a target OS.
    #[must_use]
    pub fn matches_os(&self, target_os: FormatId) -> bool {
        self.os_associations
            .iter()
            .any(|&cand_os| crate::detection::is_os_match(cand_os, target_os))
    }
}

/// Consolidated lookup table mapping MIME types, extensions, and idents to
/// format mappings.
#[derive(Debug, Default)]
pub struct FormatCatalog {
    by_mime: HashMap<String, FormatMapping>,
    by_ident: HashMap<String, FormatMapping>,
    by_extension: HashMap<String, Vec<FormatMapping>>,
}

impl FormatCatalog {
    /// Creates a new catalog by parsing the formats dataset.
    pub fn build() -> Self {
        let mut catalog = Self::default();

        // Populate from CSV files embedded in ctb_formats_dcdata
        for file in ctb_formats_dcdata::FORMATS_CATEGORIES_DIR.files() {
            let is_csv = file.path().extension().and_then(|ext| ext.to_str())
                == Some("csv");
            if !is_csv {
                continue;
            }

            let vec_bytes = file.contents().to_vec();
            let Ok(table) = csv_tools::parse_csv_reader(
                &vec_bytes,
                csv_tools::CsvParseOptions {
                    has_header: true,
                    flexible: true,
                    ..Default::default()
                },
            ) else {
                continue;
            };

            for i in 0..table.row_count() {
                let get_cell = |col: usize| -> String {
                    match table.cell(i, col) {
                        Some(s) => s.trim().to_string(),
                        None => String::new(),
                    }
                };

                let dc_str = get_cell(0);
                let Ok(dc_id) = dc_str.parse::<u128>() else {
                    continue;
                };

                let short_str = get_cell(1);
                let Ok(short_id) =
                    ctb_formats_dcdata::parse_format_shorthand(&short_str)
                else {
                    continue;
                };

                let ident = get_cell(2);
                let raw_label = get_cell(3);
                let label = if let Some(stripped) = raw_label.strip_prefix('!')
                {
                    stripped.trim().to_string()
                } else {
                    raw_label
                };

                let category = get_cell(4);
                let aliases_base_field = get_cell(5);
                let ext_field = get_cell(6);
                let mime_field = get_cell(7);

                let extensions: Vec<String> = ext_field
                    .split(',')
                    .map(|s| {
                        let trimmed = s.trim().trim_matches('"');
                        // Reason for fallback: extensions without a leading dot are already bare extensions
                        trimmed.strip_prefix('.').unwrap_or(trimmed).to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                let mime_types: Vec<String> = mime_field
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .trim_matches('"')
                            .to_ascii_lowercase()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut os_associations = Vec::new();
                if !aliases_base_field.is_empty() {
                    let mut report =
                        ctb_formats_dcdata::report::ValidationReport::default();
                    let parsed = ctb_formats_dcdata::parse_aliases_or_base_column(
                        &aliases_base_field,
                        "format_table.csv",
                        0,
                        &mut report,
                        true,
                    );
                    for os_shorthand in parsed.os_associations {
                        if let Some(fid) = FormatId::from_shorthand(&os_shorthand)
                        {
                            if !os_associations.contains(&fid) {
                                os_associations.push(fid);
                            }
                        }
                    }
                }

                let nicknames_field = get_cell(10);
                let nicknames: Vec<String> = nicknames_field
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .trim_matches('"')
                            .to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                let format_id = FormatId::from_ident(&ident);

                let mapping = FormatMapping {
                    format_id,
                    dc_id,
                    short_id,
                    ident: ident.clone(),
                    label,
                    category,
                    extensions: extensions.clone(),
                    mime_types: mime_types.clone(),
                    os_associations,
                };

                if !ident.is_empty() {
                    catalog
                        .by_ident
                        .insert(ident.to_ascii_lowercase(), mapping.clone());
                }

                for nick in nicknames {
                    catalog
                        .by_ident
                        .entry(nick.to_ascii_lowercase())
                        .or_insert_with(|| mapping.clone());
                }

                for mime in mime_types {
                    catalog.by_mime.entry(mime).or_insert_with(|| mapping.clone());
                }

                for ext in extensions {
                    catalog
                        .by_extension
                        .entry(ext.to_ascii_lowercase())
                        .or_default()
                        .push(mapping.clone());
                }
            }
        }

        // Ingest supplementary MIME-to-extension mappings from the embedded
        // Apache HTTPD MIME database.
        if let Some(httpd_bytes) =
            ctb_formats_dcdata::get_dc_data_file("mimes/httpd/mime.types")
        {
            if let Ok(content) = std::str::from_utf8(&httpd_bytes) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Some(mime) = parts.first() {
                            let mime_lower = mime.to_ascii_lowercase();
                            if !catalog.by_mime.contains_key(&mime_lower) {
                                for ext in parts.iter().skip(1) {
                                    if let Some(candidates) =
                                        catalog.by_extension.get(*ext)
                                    {
                                        if let Some(m) = candidates.first() {
                                            catalog
                                                .by_mime
                                                .insert(mime_lower.clone(), m.clone());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        catalog
    }

    /// Looks up a format mapping by exact MIME type.
    pub fn lookup_mime(&self, mime: &str) -> Option<&FormatMapping> {
        let normalized = mime.trim().to_ascii_lowercase();
        self.by_mime.get(&normalized)
    }

    /// Looks up a format mapping by format identifier name.
    pub fn lookup_ident(&self, ident: &str) -> Option<&FormatMapping> {
        let normalized = ident.trim().to_ascii_lowercase();
        self.by_ident.get(&normalized)
    }

    /// Looks up a format mapping by format identifier name or description prefix/tokens.
    pub fn lookup_description_or_ident(&self, text: &str) -> Option<&FormatMapping> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(m) = self.lookup_ident(trimmed) {
            return Some(m);
        }
        // Try first word (e.g. "gzip compressed data" -> "gzip")
        if let Some(first_word) = trimmed.split_whitespace().next() {
            let clean_word = first_word.trim_matches(|c: char| !c.is_alphanumeric());
            if let Some(m) = self.lookup_ident(clean_word) {
                return Some(m);
            }
        }
        None
    }

    /// Looks up candidate format mappings for an extension.
    pub fn lookup_extension(&self, ext: &str) -> &[FormatMapping] {
        let trimmed = ext.trim().trim_start_matches('.');
        let normalized = trimmed.to_ascii_lowercase();
        match self.by_extension.get(&normalized) {
            Some(list) => list.as_slice(),
            None => &[],
        }
    }
}

/// Global shared format catalog instance.
pub static FORMAT_CATALOG: LazyLock<FormatCatalog> =
    LazyLock::new(FormatCatalog::build);
/*

Text of LICENSE from DROID:

Copyright (c) 2016, The National Archives <pronom@nationalarchives.gov.uk>
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following
conditions are met:

 * Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.

 * Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.

 * Neither the name of the The National Archives nor the
   names of its contributors may be used to endorse or promote products
   derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

/*

Text of LICENSE from fileid:

The MIT License (MIT)

Copyright (c) 2015 David B Heise

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


*/

/*
Text of LICENSE from binwalk:


MIT License

Copyright (c) 2024 devttys0

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

/*
Text of LICENSE from polyfile:


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