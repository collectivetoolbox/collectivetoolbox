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
pub struct ExprBits {
    pub(crate) _root: SharedType<ExprBits>,
    pub(crate) _parent: SharedType<ExprBits>,
    pub(crate) _self_shared: SharedType<Self>,
    enum_seq: RefCell<ExprBits_Items>,
    a: RefCell<u64>,
    byte_size: RefCell<Vec<u8>>,
    repeat_expr: RefCell<Vec<i8>>,
    switch_on_type: RefCell<Option<ExprBits_SwitchOnType>>,
    switch_on_endian: RefCell<OptRc<ExprBits_EndianSwitch>>,
    _io: RefCell<BytesReader>,
    byte_size_raw: RefCell<Vec<u8>>,
    f_enum_inst: Cell<bool>,
    enum_inst: RefCell<ExprBits_Items>,
    f_inst_pos: Cell<bool>,
    inst_pos: RefCell<i8>,
}
#[derive(Debug, Clone)]
pub enum ExprBits_SwitchOnType {
    S1(i8),
}
impl From<i8> for ExprBits_SwitchOnType {
    fn from(v: i8) -> Self {
        Self::S1(v)
    }
}
impl TryFrom<&ExprBits_SwitchOnType> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ExprBits_SwitchOnType) -> Result<Self, Self::Error> {
        match e {
            ExprBits_SwitchOnType::S1(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&ExprBits_SwitchOnType> for i8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ExprBits_SwitchOnType) -> Result<Self, Self::Error> {
        match e {
            ExprBits_SwitchOnType::S1(v) => Ok(i8::try_from(*v)?),
        }
    }
}
impl TryFrom<&ExprBits_SwitchOnType> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &ExprBits_SwitchOnType) -> Result<Self, Self::Error> {
        match e {
            ExprBits_SwitchOnType::S1(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&ExprBits_SwitchOnType> for usize {
    type Error = KError;
    fn try_from(e: &ExprBits_SwitchOnType) -> Result<Self, Self::Error> {
        match e {
            ExprBits_SwitchOnType::S1(v) => Ok(usize::try_from(*v)?),
        }
    }
}

impl KStruct for ExprBits {
    type Root = ExprBits;
    type Parent = ExprBits;

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
        *self_rc.enum_seq.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        *self_rc.a.borrow_mut() = _io.read_bits_int_be(3)?;
        io.align_to_byte()?;
        *self_rc.byte_size.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.a())?)?;
        *self_rc.repeat_expr.borrow_mut() = Vec::new();
        let l_repeat_expr = usize::try_from(*self_rc.a())?;
        for _i in 0_usize..l_repeat_expr {
            self_rc.repeat_expr.borrow_mut().push(_io.read_s1()?);
        }
        match *self_rc.a() {
            2 => {
                *self_rc.switch_on_type.borrow_mut() = Some(_io.read_s1()?.into());
            }
            _ => {}
        }
        let t = Self::read_into::<_, ExprBits_EndianSwitch>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.switch_on_endian.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprBits {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn enum_inst(
        &self
    ) -> KResult<Ref<'_, ExprBits_Items>> {
        let _io = self._io.borrow();
        if self.f_enum_inst.get() {
            return Ok(self.enum_inst.borrow());
        }
        self.f_enum_inst.set(true);
        *self.enum_inst.borrow_mut() = i64::try_from(*self.a())?.try_into()?;
        Ok(self.enum_inst.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn inst_pos(
        &self
    ) -> KResult<Ref<'_, i8>> {
        let _io = self._io.borrow();
        if self.f_inst_pos.get() {
            return Ok(self.inst_pos.borrow());
        }
        self.f_inst_pos.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.a())?)?;
        *self.inst_pos.borrow_mut() = _io.read_s1()?;
        _io.seek(_pos)?;
        Ok(self.inst_pos.borrow())
    }
}
impl ExprBits {
    pub fn enum_seq(&self) -> Ref<'_, ExprBits_Items> {
        self.enum_seq.borrow()
    }
}
impl ExprBits {
    pub fn a(&self) -> Ref<'_, u64> {
        self.a.borrow()
    }
}
impl ExprBits {
    pub fn byte_size(&self) -> Ref<'_, Vec<u8>> {
        self.byte_size.borrow()
    }
}
impl ExprBits {
    pub fn repeat_expr(&self) -> Ref<'_, Vec<i8>> {
        self.repeat_expr.borrow()
    }
}
impl ExprBits {
    pub fn switch_on_type(&self) -> i8 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.switch_on_type.borrow().as_ref().and_then(|v| i8::try_from(v).ok()).unwrap_or(0)
    }
    pub fn switch_on_type_enum(&self) -> Ref<'_, Option<ExprBits_SwitchOnType>> {
        self.switch_on_type.borrow()
    }
}
impl ExprBits {
    pub fn switch_on_endian(&self) -> Ref<'_, OptRc<ExprBits_EndianSwitch>> {
        self.switch_on_endian.borrow()
    }
}
impl ExprBits {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprBits {
    pub fn byte_size_raw(&self) -> Ref<'_, Vec<u8>> {
        self.byte_size_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExprBits_Items {
    Foo,
    Bar,
    Unknown(i64),
}

impl TryFrom<i64> for ExprBits_Items {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ExprBits_Items> {
        match flag {
            1 => Ok(ExprBits_Items::Foo),
            2 => Ok(ExprBits_Items::Bar),
            _ => Ok(ExprBits_Items::Unknown(flag)),
        }
    }
}

impl From<&ExprBits_Items> for i64 {
    fn from(v: &ExprBits_Items) -> Self {
        match *v {
            ExprBits_Items::Foo => 1,
            ExprBits_Items::Bar => 2,
            ExprBits_Items::Unknown(v) => v
        }
    }
}

impl Default for ExprBits_Items {
    fn default() -> Self { ExprBits_Items::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct ExprBits_EndianSwitch {
    pub(crate) _root: SharedType<ExprBits>,
    pub(crate) _parent: SharedType<ExprBits>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<i16>,
    _io: RefCell<BytesReader>,
    _is_le: RefCell<i32>,
}
impl KStruct for ExprBits_EndianSwitch {
    type Root = ExprBits;
    type Parent = ExprBits;

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
        match *self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.a() {
            1 => {
                *self_rc._is_le.borrow_mut() = 1_i32;
            }
            2 => {
                *self_rc._is_le.borrow_mut() = 2_i32;
            }
            _ => {}
        }
        if *self_rc._is_le.borrow() == 0 {
            return Err(KError::UndecidedEndianness { src_path: "/types/endian_switch".to_string() });
        }
        *self_rc.foo.borrow_mut() = if *self_rc._is_le.borrow() == 1 { _io.read_s2le()? } else { _io.read_s2be()? };
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprBits_EndianSwitch {
    pub fn set_endian(&mut self, _is_le: i32) {
        *self._is_le.borrow_mut() = _is_le;
    }
}
impl ExprBits_EndianSwitch {
}
impl ExprBits_EndianSwitch {
    pub fn foo(&self) -> Ref<'_, i16> {
        self.foo.borrow()
    }
}
impl ExprBits_EndianSwitch {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
