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
pub struct IntegersDoubleOverflow {
    pub(crate) _root: SharedType<IntegersDoubleOverflow>,
    pub(crate) _parent: SharedType<IntegersDoubleOverflow>,
    pub(crate) _self_shared: SharedType<Self>,
    signed_safe_min_be: RefCell<i64>,
    signed_safe_min_le: RefCell<i64>,
    signed_safe_max_be: RefCell<i64>,
    signed_safe_max_le: RefCell<i64>,
    signed_unsafe_neg_be: RefCell<i64>,
    signed_unsafe_neg_le: RefCell<i64>,
    signed_unsafe_pos_be: RefCell<i64>,
    signed_unsafe_pos_le: RefCell<i64>,
    _io: RefCell<BytesReader>,
    f_unsigned_safe_max_be: Cell<bool>,
    unsigned_safe_max_be: RefCell<u64>,
    f_unsigned_safe_max_le: Cell<bool>,
    unsigned_safe_max_le: RefCell<u64>,
    f_unsigned_unsafe_pos_be: Cell<bool>,
    unsigned_unsafe_pos_be: RefCell<u64>,
    f_unsigned_unsafe_pos_le: Cell<bool>,
    unsigned_unsafe_pos_le: RefCell<u64>,
}
impl KStruct for IntegersDoubleOverflow {
    type Root = IntegersDoubleOverflow;
    type Parent = IntegersDoubleOverflow;

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
        *self_rc.signed_safe_min_be.borrow_mut() = _io.read_s8be()?;
        *self_rc.signed_safe_min_le.borrow_mut() = _io.read_s8le()?;
        *self_rc.signed_safe_max_be.borrow_mut() = _io.read_s8be()?;
        *self_rc.signed_safe_max_le.borrow_mut() = _io.read_s8le()?;
        *self_rc.signed_unsafe_neg_be.borrow_mut() = _io.read_s8be()?;
        *self_rc.signed_unsafe_neg_le.borrow_mut() = _io.read_s8le()?;
        *self_rc.signed_unsafe_pos_be.borrow_mut() = _io.read_s8be()?;
        *self_rc.signed_unsafe_pos_le.borrow_mut() = _io.read_s8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IntegersDoubleOverflow {
    pub fn unsigned_safe_max_be(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_unsigned_safe_max_be.get() {
            return Ok(self.unsigned_safe_max_be.borrow());
        }
        self.f_unsigned_safe_max_be.set(true);
        let _pos = _io.pos();
        _io.seek(16_usize)?;
        *self.unsigned_safe_max_be.borrow_mut() = _io.read_u8be()?;
        _io.seek(_pos)?;
        Ok(self.unsigned_safe_max_be.borrow())
    }
    pub fn unsigned_safe_max_le(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_unsigned_safe_max_le.get() {
            return Ok(self.unsigned_safe_max_le.borrow());
        }
        self.f_unsigned_safe_max_le.set(true);
        let _pos = _io.pos();
        _io.seek(24_usize)?;
        *self.unsigned_safe_max_le.borrow_mut() = _io.read_u8le()?;
        _io.seek(_pos)?;
        Ok(self.unsigned_safe_max_le.borrow())
    }
    pub fn unsigned_unsafe_pos_be(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_unsigned_unsafe_pos_be.get() {
            return Ok(self.unsigned_unsafe_pos_be.borrow());
        }
        self.f_unsigned_unsafe_pos_be.set(true);
        let _pos = _io.pos();
        _io.seek(48_usize)?;
        *self.unsigned_unsafe_pos_be.borrow_mut() = _io.read_u8be()?;
        _io.seek(_pos)?;
        Ok(self.unsigned_unsafe_pos_be.borrow())
    }
    pub fn unsigned_unsafe_pos_le(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_unsigned_unsafe_pos_le.get() {
            return Ok(self.unsigned_unsafe_pos_le.borrow());
        }
        self.f_unsigned_unsafe_pos_le.set(true);
        let _pos = _io.pos();
        _io.seek(56_usize)?;
        *self.unsigned_unsafe_pos_le.borrow_mut() = _io.read_u8le()?;
        _io.seek(_pos)?;
        Ok(self.unsigned_unsafe_pos_le.borrow())
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_safe_min_be(&self) -> Ref<'_, i64> {
        self.signed_safe_min_be.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_safe_min_le(&self) -> Ref<'_, i64> {
        self.signed_safe_min_le.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_safe_max_be(&self) -> Ref<'_, i64> {
        self.signed_safe_max_be.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_safe_max_le(&self) -> Ref<'_, i64> {
        self.signed_safe_max_le.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_unsafe_neg_be(&self) -> Ref<'_, i64> {
        self.signed_unsafe_neg_be.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_unsafe_neg_le(&self) -> Ref<'_, i64> {
        self.signed_unsafe_neg_le.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_unsafe_pos_be(&self) -> Ref<'_, i64> {
        self.signed_unsafe_pos_be.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn signed_unsafe_pos_le(&self) -> Ref<'_, i64> {
        self.signed_unsafe_pos_le.borrow()
    }
}
impl IntegersDoubleOverflow {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
