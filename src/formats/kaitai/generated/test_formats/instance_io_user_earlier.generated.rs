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
pub struct InstanceIoUserEarlier {
    pub(crate) _root: SharedType<InstanceIoUserEarlier>,
    pub(crate) _parent: SharedType<InstanceIoUserEarlier>,
    pub(crate) _self_shared: SharedType<Self>,
    sized_a: RefCell<OptRc<InstanceIoUserEarlier_Slot>>,
    sized_b: RefCell<OptRc<InstanceIoUserEarlier_Slot>>,
    into_b: RefCell<OptRc<InstanceIoUserEarlier_Foo>>,
    into_a_skipped: RefCell<OptRc<InstanceIoUserEarlier_Foo>>,
    into_a: RefCell<OptRc<InstanceIoUserEarlier_Foo>>,
    last_accessor: RefCell<OptRc<InstanceIoUserEarlier_Baz>>,
    _io: RefCell<BytesReader>,
    sized_a_raw: RefCell<Vec<u8>>,
    sized_b_raw: RefCell<Vec<u8>>,
    f_a_mid: Cell<bool>,
    a_mid: RefCell<u16>,
    f_b_mid: Cell<bool>,
    b_mid: RefCell<u16>,
}
impl KStruct for InstanceIoUserEarlier {
    type Root = InstanceIoUserEarlier;
    type Parent = InstanceIoUserEarlier;

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
        let _raw_sized_a = _io.read_bytes(6_usize)?;
        *self_rc.sized_a_raw.borrow_mut() = _raw_sized_a.clone();
        let _io_sized_a = BytesReader::from(_raw_sized_a);
        let t = Self::read_into::<BytesReader, InstanceIoUserEarlier_Slot>(&_io_sized_a, Some(self_rc._root.clone()), None)?.into();
        *self_rc.sized_a.borrow_mut() = t;
        let _raw_sized_b = _io.read_bytes(6_usize)?;
        *self_rc.sized_b_raw.borrow_mut() = _raw_sized_b.clone();
        let _io_sized_b = BytesReader::from(_raw_sized_b);
        let t = Self::read_into::<BytesReader, InstanceIoUserEarlier_Slot>(&_io_sized_b, Some(self_rc._root.clone()), None)?.into();
        *self_rc.sized_b.borrow_mut() = t;
        let t = Self::read_into::<_, InstanceIoUserEarlier_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.into_b.borrow_mut() = t;
        let t = Self::read_into::<_, InstanceIoUserEarlier_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.into_a_skipped.borrow_mut() = t;
        let t = Self::read_into::<_, InstanceIoUserEarlier_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.into_a.borrow_mut() = t;
        let t = Self::read_into::<_, InstanceIoUserEarlier_Baz>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.last_accessor.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUserEarlier {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn a_mid(
        &self
    ) -> KResult<Ref<'_, u16>> {
        let _io = self._io.borrow();
        if self.f_a_mid.get() {
            return Ok(self.a_mid.borrow());
        }
        self.f_a_mid.set(true);
        let io = KStream::clone(&*self.into_a().inst()?._io());
        let _pos = io.pos();
        io.seek(1_usize)?;
        *self.a_mid.borrow_mut() = io.read_u2le()?;
        io.seek(_pos)?;
        Ok(self.a_mid.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn b_mid(
        &self
    ) -> KResult<Ref<'_, u16>> {
        let _io = self._io.borrow();
        if self.f_b_mid.get() {
            return Ok(self.b_mid.borrow());
        }
        self.f_b_mid.set(true);
        let io = KStream::clone(&*self.into_b().inst()?._io());
        let _pos = io.pos();
        io.seek(1_usize)?;
        *self.b_mid.borrow_mut() = io.read_u2le()?;
        io.seek(_pos)?;
        Ok(self.b_mid.borrow())
    }
}
impl InstanceIoUserEarlier {
    pub fn sized_a(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Slot>> {
        self.sized_a.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn sized_b(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Slot>> {
        self.sized_b.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn into_b(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Foo>> {
        self.into_b.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn into_a_skipped(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Foo>> {
        self.into_a_skipped.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn into_a(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Foo>> {
        self.into_a.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn last_accessor(&self) -> Ref<'_, OptRc<InstanceIoUserEarlier_Baz>> {
        self.last_accessor.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn sized_a_raw(&self) -> Ref<'_, Vec<u8>> {
        self.sized_a_raw.borrow()
    }
}
impl InstanceIoUserEarlier {
    pub fn sized_b_raw(&self) -> Ref<'_, Vec<u8>> {
        self.sized_b_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceIoUserEarlier_Baz {
    pub(crate) _root: SharedType<InstanceIoUserEarlier>,
    pub(crate) _parent: SharedType<InstanceIoUserEarlier>,
    pub(crate) _self_shared: SharedType<Self>,
    v: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for InstanceIoUserEarlier_Baz {
    type Root = InstanceIoUserEarlier;
    type Parent = InstanceIoUserEarlier;

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
        if ((to_i128(self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.into_b().inst()?.last().ok_or(KError::EmptyIterator)?)) == (to_i128(89))) {
            *self_rc.v.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUserEarlier_Baz {
}
impl InstanceIoUserEarlier_Baz {
    pub fn v(&self) -> Ref<'_, u8> {
        self.v.borrow()
    }
}
impl InstanceIoUserEarlier_Baz {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceIoUserEarlier_Foo {
    pub(crate) _root: SharedType<InstanceIoUserEarlier>,
    pub(crate) _parent: SharedType<InstanceIoUserEarlier>,
    pub(crate) _self_shared: SharedType<Self>,
    indicator: RefCell<u8>,
    bar: RefCell<u8>,
    _io: RefCell<BytesReader>,
    inst_raw: RefCell<Vec<u8>>,
    f_inst: Cell<bool>,
    inst: RefCell<OptRc<InstanceIoUserEarlier_Slot>>,
}
impl KStruct for InstanceIoUserEarlier_Foo {
    type Root = InstanceIoUserEarlier;
    type Parent = InstanceIoUserEarlier;

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
        *self_rc.indicator.borrow_mut() = _io.read_u1()?;
        if  (((i64::try_from(self_rc.inst()?._io().size())?) != 0) && (((to_i128(*self_rc.inst()?.content())) == (to_i128(102)))))  {
            *self_rc.bar.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUserEarlier_Foo {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn inst(
        &self
    ) -> KResult<Ref<'_, OptRc<InstanceIoUserEarlier_Slot>>> {
        let _io = self._io.borrow();
        if self.f_inst.get() {
            return Ok(self.inst.borrow());
        }
        let io = if ((to_i128(*self.indicator())) == (to_i128(202))) { KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.sized_b()._io()) } else { KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.sized_a()._io()) };
        let _pos = io.pos();
        io.seek(1_usize)?;
        *self.inst_raw.borrow_mut() = _io.read_bytes(usize::try_from(if _io.pos() != 14 { 4_i32 } else { 0_i32 })?)?.into();
        let inst_raw = self.inst_raw.borrow();
        let _t_inst_raw_io = BytesReader::from(inst_raw.clone());
        let t = Self::read_into::<BytesReader, InstanceIoUserEarlier_Slot>(&_t_inst_raw_io, Some(self._root.clone()), None)?.into();
        *self.inst.borrow_mut() = t;
        io.seek(_pos)?;
        Ok(self.inst.borrow())
    }
}
impl InstanceIoUserEarlier_Foo {
    pub fn indicator(&self) -> Ref<'_, u8> {
        self.indicator.borrow()
    }
}
impl InstanceIoUserEarlier_Foo {
    pub fn bar(&self) -> Ref<'_, u8> {
        self.bar.borrow()
    }
}
impl InstanceIoUserEarlier_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceIoUserEarlier_Foo {
    pub fn inst_raw(&self) -> Ref<'_, Vec<u8>> {
        self.inst_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceIoUserEarlier_Slot {
    pub(crate) _root: SharedType<InstanceIoUserEarlier>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    content: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_last: Cell<bool>,
    last: RefCell<u8>,
}
impl KStruct for InstanceIoUserEarlier_Slot {
    type Root = InstanceIoUserEarlier;
    type Parent = KStructUnit;

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
        if (i64::try_from(_io.size())?) != 0 {
            *self_rc.content.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceIoUserEarlier_Slot {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn last(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_last.get() {
            return Ok(self.last.borrow());
        }
        self.f_last.set(true);
        if (i64::try_from(_io.size())?) != 0 {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((i64::try_from(_io.size())?)).saturating_sub(1_i32))?)?;
            *self.last.borrow_mut() = _io.read_u1()?;
            _io.seek(_pos)?;
        }
        Ok(self.last.borrow())
    }
}
impl InstanceIoUserEarlier_Slot {
    pub fn content(&self) -> Ref<'_, u8> {
        self.content.borrow()
    }
}
impl InstanceIoUserEarlier_Slot {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
