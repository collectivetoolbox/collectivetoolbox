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
pub struct SwitchManualStr {
    pub(crate) _root: SharedType<SwitchManualStr>,
    pub(crate) _parent: SharedType<SwitchManualStr>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchManualStr_Opcode>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStr {
    type Root = SwitchManualStr;
    type Parent = SwitchManualStr;

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
                let t = Self::read_into::<_, SwitchManualStr_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl SwitchManualStr {
}
impl SwitchManualStr {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchManualStr_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchManualStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStr_Opcode {
    pub(crate) _root: SharedType<SwitchManualStr>,
    pub(crate) _parent: SharedType<SwitchManualStr>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<String>,
    body: RefCell<Option<SwitchManualStr_Opcode_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualStr_Opcode_Body {
    SwitchManualStr_Opcode_Intval(OptRc<SwitchManualStr_Opcode_Intval>),
    SwitchManualStr_Opcode_Strval(OptRc<SwitchManualStr_Opcode_Strval>),
}
impl TryFrom<&SwitchManualStr_Opcode_Body> for OptRc<SwitchManualStr_Opcode_Intval> {
    type Error = KError;
    fn try_from(v: &SwitchManualStr_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualStr_Opcode_Body::SwitchManualStr_Opcode_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualStr_Opcode_Intval>> for SwitchManualStr_Opcode_Body {
    fn from(v: OptRc<SwitchManualStr_Opcode_Intval>) -> Self {
        Self::SwitchManualStr_Opcode_Intval(v)
    }
}
impl TryFrom<&SwitchManualStr_Opcode_Body> for OptRc<SwitchManualStr_Opcode_Strval> {
    type Error = KError;
    fn try_from(v: &SwitchManualStr_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualStr_Opcode_Body::SwitchManualStr_Opcode_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualStr_Opcode_Strval>> for SwitchManualStr_Opcode_Body {
    fn from(v: OptRc<SwitchManualStr_Opcode_Strval>) -> Self {
        Self::SwitchManualStr_Opcode_Strval(v)
    }
}
impl KStruct for SwitchManualStr_Opcode {
    type Root = SwitchManualStr;
    type Parent = SwitchManualStr;

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
        *self_rc.code.borrow_mut() = bytes_to_str(&_io.read_bytes(1_usize)?, "ASCII")?;
        match self_rc.code().as_str() {
            "I" => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualStr_Opcode_Intval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            "S" => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualStr_Opcode_Strval>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {}
        }
        Ok(())
    }
}
impl SwitchManualStr_Opcode {
}
impl SwitchManualStr_Opcode {
    pub fn code(&self) -> Ref<'_, String> {
        self.code.borrow()
    }
}
impl SwitchManualStr_Opcode {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualStr_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualStr_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchManualStr_Opcode {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStr_Opcode_Intval {
    pub(crate) _root: SharedType<SwitchManualStr>,
    pub(crate) _parent: SharedType<SwitchManualStr_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStr_Opcode_Intval {
    type Root = SwitchManualStr;
    type Parent = SwitchManualStr_Opcode;

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
impl SwitchManualStr_Opcode_Intval {
}
impl SwitchManualStr_Opcode_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl SwitchManualStr_Opcode_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStr_Opcode_Strval {
    pub(crate) _root: SharedType<SwitchManualStr>,
    pub(crate) _parent: SharedType<SwitchManualStr_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStr_Opcode_Strval {
    type Root = SwitchManualStr;
    type Parent = SwitchManualStr_Opcode;

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
impl SwitchManualStr_Opcode_Strval {
}
impl SwitchManualStr_Opcode_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl SwitchManualStr_Opcode_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
