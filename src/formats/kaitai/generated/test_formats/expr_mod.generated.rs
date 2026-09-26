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
pub struct ExprMod {
    pub(crate) _root: SharedType<ExprMod>,
    pub(crate) _parent: SharedType<ExprMod>,
    pub(crate) _self_shared: SharedType<Self>,
    int_u: RefCell<u32>,
    int_s: RefCell<i32>,
    _io: RefCell<BytesReader>,
    f_mod_neg_const: Cell<bool>,
    mod_neg_const: RefCell<i32>,
    f_mod_neg_seq: Cell<bool>,
    mod_neg_seq: RefCell<i32>,
    f_mod_pos_const: Cell<bool>,
    mod_pos_const: RefCell<i32>,
    f_mod_pos_seq: Cell<bool>,
    mod_pos_seq: RefCell<i32>,
}
impl KStruct for ExprMod {
    type Root = ExprMod;
    type Parent = ExprMod;

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
        *self_rc.int_u.borrow_mut() = _io.read_u4le()?;
        *self_rc.int_s.borrow_mut() = _io.read_s4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprMod {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn mod_neg_const(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_mod_neg_const.get() {
            return Ok(self.mod_neg_const.borrow());
        }
        self.f_mod_neg_const.set(true);
        *self.mod_neg_const.borrow_mut() = (modulo(i64::from((0_i32).saturating_sub(to_i32(9837))), 13_i64)).try_into()?;
        Ok(self.mod_neg_const.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn mod_neg_seq(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_mod_neg_seq.get() {
            return Ok(self.mod_neg_seq.borrow());
        }
        self.f_mod_neg_seq.set(true);
        *self.mod_neg_seq.borrow_mut() = (modulo(i64::from(*self.int_s()), 13_i64)).try_into()?;
        Ok(self.mod_neg_seq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn mod_pos_const(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_mod_pos_const.get() {
            return Ok(self.mod_pos_const.borrow());
        }
        self.f_mod_pos_const.set(true);
        *self.mod_pos_const.borrow_mut() = (modulo(9837_i64, 13_i64)).try_into()?;
        Ok(self.mod_pos_const.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn mod_pos_seq(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_mod_pos_seq.get() {
            return Ok(self.mod_pos_seq.borrow());
        }
        self.f_mod_pos_seq.set(true);
        *self.mod_pos_seq.borrow_mut() = ((*self.int_u()).checked_rem(13_u32).ok_or(KError::CastError)?).try_into()?;
        Ok(self.mod_pos_seq.borrow())
    }
}
impl ExprMod {
    pub fn int_u(&self) -> Ref<'_, u32> {
        self.int_u.borrow()
    }
}
impl ExprMod {
    pub fn int_s(&self) -> Ref<'_, i32> {
        self.int_s.borrow()
    }
}
impl ExprMod {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
