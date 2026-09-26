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
pub struct ExprFstring0 {
    pub(crate) _root: SharedType<ExprFstring0>,
    pub(crate) _parent: SharedType<ExprFstring0>,
    pub(crate) _self_shared: SharedType<Self>,
    seq_str: RefCell<String>,
    seq_int: RefCell<u8>,
    _io: RefCell<BytesReader>,
    seq_str_raw: RefCell<Vec<u8>>,
    f_empty: Cell<bool>,
    empty: RefCell<String>,
    f_head_and_int: Cell<bool>,
    head_and_int: RefCell<String>,
    f_head_and_int_literal: Cell<bool>,
    head_and_int_literal: RefCell<String>,
    f_head_and_str: Cell<bool>,
    head_and_str: RefCell<String>,
    f_head_and_str_literal: Cell<bool>,
    head_and_str_literal: RefCell<String>,
    f_literal: Cell<bool>,
    literal: RefCell<String>,
    f_literal_with_escapes: Cell<bool>,
    literal_with_escapes: RefCell<String>,
}
impl KStruct for ExprFstring0 {
    type Root = ExprFstring0;
    type Parent = ExprFstring0;

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
        *self_rc.seq_str.borrow_mut() = bytes_to_str(&_io.read_bytes(5_usize)?, "ASCII")?;
        *self_rc.seq_int.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprFstring0 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn empty(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_empty.get() {
            return Ok(self.empty.borrow());
        }
        self.f_empty.set(true);
        *self.empty.borrow_mut() = "".to_string();
        Ok(self.empty.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn head_and_int(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_head_and_int.get() {
            return Ok(self.head_and_int.borrow());
        }
        self.f_head_and_int.set(true);
        *self.head_and_int.borrow_mut() = format!("{}{}", "abc=", *self.seq_int()).to_string();
        Ok(self.head_and_int.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn head_and_int_literal(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_head_and_int_literal.get() {
            return Ok(self.head_and_int_literal.borrow());
        }
        self.f_head_and_int_literal.set(true);
        *self.head_and_int_literal.borrow_mut() = format!("{}{}", "abc=", 123).to_string();
        Ok(self.head_and_int_literal.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn head_and_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_head_and_str.get() {
            return Ok(self.head_and_str.borrow());
        }
        self.f_head_and_str.set(true);
        *self.head_and_str.borrow_mut() = format!("{}{}", "abc=", self.seq_str()).to_string();
        Ok(self.head_and_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn head_and_str_literal(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_head_and_str_literal.get() {
            return Ok(self.head_and_str_literal.borrow());
        }
        self.f_head_and_str_literal.set(true);
        *self.head_and_str_literal.borrow_mut() = format!("{}{}", "abc=", "foo").to_string();
        Ok(self.head_and_str_literal.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn literal(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_literal.get() {
            return Ok(self.literal.borrow());
        }
        self.f_literal.set(true);
        *self.literal.borrow_mut() = "abc".to_string();
        Ok(self.literal.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn literal_with_escapes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_literal_with_escapes.get() {
            return Ok(self.literal_with_escapes.borrow());
        }
        self.f_literal_with_escapes.set(true);
        *self.literal_with_escapes.borrow_mut() = "abc\n\tt".to_string();
        Ok(self.literal_with_escapes.borrow())
    }
}
impl ExprFstring0 {
    pub fn seq_str(&self) -> Ref<'_, String> {
        self.seq_str.borrow()
    }
}
impl ExprFstring0 {
    pub fn seq_int(&self) -> Ref<'_, u8> {
        self.seq_int.borrow()
    }
}
impl ExprFstring0 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprFstring0 {
    pub fn seq_str_raw(&self) -> Ref<'_, Vec<u8>> {
        self.seq_str_raw.borrow()
    }
}
