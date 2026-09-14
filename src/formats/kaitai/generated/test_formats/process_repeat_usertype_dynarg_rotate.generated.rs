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
pub struct ProcessRepeatUsertypeDynargRotate {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _self_shared: SharedType<Self>,
    blocks_rol: RefCell<Vec<OptRc<ProcessRepeatUsertypeDynargRotate_Block>>>,
    blocks_ror: RefCell<Vec<OptRc<ProcessRepeatUsertypeDynargRotate_Block>>>,
    blocks_b: RefCell<OptRc<ProcessRepeatUsertypeDynargRotate_BlocksBWrapper>>,
    _io: RefCell<BytesReader>,
    blocks_rol_raw: RefCell<Vec<u8>>,
    blocks_ror_raw: RefCell<Vec<u8>>,
}
impl KStruct for ProcessRepeatUsertypeDynargRotate {
    type Root = ProcessRepeatUsertypeDynargRotate;
    type Parent = ProcessRepeatUsertypeDynargRotate;

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
        *self_rc.blocks_rol.borrow_mut() = Vec::new();
        let l_blocks_rol = 2_usize;
        for _i in 0_usize..l_blocks_rol {
            let _raw_blocks_rol = _io.read_bytes(3_usize)?;
            let _processed_blocks_rol = process_rotate_left(&_raw_blocks_rol, i64::try_from((_io.pos()).saturating_sub((4_usize).saturating_mul(_i)))?);
            let _io_blocks_rol = BytesReader::from(_processed_blocks_rol);
            let t = Self::read_into::<BytesReader, ProcessRepeatUsertypeDynargRotate_Block>(&_io_blocks_rol, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.blocks_rol.borrow_mut().push(t);
        }
        *self_rc.blocks_ror.borrow_mut() = Vec::new();
        let l_blocks_ror = 3_usize;
        for _i in 0_usize..l_blocks_ror {
            let _raw_blocks_ror = _io.read_bytes(3_usize)?;
            let _processed_blocks_ror = process_rotate_right(&_raw_blocks_ror, i64::try_from(((_io.pos()).saturating_sub(6_usize)).saturating_sub((4_usize).saturating_mul(_i)))?);
            let _io_blocks_ror = BytesReader::from(_processed_blocks_ror);
            let t = Self::read_into::<BytesReader, ProcessRepeatUsertypeDynargRotate_Block>(&_io_blocks_ror, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.blocks_ror.borrow_mut().push(t);
        }
        let t = Self::read_into::<_, ProcessRepeatUsertypeDynargRotate_BlocksBWrapper>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.blocks_b.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargRotate {
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn blocks_rol(&self) -> Ref<'_, Vec<OptRc<ProcessRepeatUsertypeDynargRotate_Block>>> {
        self.blocks_rol.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn blocks_ror(&self) -> Ref<'_, Vec<OptRc<ProcessRepeatUsertypeDynargRotate_Block>>> {
        self.blocks_ror.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn blocks_b(&self) -> Ref<'_, OptRc<ProcessRepeatUsertypeDynargRotate_BlocksBWrapper>> {
        self.blocks_b.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn blocks_rol_raw(&self) -> Ref<'_, Vec<u8>> {
        self.blocks_rol_raw.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate {
    pub fn blocks_ror_raw(&self) -> Ref<'_, Vec<u8>> {
        self.blocks_ror_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessRepeatUsertypeDynargRotate_Block {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ProcessRepeatUsertypeDynargRotate_Block {
    type Root = ProcessRepeatUsertypeDynargRotate;
    type Parent = ProcessRepeatUsertypeDynargRotate;

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
        *self_rc.a.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargRotate_Block {
}
impl ProcessRepeatUsertypeDynargRotate_Block {
    pub fn a(&self) -> Ref<'_, u16> {
        self.a.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessRepeatUsertypeDynargRotate_BlocksBWrapper {
    pub(crate) _root: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _parent: SharedType<ProcessRepeatUsertypeDynargRotate>,
    pub(crate) _self_shared: SharedType<Self>,
    dummy: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_blocks_rol_0_b: Cell<bool>,
    blocks_rol_0_b: RefCell<u8>,
    f_blocks_rol_1_b: Cell<bool>,
    blocks_rol_1_b: RefCell<u8>,
    f_blocks_ror_0_b: Cell<bool>,
    blocks_ror_0_b: RefCell<u8>,
    f_blocks_ror_1_b: Cell<bool>,
    blocks_ror_1_b: RefCell<u8>,
    f_blocks_ror_2_b: Cell<bool>,
    blocks_ror_2_b: RefCell<u8>,
}
impl KStruct for ProcessRepeatUsertypeDynargRotate_BlocksBWrapper {
    type Root = ProcessRepeatUsertypeDynargRotate;
    type Parent = ProcessRepeatUsertypeDynargRotate;

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
        *self_rc.dummy.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessRepeatUsertypeDynargRotate_BlocksBWrapper {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn blocks_rol_0_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_rol_0_b.get() {
            return Ok(self.blocks_rol_0_b.borrow());
        }
        self.f_blocks_rol_0_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks_rol().get(0_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(2_usize)?;
        *self.blocks_rol_0_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_rol_0_b.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn blocks_rol_1_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_rol_1_b.get() {
            return Ok(self.blocks_rol_1_b.borrow());
        }
        self.f_blocks_rol_1_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks_rol().get(1_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(2_usize)?;
        *self.blocks_rol_1_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_rol_1_b.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn blocks_ror_0_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_ror_0_b.get() {
            return Ok(self.blocks_ror_0_b.borrow());
        }
        self.f_blocks_ror_0_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks_ror().get(0_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(2_usize)?;
        *self.blocks_ror_0_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_ror_0_b.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn blocks_ror_1_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_ror_1_b.get() {
            return Ok(self.blocks_ror_1_b.borrow());
        }
        self.f_blocks_ror_1_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks_ror().get(1_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(2_usize)?;
        *self.blocks_ror_1_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_ror_1_b.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn blocks_ror_2_b(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_blocks_ror_2_b.get() {
            return Ok(self.blocks_ror_2_b.borrow());
        }
        self.f_blocks_ror_2_b.set(true);
        let io = KStream::clone(&*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.blocks_ror().get(2_usize).ok_or(KError::CastError)?._io());
        let _pos = io.pos();
        io.seek(2_usize)?;
        *self.blocks_ror_2_b.borrow_mut() = io.read_u1()?;
        io.seek(_pos)?;
        Ok(self.blocks_ror_2_b.borrow())
    }
}
impl ProcessRepeatUsertypeDynargRotate_BlocksBWrapper {
    pub fn dummy(&self) -> Ref<'_, u8> {
        self.dummy.borrow()
    }
}
impl ProcessRepeatUsertypeDynargRotate_BlocksBWrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
