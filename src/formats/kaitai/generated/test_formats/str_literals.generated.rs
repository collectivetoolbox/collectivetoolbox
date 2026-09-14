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
pub struct StrLiterals {
    pub(crate) _root: SharedType<StrLiterals>,
    pub(crate) _parent: SharedType<StrLiterals>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_backslashes: Cell<bool>,
    backslashes: RefCell<String>,
    f_complex_str: Cell<bool>,
    complex_str: RefCell<String>,
    f_double_quotes: Cell<bool>,
    double_quotes: RefCell<String>,
    f_octal_eatup: Cell<bool>,
    octal_eatup: RefCell<String>,
    f_octal_eatup2: Cell<bool>,
    octal_eatup2: RefCell<String>,
}
impl KStruct for StrLiterals {
    type Root = StrLiterals;
    type Parent = StrLiterals;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrLiterals {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn backslashes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_backslashes.get() {
            return Ok(self.backslashes.borrow());
        }
        self.f_backslashes.set(true);
        *self.backslashes.borrow_mut() = "\\\\134".to_string();
        Ok(self.backslashes.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn complex_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_complex_str.get() {
            return Ok(self.complex_str.borrow());
        }
        self.f_complex_str.set(true);
        *self.complex_str.borrow_mut() = "\012\u{7}\u{8}\n\r\t\u{b}\u{c}e757\n$☻".to_string();
        Ok(self.complex_str.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn double_quotes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_double_quotes.get() {
            return Ok(self.double_quotes.borrow());
        }
        self.f_double_quotes.set(true);
        *self.double_quotes.borrow_mut() = "\"\"42".to_string();
        Ok(self.double_quotes.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn octal_eatup(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_octal_eatup.get() {
            return Ok(self.octal_eatup.borrow());
        }
        self.f_octal_eatup.set(true);
        *self.octal_eatup.borrow_mut() = "\06262".to_string();
        Ok(self.octal_eatup.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn octal_eatup2(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_octal_eatup2.get() {
            return Ok(self.octal_eatup2.borrow());
        }
        self.f_octal_eatup2.set(true);
        *self.octal_eatup2.borrow_mut() = "262".to_string();
        Ok(self.octal_eatup2.borrow())
    }
}
impl StrLiterals {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
