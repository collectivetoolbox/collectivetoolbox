// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from kaitai_struct_tests: MIT
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

// Parts derived from kaitai_struct_tests (see full license information at the end of this file):
// Copyright (c) 2019 Kaitai Project

//! Data structures representing Kaitai Struct Test (`.kst`) specifications.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A parsed Kaitai Struct Test (`.kst`) specification file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KstSpec {
    /// Format identifier matching the format definition under test.
    pub id: String,

    /// Name of the binary fixture file to test against.
    pub data: Option<String>,

    /// Expected exception, if this test expects parsing to fail.
    pub exception: Option<KstException>,

    /// List of assertions to perform on the parsed structure.
    #[serde(default)]
    pub asserts: Vec<KstAssert>,

    /// Additional imports specified by the test.
    #[serde(default)]
    pub imports: Vec<String>,
}

/// Representation of an expected failure or exception in `.kst`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KstException {
    /// Simple exception name string (e.g. `ValidationNotEqualError<u1>`).
    Simple(String),

    /// Detailed exception specification with type and optional message.
    Detailed {
        r#type: String,
        message: Option<String>,
    },
}

/// An assertion item in `.kst`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KstAssert {
    /// The actual expression to evaluate on the parsed data object.
    pub actual: serde_yaml::Value,

    /// The expected value, if asserting equality.
    #[serde(default)]
    pub expected: Option<serde_yaml::Value>,

    /// Expected exception on evaluating actual, if expecting an error.
    #[serde(default)]
    pub exception: Option<String>,
}

/// Parses a `.kst` YAML string into a `KstSpec`.
///
/// # Errors
/// Returns an error if the YAML content does not conform to `KstSpec`.
pub fn parse_kst_str(src: &str) -> Result<KstSpec> {
    let spec: KstSpec = serde_yaml::from_str(src)
        .context("Failed to parse .kst YAML specification")?;
    Ok(spec)
}

/// Parses a `.kst` file from disk.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn parse_kst_file(path: &Path) -> Result<KstSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read .kst file at {}", path.display()))?;
    parse_kst_str(&content)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_parse_sample_kst() -> Result<()> {
        let yaml = r#"
id: bcd_user_type_be
data: bcd_user_type_be.bin
asserts:
  - actual: ltr.as_int
    expected: 12345678
  - actual: ltr.as_str
    expected: '"12345678"'
"#;
        let spec = parse_kst_str(yaml)?;
        assert_eq!(spec.id, "bcd_user_type_be");
        assert_eq!(spec.data.as_deref(), Some("bcd_user_type_be.bin"));
        assert_eq!(spec.asserts.len(), 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_all_kst_specs() -> Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let ks_dir = manifest_dir.join("kaitai_struct_tests/spec/ks");
        if !ks_dir.exists() {
            return Ok(());
        }

        let mut count = 0usize;
        for entry in std::fs::read_dir(ks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "kst") {
                let spec = parse_kst_file(&path)
                    .with_context(|| format!("Failed to parse {}", path.display()))?;
                assert!(!spec.id.is_empty(), "KstSpec id should not be empty in {}", path.display());
                count = count.saturating_add(1);
            }
        }
        assert!(count >= 250, "Expected at least 250 .kst files, found {count}");
        Ok(())
    }
}

/*
== License information for parts derived from kaitai_struct_tests, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

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