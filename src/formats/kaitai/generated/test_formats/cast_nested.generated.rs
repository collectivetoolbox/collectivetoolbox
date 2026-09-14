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
pub struct CastNested {
    pub(crate) _root: SharedType<CastNested>,
    pub(crate) _parent: SharedType<CastNested>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<CastNested_Opcode>>>,
    _io: RefCell<BytesReader>,
    f_opcodes_0_str: Cell<bool>,
    opcodes_0_str: RefCell<i32>,
    f_opcodes_0_str_value: Cell<bool>,
    opcodes_0_str_value: RefCell<i32>,
    f_opcodes_1_int: Cell<bool>,
    opcodes_1_int: RefCell<i32>,
    f_opcodes_1_int_value: Cell<bool>,
    opcodes_1_int_value: RefCell<i32>,
}
impl KStruct for CastNested {
    type Root = CastNested;
    type Parent = CastNested;

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
        *self_rc.opcodes.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, CastNested_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CastNested {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn opcodes_0_str(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_opcodes_0_str.get() {
            return Ok(self.opcodes_0_str.borrow());
        }
        self.f_opcodes_0_str.set(true);
        *self.opcodes_0_str.borrow_mut() = (OptRc::<CastNested_Opcode_Strval>::try_from(&*(self.opcodes().get(0_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?).try_into()?;
        Ok(self.opcodes_0_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn opcodes_0_str_value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_opcodes_0_str_value.get() {
            return Ok(self.opcodes_0_str_value.borrow());
        }
        self.f_opcodes_0_str_value.set(true);
        *self.opcodes_0_str_value.borrow_mut() = (*OptRc::<CastNested_Opcode_Strval>::try_from(&*(self.opcodes().get(0_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?.value()).try_into()?;
        Ok(self.opcodes_0_str_value.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn opcodes_1_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_opcodes_1_int.get() {
            return Ok(self.opcodes_1_int.borrow());
        }
        self.f_opcodes_1_int.set(true);
        *self.opcodes_1_int.borrow_mut() = (OptRc::<CastNested_Opcode_Intval>::try_from(&*(self.opcodes().get(1_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?).try_into()?;
        Ok(self.opcodes_1_int.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn opcodes_1_int_value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_opcodes_1_int_value.get() {
            return Ok(self.opcodes_1_int_value.borrow());
        }
        self.f_opcodes_1_int_value.set(true);
        *self.opcodes_1_int_value.borrow_mut() = (*OptRc::<CastNested_Opcode_Intval>::try_from(&*(self.opcodes().get(1_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?.value()).try_into()?;
        Ok(self.opcodes_1_int_value.borrow())
    }
}
impl CastNested {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<CastNested_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl CastNested {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CastNested_Opcode {
    pub(crate) _root: SharedType<CastNested>,
    pub(crate) _parent: SharedType<CastNested>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    body: RefCell<Option<CastNested_Opcode_Body>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum CastNested_Opcode_Body {
    CastNested_Opcode_Intval(OptRc<CastNested_Opcode_Intval>),
    CastNested_Opcode_Strval(OptRc<CastNested_Opcode_Strval>),
}
impl TryFrom<&CastNested_Opcode_Body> for OptRc<CastNested_Opcode_Intval> {
    type Error = KError;
    fn try_from(v: &CastNested_Opcode_Body) -> Result<Self, Self::Error> {
        if let CastNested_Opcode_Body::CastNested_Opcode_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<CastNested_Opcode_Intval>> for CastNested_Opcode_Body {
    fn from(v: OptRc<CastNested_Opcode_Intval>) -> Self {
        Self::CastNested_Opcode_Intval(v)
    }
}
impl TryFrom<&CastNested_Opcode_Body> for OptRc<CastNested_Opcode_Strval> {
    type Error = KError;
    fn try_from(v: &CastNested_Opcode_Body) -> Result<Self, Self::Error> {
        if let CastNested_Opcode_Body::CastNested_Opcode_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<CastNested_Opcode_Strval>> for CastNested_Opcode_Body {
    fn from(v: OptRc<CastNested_Opcode_Strval>) -> Self {
        Self::CastNested_Opcode_Strval(v)
    }
}
impl KStruct for CastNested_Opcode {
    type Root = CastNested;
    type Parent = CastNested;

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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        match *self_rc.code() {
            73 => {
                let t = Self::read_into::<_, CastNested_Opcode_Intval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            83 => {
                let t = Self::read_into::<_, CastNested_Opcode_Strval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CastNested_Opcode {
}
impl CastNested_Opcode {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl CastNested_Opcode {
    pub fn body(&self) -> Ref<'_, Option<CastNested_Opcode_Body>> {
        self.body.borrow()
    }
}
impl CastNested_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CastNested_Opcode_Intval {
    pub(crate) _root: SharedType<CastNested>,
    pub(crate) _parent: SharedType<CastNested_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for CastNested_Opcode_Intval {
    type Root = CastNested;
    type Parent = CastNested_Opcode;

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
        *self_rc.value.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CastNested_Opcode_Intval {
}
impl CastNested_Opcode_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl CastNested_Opcode_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct CastNested_Opcode_Strval {
    pub(crate) _root: SharedType<CastNested>,
    pub(crate) _parent: SharedType<CastNested_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for CastNested_Opcode_Strval {
    type Root = CastNested;
    type Parent = CastNested_Opcode;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CastNested_Opcode_Strval {
}
impl CastNested_Opcode_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl CastNested_Opcode_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
