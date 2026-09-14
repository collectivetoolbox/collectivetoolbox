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
pub struct SwitchManualEnumInvalidElse {
    pub(crate) _root: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _parent: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchManualEnumInvalidElse_Opcode>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualEnumInvalidElse {
    type Root = SwitchManualEnumInvalidElse;
    type Parent = SwitchManualEnumInvalidElse;

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
        *self_rc.opcodes.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, SwitchManualEnumInvalidElse_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualEnumInvalidElse {
}
impl SwitchManualEnumInvalidElse {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchManualEnumInvalidElse_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchManualEnumInvalidElse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualEnumInvalidElse_Opcode {
    pub(crate) _root: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _parent: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<SwitchManualEnumInvalidElse_Opcode_CodeEnum>,
    body: RefCell<Option<SwitchManualEnumInvalidElse_Opcode_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualEnumInvalidElse_Opcode_Body {
    SwitchManualEnumInvalidElse_Opcode_Intval(OptRc<SwitchManualEnumInvalidElse_Opcode_Intval>),
    SwitchManualEnumInvalidElse_Opcode_Strval(OptRc<SwitchManualEnumInvalidElse_Opcode_Strval>),
    SwitchManualEnumInvalidElse_Opcode_Defval(OptRc<SwitchManualEnumInvalidElse_Opcode_Defval>),
}
impl TryFrom<&SwitchManualEnumInvalidElse_Opcode_Body> for OptRc<SwitchManualEnumInvalidElse_Opcode_Intval> {
    type Error = KError;
    fn try_from(v: &SwitchManualEnumInvalidElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualEnumInvalidElse_Opcode_Body::SwitchManualEnumInvalidElse_Opcode_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualEnumInvalidElse_Opcode_Intval>> for SwitchManualEnumInvalidElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualEnumInvalidElse_Opcode_Intval>) -> Self {
        Self::SwitchManualEnumInvalidElse_Opcode_Intval(v)
    }
}
impl TryFrom<&SwitchManualEnumInvalidElse_Opcode_Body> for OptRc<SwitchManualEnumInvalidElse_Opcode_Strval> {
    type Error = KError;
    fn try_from(v: &SwitchManualEnumInvalidElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualEnumInvalidElse_Opcode_Body::SwitchManualEnumInvalidElse_Opcode_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualEnumInvalidElse_Opcode_Strval>> for SwitchManualEnumInvalidElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualEnumInvalidElse_Opcode_Strval>) -> Self {
        Self::SwitchManualEnumInvalidElse_Opcode_Strval(v)
    }
}
impl TryFrom<&SwitchManualEnumInvalidElse_Opcode_Body> for OptRc<SwitchManualEnumInvalidElse_Opcode_Defval> {
    type Error = KError;
    fn try_from(v: &SwitchManualEnumInvalidElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualEnumInvalidElse_Opcode_Body::SwitchManualEnumInvalidElse_Opcode_Defval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualEnumInvalidElse_Opcode_Defval>> for SwitchManualEnumInvalidElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualEnumInvalidElse_Opcode_Defval>) -> Self {
        Self::SwitchManualEnumInvalidElse_Opcode_Defval(v)
    }
}
impl KStruct for SwitchManualEnumInvalidElse_Opcode {
    type Root = SwitchManualEnumInvalidElse;
    type Parent = SwitchManualEnumInvalidElse;

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
        *self_rc.code.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        match *self_rc.code() {
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Intval => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualEnumInvalidElse_Opcode_Intval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Strval => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualEnumInvalidElse_Opcode_Strval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualEnumInvalidElse_Opcode_Defval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualEnumInvalidElse_Opcode {
}
impl SwitchManualEnumInvalidElse_Opcode {
    pub fn code(&self) -> Ref<'_, SwitchManualEnumInvalidElse_Opcode_CodeEnum> {
        self.code.borrow()
    }
}
impl SwitchManualEnumInvalidElse_Opcode {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualEnumInvalidElse_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualEnumInvalidElse_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchManualEnumInvalidElse_Opcode {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SwitchManualEnumInvalidElse_Opcode_CodeEnum {
    Foo,
    Intval,
    Strval,
    Unknown(i64),
}

impl TryFrom<i64> for SwitchManualEnumInvalidElse_Opcode_CodeEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<SwitchManualEnumInvalidElse_Opcode_CodeEnum> {
        match flag {
            1 => Ok(SwitchManualEnumInvalidElse_Opcode_CodeEnum::Foo),
            73 => Ok(SwitchManualEnumInvalidElse_Opcode_CodeEnum::Intval),
            83 => Ok(SwitchManualEnumInvalidElse_Opcode_CodeEnum::Strval),
            _ => Ok(SwitchManualEnumInvalidElse_Opcode_CodeEnum::Unknown(flag)),
        }
    }
}

impl From<&SwitchManualEnumInvalidElse_Opcode_CodeEnum> for i64 {
    fn from(v: &SwitchManualEnumInvalidElse_Opcode_CodeEnum) -> Self {
        match *v {
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Foo => 1,
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Intval => 73,
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Strval => 83,
            SwitchManualEnumInvalidElse_Opcode_CodeEnum::Unknown(v) => v
        }
    }
}

impl Default for SwitchManualEnumInvalidElse_Opcode_CodeEnum {
    fn default() -> Self { SwitchManualEnumInvalidElse_Opcode_CodeEnum::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct SwitchManualEnumInvalidElse_Opcode_Defval {
    pub(crate) _root: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _parent: SharedType<SwitchManualEnumInvalidElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_value: Cell<bool>,
    value: RefCell<i32>,
}
impl KStruct for SwitchManualEnumInvalidElse_Opcode_Defval {
    type Root = SwitchManualEnumInvalidElse;
    type Parent = SwitchManualEnumInvalidElse_Opcode;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Defval {
    pub fn value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = (123).try_into()?;
        Ok(self.value.borrow())
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Defval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualEnumInvalidElse_Opcode_Intval {
    pub(crate) _root: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _parent: SharedType<SwitchManualEnumInvalidElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualEnumInvalidElse_Opcode_Intval {
    type Root = SwitchManualEnumInvalidElse;
    type Parent = SwitchManualEnumInvalidElse_Opcode;

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
        *self_rc.value.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Intval {
}
impl SwitchManualEnumInvalidElse_Opcode_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualEnumInvalidElse_Opcode_Strval {
    pub(crate) _root: SharedType<SwitchManualEnumInvalidElse>,
    pub(crate) _parent: SharedType<SwitchManualEnumInvalidElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualEnumInvalidElse_Opcode_Strval {
    type Root = SwitchManualEnumInvalidElse;
    type Parent = SwitchManualEnumInvalidElse_Opcode;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Strval {
}
impl SwitchManualEnumInvalidElse_Opcode_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl SwitchManualEnumInvalidElse_Opcode_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
