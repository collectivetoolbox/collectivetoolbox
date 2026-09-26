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
pub struct Expr1 {
    pub(crate) _root: SharedType<Expr1>,
    pub(crate) _parent: SharedType<Expr1>,
    pub(crate) _self_shared: SharedType<Self>,
    len_of_1: RefCell<u16>,
    str1: RefCell<String>,
    _io: RefCell<BytesReader>,
    str1_raw: RefCell<Vec<u8>>,
    f_len_of_1_mod: Cell<bool>,
    len_of_1_mod: RefCell<i32>,
    f_str1_len: Cell<bool>,
    str1_len: RefCell<i32>,
}
impl KStruct for Expr1 {
    type Root = Expr1;
    type Parent = Expr1;

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
        *self_rc.len_of_1.borrow_mut() = _io.read_u2le()?;
        *self_rc.str1.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.len_of_1_mod()?)?)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Expr1 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn len_of_1_mod(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_of_1_mod.get() {
            return Ok(self.len_of_1_mod.borrow());
        }
        self.f_len_of_1_mod.set(true);
        *self.len_of_1_mod.borrow_mut() = ((i32::from(*self.len_of_1())).saturating_sub(2_i32)).try_into()?;
        Ok(self.len_of_1_mod.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str1_len.get() {
            return Ok(self.str1_len.borrow());
        }
        self.f_str1_len.set(true);
        *self.str1_len.borrow_mut() = (self.str1().len()).try_into()?;
        Ok(self.str1_len.borrow())
    }
}
impl Expr1 {
    pub fn len_of_1(&self) -> Ref<'_, u16> {
        self.len_of_1.borrow()
    }
}
impl Expr1 {
    pub fn str1(&self) -> Ref<'_, String> {
        self.str1.borrow()
    }
}
impl Expr1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Expr1 {
    pub fn str1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str1_raw.borrow()
    }
}
