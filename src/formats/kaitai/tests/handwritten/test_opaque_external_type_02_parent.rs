// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL

/*
== This file is derived from kaitai_struct_tests. License, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

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

use std::path::PathBuf;
use kaitai::*;
use rust::formats::opaque_external_type_02_parent::*;

#[crate::ctb_test]
fn test_opaque_external_type_02_parent() -> KResult<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(manifest_dir.join("kaitai_struct_tests/src/term_strz.bin"))?;
    let _io = BytesReader::from(bytes);
    let r: OptRc<OpaqueExternalType02Parent> = OpaqueExternalType02Parent::read_into(&_io, None, None)?;

    assert_eq!(*r.parent().child().s1(), "foo");
    assert_eq!(*r.parent().child().s2(), "bar");
    assert_eq!(*r.parent().child().s3().s3(), "|baz@");
    Ok(())
}
