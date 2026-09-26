// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

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
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct TermStrzUtf16V1 {
    pub(crate) _root: SharedType<TermStrzUtf16V1>,
    pub(crate) _parent: SharedType<TermStrzUtf16V1>,
    pub(crate) _self_shared: SharedType<Self>,
    s1: RefCell<String>,
    s2: RefCell<String>,
    term: RefCell<u16>,
    s3: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for TermStrzUtf16V1 {
    type Root = TermStrzUtf16V1;
    type Parent = TermStrzUtf16V1;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
    fn read<S: KStream>(
        self_rc: &OptRc<Self>,
        io: &S,
        root: SharedType<Self::Root>,
        parent: SharedType<Self::Parent>,
    ) -> KResult<()> {
        *self_rc._io.borrow_mut() = io.clone();
        self_rc._root.set(root.get());
        self_rc._parent.set(parent.get());
        self_rc._self_shared.set(Ok(self_rc.clone()));
        let _io = io;
        *self_rc.s1.borrow_mut() = bytes_to_str(&_io.read_bytes_term_multi(&[0, 0], false, true, true)?, "UTF-16LE")?;
        *self_rc.s2.borrow_mut() = bytes_to_str(&_io.read_bytes_term_multi(&[0, 0], false, false, true)?, "UTF-16LE")?;
        *self_rc.term.borrow_mut() = _io.read_u2le()?;
        *self_rc.s3.borrow_mut() = bytes_to_str(&_io.read_bytes_term_multi(&[0, 0], true, true, true)?, "UTF-16LE")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TermStrzUtf16V1 {
}
impl TermStrzUtf16V1 {
    pub fn s1(&self) -> Ref<'_, String> {
        self.s1.borrow()
    }
}
impl TermStrzUtf16V1 {
    pub fn s2(&self) -> Ref<'_, String> {
        self.s2.borrow()
    }
}
impl TermStrzUtf16V1 {
    pub fn term(&self) -> Ref<'_, u16> {
        self.term.borrow()
    }
}
impl TermStrzUtf16V1 {
    pub fn s3(&self) -> Ref<'_, String> {
        self.s3.borrow()
    }
}
impl TermStrzUtf16V1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
