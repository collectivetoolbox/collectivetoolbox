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
pub struct IntegersMinMax {
    pub(crate) _root: SharedType<IntegersMinMax>,
    pub(crate) _parent: SharedType<IntegersMinMax>,
    pub(crate) _self_shared: SharedType<Self>,
    unsigned_min: RefCell<OptRc<IntegersMinMax_Unsigned>>,
    unsigned_max: RefCell<OptRc<IntegersMinMax_Unsigned>>,
    signed_min: RefCell<OptRc<IntegersMinMax_Signed>>,
    signed_max: RefCell<OptRc<IntegersMinMax_Signed>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IntegersMinMax {
    type Root = IntegersMinMax;
    type Parent = IntegersMinMax;

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
        let t = Self::read_into::<_, IntegersMinMax_Unsigned>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.unsigned_min.borrow_mut() = t;
        let t = Self::read_into::<_, IntegersMinMax_Unsigned>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.unsigned_max.borrow_mut() = t;
        let t = Self::read_into::<_, IntegersMinMax_Signed>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.signed_min.borrow_mut() = t;
        let t = Self::read_into::<_, IntegersMinMax_Signed>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.signed_max.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IntegersMinMax {
}
impl IntegersMinMax {
    pub fn unsigned_min(&self) -> Ref<'_, OptRc<IntegersMinMax_Unsigned>> {
        self.unsigned_min.borrow()
    }
}
impl IntegersMinMax {
    pub fn unsigned_max(&self) -> Ref<'_, OptRc<IntegersMinMax_Unsigned>> {
        self.unsigned_max.borrow()
    }
}
impl IntegersMinMax {
    pub fn signed_min(&self) -> Ref<'_, OptRc<IntegersMinMax_Signed>> {
        self.signed_min.borrow()
    }
}
impl IntegersMinMax {
    pub fn signed_max(&self) -> Ref<'_, OptRc<IntegersMinMax_Signed>> {
        self.signed_max.borrow()
    }
}
impl IntegersMinMax {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IntegersMinMax_Signed {
    pub(crate) _root: SharedType<IntegersMinMax>,
    pub(crate) _parent: SharedType<IntegersMinMax>,
    pub(crate) _self_shared: SharedType<Self>,
    s1: RefCell<i8>,
    s2le: RefCell<i16>,
    s4le: RefCell<i32>,
    s8le: RefCell<i64>,
    s2be: RefCell<i16>,
    s4be: RefCell<i32>,
    s8be: RefCell<i64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IntegersMinMax_Signed {
    type Root = IntegersMinMax;
    type Parent = IntegersMinMax;

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
        *self_rc.s1.borrow_mut() = _io.read_s1()?;
        *self_rc.s2le.borrow_mut() = _io.read_s2le()?;
        *self_rc.s4le.borrow_mut() = _io.read_s4le()?;
        *self_rc.s8le.borrow_mut() = _io.read_s8le()?;
        *self_rc.s2be.borrow_mut() = _io.read_s2be()?;
        *self_rc.s4be.borrow_mut() = _io.read_s4be()?;
        *self_rc.s8be.borrow_mut() = _io.read_s8be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IntegersMinMax_Signed {
}
impl IntegersMinMax_Signed {
    pub fn s1(&self) -> Ref<'_, i8> {
        self.s1.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s2le(&self) -> Ref<'_, i16> {
        self.s2le.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s4le(&self) -> Ref<'_, i32> {
        self.s4le.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s8le(&self) -> Ref<'_, i64> {
        self.s8le.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s2be(&self) -> Ref<'_, i16> {
        self.s2be.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s4be(&self) -> Ref<'_, i32> {
        self.s4be.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn s8be(&self) -> Ref<'_, i64> {
        self.s8be.borrow()
    }
}
impl IntegersMinMax_Signed {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct IntegersMinMax_Unsigned {
    pub(crate) _root: SharedType<IntegersMinMax>,
    pub(crate) _parent: SharedType<IntegersMinMax>,
    pub(crate) _self_shared: SharedType<Self>,
    u1: RefCell<u8>,
    u2le: RefCell<u16>,
    u4le: RefCell<u32>,
    u8le: RefCell<u64>,
    u2be: RefCell<u16>,
    u4be: RefCell<u32>,
    u8be: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for IntegersMinMax_Unsigned {
    type Root = IntegersMinMax;
    type Parent = IntegersMinMax;

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
        *self_rc.u1.borrow_mut() = _io.read_u1()?;
        *self_rc.u2le.borrow_mut() = _io.read_u2le()?;
        *self_rc.u4le.borrow_mut() = _io.read_u4le()?;
        *self_rc.u8le.borrow_mut() = _io.read_u8le()?;
        *self_rc.u2be.borrow_mut() = _io.read_u2be()?;
        *self_rc.u4be.borrow_mut() = _io.read_u4be()?;
        *self_rc.u8be.borrow_mut() = _io.read_u8be()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl IntegersMinMax_Unsigned {
}
impl IntegersMinMax_Unsigned {
    pub fn u1(&self) -> Ref<'_, u8> {
        self.u1.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u2le(&self) -> Ref<'_, u16> {
        self.u2le.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u4le(&self) -> Ref<'_, u32> {
        self.u4le.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u8le(&self) -> Ref<'_, u64> {
        self.u8le.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u2be(&self) -> Ref<'_, u16> {
        self.u2be.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u4be(&self) -> Ref<'_, u32> {
        self.u4be.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn u8be(&self) -> Ref<'_, u64> {
        self.u8be.borrow()
    }
}
impl IntegersMinMax_Unsigned {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
