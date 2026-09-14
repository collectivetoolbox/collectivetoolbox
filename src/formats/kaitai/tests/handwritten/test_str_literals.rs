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
use rust::formats::str_literals::*;

#[crate::ctb_test]
fn test_str_literals() -> KResult<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(manifest_dir.join("kaitai_struct_tests/src/fixed_struct.bin"))?;
    let _io = BytesReader::from(bytes);
    let r: OptRc<StrLiterals> = StrLiterals::read_into(&_io, None, None)?;

    assert_eq!(*r.complex_str()?, "\u{0}\u{1}\u{2}\u{7}\u{8}\u{a}\u{d}\u{9}\u{b}\u{c}\u{1b}\u{3d}\u{7}\u{a}\u{24}\u{263b}");
    assert_eq!(*r.double_quotes()?, "\u{22}\u{22}\u{22}");
    assert_eq!(*r.backslashes()?, "\u{5c}\u{5c}\u{5c}");
    assert_eq!(*r.octal_eatup()?, "\u{0}\u{32}\u{32}");
    assert_eq!(*r.octal_eatup2()?, "\u{2}\u{32}");
    Ok(())
}
