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
pub struct SwitchManualIntElse {
    pub(crate) _root: SharedType<SwitchManualIntElse>,
    pub(crate) _parent: SharedType<SwitchManualIntElse>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchManualIntElse_Opcode>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntElse {
    type Root = SwitchManualIntElse;
    type Parent = SwitchManualIntElse;

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
                let t = Self::read_into::<_, SwitchManualIntElse_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl SwitchManualIntElse {
}
impl SwitchManualIntElse {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchManualIntElse_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchManualIntElse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntElse_Opcode {
    pub(crate) _root: SharedType<SwitchManualIntElse>,
    pub(crate) _parent: SharedType<SwitchManualIntElse>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    body: RefCell<Option<SwitchManualIntElse_Opcode_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualIntElse_Opcode_Body {
    SwitchManualIntElse_Opcode_Intval(OptRc<SwitchManualIntElse_Opcode_Intval>),
    SwitchManualIntElse_Opcode_Strval(OptRc<SwitchManualIntElse_Opcode_Strval>),
    SwitchManualIntElse_Opcode_Noneval(OptRc<SwitchManualIntElse_Opcode_Noneval>),
}
impl TryFrom<&SwitchManualIntElse_Opcode_Body> for OptRc<SwitchManualIntElse_Opcode_Intval> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntElse_Opcode_Body::SwitchManualIntElse_Opcode_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntElse_Opcode_Intval>> for SwitchManualIntElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualIntElse_Opcode_Intval>) -> Self {
        Self::SwitchManualIntElse_Opcode_Intval(v)
    }
}
impl TryFrom<&SwitchManualIntElse_Opcode_Body> for OptRc<SwitchManualIntElse_Opcode_Strval> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntElse_Opcode_Body::SwitchManualIntElse_Opcode_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntElse_Opcode_Strval>> for SwitchManualIntElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualIntElse_Opcode_Strval>) -> Self {
        Self::SwitchManualIntElse_Opcode_Strval(v)
    }
}
impl TryFrom<&SwitchManualIntElse_Opcode_Body> for OptRc<SwitchManualIntElse_Opcode_Noneval> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntElse_Opcode_Body::SwitchManualIntElse_Opcode_Noneval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntElse_Opcode_Noneval>> for SwitchManualIntElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualIntElse_Opcode_Noneval>) -> Self {
        Self::SwitchManualIntElse_Opcode_Noneval(v)
    }
}
impl KStruct for SwitchManualIntElse_Opcode {
    type Root = SwitchManualIntElse;
    type Parent = SwitchManualIntElse;

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
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntElse_Opcode_Intval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            83 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntElse_Opcode_Strval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntElse_Opcode_Noneval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
        }
        Ok(())
    }
}
impl SwitchManualIntElse_Opcode {
}
impl SwitchManualIntElse_Opcode {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchManualIntElse_Opcode {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualIntElse_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualIntElse_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchManualIntElse_Opcode {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntElse_Opcode_Intval {
    pub(crate) _root: SharedType<SwitchManualIntElse>,
    pub(crate) _parent: SharedType<SwitchManualIntElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntElse_Opcode_Intval {
    type Root = SwitchManualIntElse;
    type Parent = SwitchManualIntElse_Opcode;

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
        Ok(())
    }
}
impl SwitchManualIntElse_Opcode_Intval {
}
impl SwitchManualIntElse_Opcode_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl SwitchManualIntElse_Opcode_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntElse_Opcode_Noneval {
    pub(crate) _root: SharedType<SwitchManualIntElse>,
    pub(crate) _parent: SharedType<SwitchManualIntElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    filler: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntElse_Opcode_Noneval {
    type Root = SwitchManualIntElse;
    type Parent = SwitchManualIntElse_Opcode;

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
        *self_rc.filler.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl SwitchManualIntElse_Opcode_Noneval {
}
impl SwitchManualIntElse_Opcode_Noneval {
    pub fn filler(&self) -> Ref<'_, u32> {
        self.filler.borrow()
    }
}
impl SwitchManualIntElse_Opcode_Noneval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntElse_Opcode_Strval {
    pub(crate) _root: SharedType<SwitchManualIntElse>,
    pub(crate) _parent: SharedType<SwitchManualIntElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntElse_Opcode_Strval {
    type Root = SwitchManualIntElse;
    type Parent = SwitchManualIntElse_Opcode;

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
        Ok(())
    }
}
impl SwitchManualIntElse_Opcode_Strval {
}
impl SwitchManualIntElse_Opcode_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl SwitchManualIntElse_Opcode_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
