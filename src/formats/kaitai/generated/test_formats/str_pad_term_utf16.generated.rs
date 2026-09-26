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
pub struct StrPadTermUtf16 {
    pub(crate) _root: SharedType<StrPadTermUtf16>,
    pub(crate) _parent: SharedType<StrPadTermUtf16>,
    pub(crate) _self_shared: SharedType<Self>,
    str_term: RefCell<String>,
    str_term_include: RefCell<String>,
    str_term_and_pad: RefCell<String>,
    _io: RefCell<BytesReader>,
    str_term_raw: RefCell<Vec<u8>>,
    str_term_include_raw: RefCell<Vec<u8>>,
    str_term_and_pad_raw: RefCell<Vec<u8>>,
}
impl KStruct for StrPadTermUtf16 {
    type Root = StrPadTermUtf16;
    type Parent = StrPadTermUtf16;

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
        *self_rc.str_term.borrow_mut() = bytes_to_str(&bytes_terminate_pad_multi(&_io.read_bytes(10_usize)?, Some(&[0, 0][..]), false, None), "UTF-16LE")?;
        *self_rc.str_term_include.borrow_mut() = bytes_to_str(&bytes_terminate_pad_multi(&_io.read_bytes(10_usize)?, Some(&[0, 0][..]), true, None), "UTF-16LE")?;
        *self_rc.str_term_and_pad.borrow_mut() = bytes_to_str(&bytes_terminate_pad_multi(&_io.read_bytes(9_usize)?, Some(&[0, 0][..]), false, Some(43)), "UTF-16LE")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrPadTermUtf16 {
}
impl StrPadTermUtf16 {
    pub fn str_term(&self) -> Ref<'_, String> {
        self.str_term.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn str_term_include(&self) -> Ref<'_, String> {
        self.str_term_include.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn str_term_and_pad(&self) -> Ref<'_, String> {
        self.str_term_and_pad.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn str_term_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_raw.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn str_term_include_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_include_raw.borrow()
    }
}
impl StrPadTermUtf16 {
    pub fn str_term_and_pad_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_and_pad_raw.borrow()
    }
}
