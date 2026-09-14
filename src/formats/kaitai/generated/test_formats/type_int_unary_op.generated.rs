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
pub struct TypeIntUnaryOp {
    pub(crate) _root: SharedType<TypeIntUnaryOp>,
    pub(crate) _parent: SharedType<TypeIntUnaryOp>,
    pub(crate) _self_shared: SharedType<Self>,
    value_s2: RefCell<i16>,
    value_s8: RefCell<i64>,
    _io: RefCell<BytesReader>,
    f_unary_s2: Cell<bool>,
    unary_s2: RefCell<i16>,
    f_unary_s8: Cell<bool>,
    unary_s8: RefCell<i64>,
}
impl KStruct for TypeIntUnaryOp {
    type Root = TypeIntUnaryOp;
    type Parent = TypeIntUnaryOp;

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
        *self_rc.value_s2.borrow_mut() = _io.read_s2le()?;
        *self_rc.value_s8.borrow_mut() = _io.read_s8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TypeIntUnaryOp {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn unary_s2(
        &self
    ) -> KResult<Ref<'_, i16>> {
        let _io = self._io.borrow();
        if self.f_unary_s2.get() {
            return Ok(self.unary_s2.borrow());
        }
        self.f_unary_s2.set(true);
        *self.unary_s2.borrow_mut() = ((0_i32).saturating_sub(to_i32(*self.value_s2()))).try_into()?;
        Ok(self.unary_s2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn unary_s8(
        &self
    ) -> KResult<Ref<'_, i64>> {
        let _io = self._io.borrow();
        if self.f_unary_s8.get() {
            return Ok(self.unary_s8.borrow());
        }
        self.f_unary_s8.set(true);
        *self.unary_s8.borrow_mut() = ((0_i64).saturating_sub(*self.value_s8())).try_into()?;
        Ok(self.unary_s8.borrow())
    }
}
impl TypeIntUnaryOp {
    pub fn value_s2(&self) -> Ref<'_, i16> {
        self.value_s2.borrow()
    }
}
impl TypeIntUnaryOp {
    pub fn value_s8(&self) -> Ref<'_, i64> {
        self.value_s8.borrow()
    }
}
impl TypeIntUnaryOp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
