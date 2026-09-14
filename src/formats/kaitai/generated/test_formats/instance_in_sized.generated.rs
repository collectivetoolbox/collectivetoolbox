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
pub struct InstanceInSized {
    pub(crate) _root: SharedType<InstanceInSized>,
    pub(crate) _parent: SharedType<InstanceInSized>,
    pub(crate) _self_shared: SharedType<Self>,
    cont: RefCell<OptRc<InstanceInSized_Wrapper>>,
    _io: RefCell<BytesReader>,
    cont_raw: RefCell<Vec<u8>>,
}
impl KStruct for InstanceInSized {
    type Root = InstanceInSized;
    type Parent = InstanceInSized;

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
        let _raw_cont = _io.read_bytes(16_usize)?;
        *self_rc.cont_raw.borrow_mut() = _raw_cont.clone();
        let _io_cont = BytesReader::from(_raw_cont);
        let t = Self::read_into::<BytesReader, InstanceInSized_Wrapper>(&_io_cont, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.cont.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInSized {
}
impl InstanceInSized {
    pub fn cont(&self) -> Ref<'_, OptRc<InstanceInSized_Wrapper>> {
        self.cont.borrow()
    }
}
impl InstanceInSized {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceInSized {
    pub fn cont_raw(&self) -> Ref<'_, Vec<u8>> {
        self.cont_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceInSized_Bar {
    pub(crate) _root: SharedType<InstanceInSized>,
    pub(crate) _parent: SharedType<InstanceInSized_Wrapper>,
    pub(crate) _self_shared: SharedType<Self>,
    seq_f: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_inst: Cell<bool>,
    inst: RefCell<Vec<u8>>,
}
impl KStruct for InstanceInSized_Bar {
    type Root = InstanceInSized;
    type Parent = InstanceInSized_Wrapper;

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
        *self_rc.seq_f.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInSized_Bar {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_inst.get() {
            return Ok(self.inst.borrow());
        }
        self.f_inst.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((4_i32).saturating_add(1_i32))?)?;
        *self.inst.borrow_mut() = _io.read_bytes(3_usize)?;
        _io.seek(_pos)?;
        Ok(self.inst.borrow())
    }
}
impl InstanceInSized_Bar {
    pub fn seq_f(&self) -> Ref<'_, u8> {
        self.seq_f.borrow()
    }
}
impl InstanceInSized_Bar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceInSized_Baz {
    pub(crate) _root: SharedType<InstanceInSized>,
    pub(crate) _parent: SharedType<InstanceInSized_Wrapper>,
    pub(crate) _self_shared: SharedType<Self>,
    seq_f: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_inst: Cell<bool>,
    inst: RefCell<Vec<u8>>,
}
impl KStruct for InstanceInSized_Baz {
    type Root = InstanceInSized;
    type Parent = InstanceInSized_Wrapper;

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
        *self_rc.seq_f.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInSized_Baz {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_inst.get() {
            return Ok(self.inst.borrow());
        }
        self.f_inst.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((8_i32).saturating_add(1_i32))?)?;
        *self.inst.borrow_mut() = _io.read_bytes(3_usize)?;
        _io.seek(_pos)?;
        Ok(self.inst.borrow())
    }
}
impl InstanceInSized_Baz {
    pub fn seq_f(&self) -> Ref<'_, u8> {
        self.seq_f.borrow()
    }
}
impl InstanceInSized_Baz {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceInSized_Qux {
    pub(crate) _root: SharedType<InstanceInSized>,
    pub(crate) _parent: SharedType<InstanceInSized_Wrapper>,
    pub(crate) _self_shared: SharedType<Self>,
    seq_f: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_inst_invoked: Cell<bool>,
    inst_invoked: RefCell<u8>,
    f_inst_unused_by_seq: Cell<bool>,
    inst_unused_by_seq: RefCell<Vec<u8>>,
}
impl KStruct for InstanceInSized_Qux {
    type Root = InstanceInSized;
    type Parent = InstanceInSized_Wrapper;

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
        if ((to_i128(*self_rc.inst_invoked()?)) > (to_i128(0))) {
            *self_rc.seq_f.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInSized_Qux {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst_invoked(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_inst_invoked.get() {
            return Ok(self.inst_invoked.borrow());
        }
        self.f_inst_invoked.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(1_usize))?)?;
        *self.inst_invoked.borrow_mut() = _io.read_u1()?;
        _io.seek(_pos)?;
        Ok(self.inst_invoked.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst_unused_by_seq(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_inst_unused_by_seq.get() {
            return Ok(self.inst_unused_by_seq.borrow());
        }
        self.f_inst_unused_by_seq.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(1_usize))?)?;
        *self.inst_unused_by_seq.borrow_mut() = _io.read_bytes(2_usize)?;
        _io.seek(_pos)?;
        Ok(self.inst_unused_by_seq.borrow())
    }
}
impl InstanceInSized_Qux {
    pub fn seq_f(&self) -> Ref<'_, u8> {
        self.seq_f.borrow()
    }
}
impl InstanceInSized_Qux {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InstanceInSized_Wrapper {
    pub(crate) _root: SharedType<InstanceInSized>,
    pub(crate) _parent: SharedType<InstanceInSized>,
    pub(crate) _self_shared: SharedType<Self>,
    seq_sized: RefCell<OptRc<InstanceInSized_Qux>>,
    seq_in_stream: RefCell<OptRc<InstanceInSized_Bar>>,
    _io: RefCell<BytesReader>,
    seq_sized_raw: RefCell<Vec<u8>>,
    inst_sized_raw: RefCell<Vec<u8>>,
    f_inst_in_stream: Cell<bool>,
    inst_in_stream: RefCell<OptRc<InstanceInSized_Baz>>,
    f_inst_sized: Cell<bool>,
    inst_sized: RefCell<OptRc<InstanceInSized_Qux>>,
}
impl KStruct for InstanceInSized_Wrapper {
    type Root = InstanceInSized;
    type Parent = InstanceInSized;

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
        let _raw_seq_sized = _io.read_bytes(4_usize)?;
        *self_rc.seq_sized_raw.borrow_mut() = _raw_seq_sized.clone();
        let _io_seq_sized = BytesReader::from(_raw_seq_sized);
        let t = Self::read_into::<BytesReader, InstanceInSized_Qux>(&_io_seq_sized, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.seq_sized.borrow_mut() = t;
        let t = Self::read_into::<_, InstanceInSized_Bar>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.seq_in_stream.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl InstanceInSized_Wrapper {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst_in_stream(
        &self
    ) -> KResult<Ref<'_, OptRc<InstanceInSized_Baz>>> {
        let _io = self._io.borrow();
        if self.f_inst_in_stream.get() {
            return Ok(self.inst_in_stream.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(3_usize))?)?;
        let t = Self::read_into::<_, InstanceInSized_Baz>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.inst_in_stream.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.inst_in_stream.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn inst_sized(
        &self
    ) -> KResult<Ref<'_, OptRc<InstanceInSized_Qux>>> {
        let _io = self._io.borrow();
        if self.f_inst_sized.get() {
            return Ok(self.inst_sized.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from((_io.pos()).saturating_add(7_usize))?)?;
        *self.inst_sized_raw.borrow_mut() = _io.read_bytes(4_usize)?.into();
        let inst_sized_raw = self.inst_sized_raw.borrow();
        let _t_inst_sized_raw_io = BytesReader::from(inst_sized_raw.clone());
        let t = Self::read_into::<BytesReader, InstanceInSized_Qux>(&_t_inst_sized_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.inst_sized.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.inst_sized.borrow())
    }
}
impl InstanceInSized_Wrapper {
    pub fn seq_sized(&self) -> Ref<'_, OptRc<InstanceInSized_Qux>> {
        self.seq_sized.borrow()
    }
}
impl InstanceInSized_Wrapper {
    pub fn seq_in_stream(&self) -> Ref<'_, OptRc<InstanceInSized_Bar>> {
        self.seq_in_stream.borrow()
    }
}
impl InstanceInSized_Wrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl InstanceInSized_Wrapper {
    pub fn seq_sized_raw(&self) -> Ref<'_, Vec<u8>> {
        self.seq_sized_raw.borrow()
    }
}
impl InstanceInSized_Wrapper {
    pub fn inst_sized_raw(&self) -> Ref<'_, Vec<u8>> {
        self.inst_sized_raw.borrow()
    }
}
