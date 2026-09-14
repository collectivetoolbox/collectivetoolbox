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
pub struct ExprBytesCmp {
    pub(crate) _root: SharedType<ExprBytesCmp>,
    pub(crate) _parent: SharedType<ExprBytesCmp>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<Vec<u8>>,
    two: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_ack: Cell<bool>,
    ack: RefCell<Vec<i32>>,
    f_ack2: Cell<bool>,
    ack2: RefCell<Vec<i32>>,
    f_hi_val: Cell<bool>,
    hi_val: RefCell<Vec<i32>>,
    f_is_eq: Cell<bool>,
    is_eq: RefCell<bool>,
    f_is_ge: Cell<bool>,
    is_ge: RefCell<bool>,
    f_is_gt: Cell<bool>,
    is_gt: RefCell<bool>,
    f_is_gt2: Cell<bool>,
    is_gt2: RefCell<bool>,
    f_is_le: Cell<bool>,
    is_le: RefCell<bool>,
    f_is_lt: Cell<bool>,
    is_lt: RefCell<bool>,
    f_is_lt2: Cell<bool>,
    is_lt2: RefCell<bool>,
    f_is_ne: Cell<bool>,
    is_ne: RefCell<bool>,
}
impl KStruct for ExprBytesCmp {
    type Root = ExprBytesCmp;
    type Parent = ExprBytesCmp;

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
        *self_rc.one.borrow_mut() = _io.read_bytes(1_usize)?;
        *self_rc.two.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprBytesCmp {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ack(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_ack.get() {
            return Ok(self.ack.borrow());
        }
        self.f_ack.set(true);
        *self.ack.borrow_mut() = vec![65_i32, 67_i32, 75_i32];
        Ok(self.ack.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ack2(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_ack2.get() {
            return Ok(self.ack2.borrow());
        }
        self.f_ack2.set(true);
        *self.ack2.borrow_mut() = vec![65_i32, 67_i32, 75_i32, 50_i32];
        Ok(self.ack2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn hi_val(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_hi_val.get() {
            return Ok(self.hi_val.borrow());
        }
        self.f_hi_val.set(true);
        *self.hi_val.borrow_mut() = vec![144_i32, 67_i32];
        Ok(self.hi_val.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_eq.get() {
            return Ok(self.is_eq.borrow());
        }
        self.f_is_eq.set(true);
        *self.is_eq.borrow_mut() = (*self.two() == *self.ack()?).try_into()?;
        Ok(self.is_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_ge(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_ge.get() {
            return Ok(self.is_ge.borrow());
        }
        self.f_is_ge.set(true);
        *self.is_ge.borrow_mut() = (*self.two() >= *self.ack2()?).try_into()?;
        Ok(self.is_ge.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_gt(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_gt.get() {
            return Ok(self.is_gt.borrow());
        }
        self.f_is_gt.set(true);
        *self.is_gt.borrow_mut() = (*self.two() > *self.ack2()?).try_into()?;
        Ok(self.is_gt.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_gt2(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_gt2.get() {
            return Ok(self.is_gt2.borrow());
        }
        self.f_is_gt2.set(true);
        *self.is_gt2.borrow_mut() = (*self.hi_val()? > *self.two()).try_into()?;
        Ok(self.is_gt2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_le(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_le.get() {
            return Ok(self.is_le.borrow());
        }
        self.f_is_le.set(true);
        *self.is_le.borrow_mut() = (*self.two() <= *self.ack2()?).try_into()?;
        Ok(self.is_le.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_lt(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_lt.get() {
            return Ok(self.is_lt.borrow());
        }
        self.f_is_lt.set(true);
        *self.is_lt.borrow_mut() = (*self.two() < *self.ack2()?).try_into()?;
        Ok(self.is_lt.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_lt2(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_lt2.get() {
            return Ok(self.is_lt2.borrow());
        }
        self.f_is_lt2.set(true);
        *self.is_lt2.borrow_mut() = (*self.one() < *self.two()).try_into()?;
        Ok(self.is_lt2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_ne(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_ne.get() {
            return Ok(self.is_ne.borrow());
        }
        self.f_is_ne.set(true);
        *self.is_ne.borrow_mut() = (*self.two() != *self.ack()?).try_into()?;
        Ok(self.is_ne.borrow())
    }
}
impl ExprBytesCmp {
    pub fn one(&self) -> Ref<'_, Vec<u8>> {
        self.one.borrow()
    }
}
impl ExprBytesCmp {
    pub fn two(&self) -> Ref<'_, Vec<u8>> {
        self.two.borrow()
    }
}
impl ExprBytesCmp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
