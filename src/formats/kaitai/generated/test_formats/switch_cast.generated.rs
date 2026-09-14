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
pub struct SwitchCast {
    pub(crate) _root: SharedType<SwitchCast>,
    pub(crate) _parent: SharedType<SwitchCast>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchCast_Opcode>>>,
    _io: RefCell<BytesReader>,
    f_err_cast: Cell<bool>,
    err_cast: RefCell<OptRc<SwitchCast_Strval>>,
    f_first_obj: Cell<bool>,
    first_obj: RefCell<OptRc<SwitchCast_Strval>>,
    f_second_val: Cell<bool>,
    second_val: RefCell<u8>,
}
impl KStruct for SwitchCast {
    type Root = SwitchCast;
    type Parent = SwitchCast;

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
                let t = Self::read_into::<_, SwitchCast_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchCast {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn err_cast(
        &self
    ) -> KResult<Ref<'_, OptRc<SwitchCast_Strval>>> {
        let _io = self._io.borrow();
        if self.f_err_cast.get() {
            return Ok(self.err_cast.borrow());
        }
        *self.err_cast.borrow_mut() = OptRc::<SwitchCast_Strval>::try_from(&*(self.opcodes().get(2_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?.clone();
        Ok(self.err_cast.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn first_obj(
        &self
    ) -> KResult<Ref<'_, OptRc<SwitchCast_Strval>>> {
        let _io = self._io.borrow();
        if self.f_first_obj.get() {
            return Ok(self.first_obj.borrow());
        }
        *self.first_obj.borrow_mut() = OptRc::<SwitchCast_Strval>::try_from(&*(self.opcodes().get(0_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?.clone();
        Ok(self.first_obj.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn second_val(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_second_val.get() {
            return Ok(self.second_val.borrow());
        }
        self.f_second_val.set(true);
        *self.second_val.borrow_mut() = (*OptRc::<SwitchCast_Intval>::try_from(&*(self.opcodes().get(1_usize).ok_or(KError::CastError)?.body()).as_ref().ok_or(KError::CastError)?)?.value()).try_into()?;
        Ok(self.second_val.borrow())
    }
}
impl SwitchCast {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchCast_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchCast {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchCast_Intval {
    pub(crate) _root: SharedType<SwitchCast>,
    pub(crate) _parent: SharedType<SwitchCast_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchCast_Intval {
    type Root = SwitchCast;
    type Parent = SwitchCast_Opcode;

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
impl SwitchCast_Intval {
}
impl SwitchCast_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl SwitchCast_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchCast_Opcode {
    pub(crate) _root: SharedType<SwitchCast>,
    pub(crate) _parent: SharedType<SwitchCast>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    body: RefCell<Option<SwitchCast_Opcode_Body>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum SwitchCast_Opcode_Body {
    SwitchCast_Intval(OptRc<SwitchCast_Intval>),
    SwitchCast_Strval(OptRc<SwitchCast_Strval>),
}
impl TryFrom<&SwitchCast_Opcode_Body> for OptRc<SwitchCast_Intval> {
    type Error = KError;
    fn try_from(v: &SwitchCast_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchCast_Opcode_Body::SwitchCast_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchCast_Intval>> for SwitchCast_Opcode_Body {
    fn from(v: OptRc<SwitchCast_Intval>) -> Self {
        Self::SwitchCast_Intval(v)
    }
}
impl TryFrom<&SwitchCast_Opcode_Body> for OptRc<SwitchCast_Strval> {
    type Error = KError;
    fn try_from(v: &SwitchCast_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchCast_Opcode_Body::SwitchCast_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchCast_Strval>> for SwitchCast_Opcode_Body {
    fn from(v: OptRc<SwitchCast_Strval>) -> Self {
        Self::SwitchCast_Strval(v)
    }
}
impl KStruct for SwitchCast_Opcode {
    type Root = SwitchCast;
    type Parent = SwitchCast;

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
                let t = Self::read_into::<_, SwitchCast_Intval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            83 => {
                let t = Self::read_into::<_, SwitchCast_Strval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchCast_Opcode {
}
impl SwitchCast_Opcode {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchCast_Opcode {
    pub fn body(&self) -> Ref<'_, Option<SwitchCast_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchCast_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchCast_Strval {
    pub(crate) _root: SharedType<SwitchCast>,
    pub(crate) _parent: SharedType<SwitchCast_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchCast_Strval {
    type Root = SwitchCast;
    type Parent = SwitchCast_Opcode;

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
impl SwitchCast_Strval {
}
impl SwitchCast_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl SwitchCast_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
