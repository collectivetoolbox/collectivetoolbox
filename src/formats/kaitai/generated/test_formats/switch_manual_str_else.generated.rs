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
pub struct SwitchManualStrElse {
    pub(crate) _root: SharedType<SwitchManualStrElse>,
    pub(crate) _parent: SharedType<SwitchManualStrElse>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchManualStrElse_Opcode>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStrElse {
    type Root = SwitchManualStrElse;
    type Parent = SwitchManualStrElse;

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
                let t = Self::read_into::<_, SwitchManualStrElse_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualStrElse {
}
impl SwitchManualStrElse {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchManualStrElse_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchManualStrElse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStrElse_Opcode {
    pub(crate) _root: SharedType<SwitchManualStrElse>,
    pub(crate) _parent: SharedType<SwitchManualStrElse>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<String>,
    body: RefCell<Option<SwitchManualStrElse_Opcode_Body>>,
    _io: RefCell<BytesReader>,
    code_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualStrElse_Opcode_Body {
    SwitchManualStrElse_Opcode_Intval(OptRc<SwitchManualStrElse_Opcode_Intval>),
    SwitchManualStrElse_Opcode_Strval(OptRc<SwitchManualStrElse_Opcode_Strval>),
    SwitchManualStrElse_Opcode_Noneval(OptRc<SwitchManualStrElse_Opcode_Noneval>),
}
impl TryFrom<&SwitchManualStrElse_Opcode_Body> for OptRc<SwitchManualStrElse_Opcode_Intval> {
    type Error = KError;
    fn try_from(v: &SwitchManualStrElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualStrElse_Opcode_Body::SwitchManualStrElse_Opcode_Intval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualStrElse_Opcode_Intval>> for SwitchManualStrElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualStrElse_Opcode_Intval>) -> Self {
        Self::SwitchManualStrElse_Opcode_Intval(v)
    }
}
impl TryFrom<&SwitchManualStrElse_Opcode_Body> for OptRc<SwitchManualStrElse_Opcode_Strval> {
    type Error = KError;
    fn try_from(v: &SwitchManualStrElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualStrElse_Opcode_Body::SwitchManualStrElse_Opcode_Strval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualStrElse_Opcode_Strval>> for SwitchManualStrElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualStrElse_Opcode_Strval>) -> Self {
        Self::SwitchManualStrElse_Opcode_Strval(v)
    }
}
impl TryFrom<&SwitchManualStrElse_Opcode_Body> for OptRc<SwitchManualStrElse_Opcode_Noneval> {
    type Error = KError;
    fn try_from(v: &SwitchManualStrElse_Opcode_Body) -> Result<Self, Self::Error> {
        if let SwitchManualStrElse_Opcode_Body::SwitchManualStrElse_Opcode_Noneval(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualStrElse_Opcode_Noneval>> for SwitchManualStrElse_Opcode_Body {
    fn from(v: OptRc<SwitchManualStrElse_Opcode_Noneval>) -> Self {
        Self::SwitchManualStrElse_Opcode_Noneval(v)
    }
}
impl KStruct for SwitchManualStrElse_Opcode {
    type Root = SwitchManualStrElse;
    type Parent = SwitchManualStrElse;

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
        *self_rc.code.borrow_mut() = bytes_to_str(&_io.read_bytes(1_usize)?, "ASCII")?;
        match self_rc.code().as_str() {
            "I" => {
                let t = Self::read_into::<_, SwitchManualStrElse_Opcode_Intval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            "S" => {
                let t = Self::read_into::<_, SwitchManualStrElse_Opcode_Strval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                let t = Self::read_into::<_, SwitchManualStrElse_Opcode_Noneval>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualStrElse_Opcode {
}
impl SwitchManualStrElse_Opcode {
    pub fn code(&self) -> Ref<'_, String> {
        self.code.borrow()
    }
}
impl SwitchManualStrElse_Opcode {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualStrElse_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualStrElse_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchManualStrElse_Opcode {
    pub fn code_raw(&self) -> Ref<'_, Vec<u8>> {
        self.code_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStrElse_Opcode_Intval {
    pub(crate) _root: SharedType<SwitchManualStrElse>,
    pub(crate) _parent: SharedType<SwitchManualStrElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStrElse_Opcode_Intval {
    type Root = SwitchManualStrElse;
    type Parent = SwitchManualStrElse_Opcode;

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
impl SwitchManualStrElse_Opcode_Intval {
}
impl SwitchManualStrElse_Opcode_Intval {
    pub fn value(&self) -> Ref<'_, u8> {
        self.value.borrow()
    }
}
impl SwitchManualStrElse_Opcode_Intval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStrElse_Opcode_Noneval {
    pub(crate) _root: SharedType<SwitchManualStrElse>,
    pub(crate) _parent: SharedType<SwitchManualStrElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    filler: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStrElse_Opcode_Noneval {
    type Root = SwitchManualStrElse;
    type Parent = SwitchManualStrElse_Opcode;

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
        *self_rc.filler.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualStrElse_Opcode_Noneval {
}
impl SwitchManualStrElse_Opcode_Noneval {
    pub fn filler(&self) -> Ref<'_, u32> {
        self.filler.borrow()
    }
}
impl SwitchManualStrElse_Opcode_Noneval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualStrElse_Opcode_Strval {
    pub(crate) _root: SharedType<SwitchManualStrElse>,
    pub(crate) _parent: SharedType<SwitchManualStrElse_Opcode>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualStrElse_Opcode_Strval {
    type Root = SwitchManualStrElse;
    type Parent = SwitchManualStrElse_Opcode;

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
impl SwitchManualStrElse_Opcode_Strval {
}
impl SwitchManualStrElse_Opcode_Strval {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl SwitchManualStrElse_Opcode_Strval {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
