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
pub struct SwitchIntegers {
    pub(crate) _root: SharedType<SwitchIntegers>,
    pub(crate) _parent: SharedType<SwitchIntegers>,
    pub(crate) _self_shared: SharedType<Self>,
    opcodes: RefCell<Vec<OptRc<SwitchIntegers_Opcode>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchIntegers {
    type Root = SwitchIntegers;
    type Parent = SwitchIntegers;

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
        *self_rc.opcodes.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, SwitchIntegers_Opcode>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.opcodes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchIntegers {
}
impl SwitchIntegers {
    pub fn opcodes(&self) -> Ref<'_, Vec<OptRc<SwitchIntegers_Opcode>>> {
        self.opcodes.borrow()
    }
}
impl SwitchIntegers {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchIntegers_Opcode {
    pub(crate) _root: SharedType<SwitchIntegers>,
    pub(crate) _parent: SharedType<SwitchIntegers>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    body: RefCell<Option<SwitchIntegers_Opcode_Body>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum SwitchIntegers_Opcode_Body {
    U1(u8),
    U2(u16),
    U4(u32),
    U8(u64),
}
impl From<u8> for SwitchIntegers_Opcode_Body {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<u16> for SwitchIntegers_Opcode_Body {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<u32> for SwitchIntegers_Opcode_Body {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl From<u64> for SwitchIntegers_Opcode_Body {
    fn from(v: u64) -> Self {
        Self::U8(v)
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for u16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for u32 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(u32::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for u8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(u8::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers_Opcode_Body> for usize {
    type Error = KError;
    fn try_from(e: &SwitchIntegers_Opcode_Body) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers_Opcode_Body::U1(v) => Ok(usize::from(*v)),
            SwitchIntegers_Opcode_Body::U2(v) => Ok(usize::from(*v)),
            SwitchIntegers_Opcode_Body::U4(v) => Ok(usize::try_from(*v)?),
            SwitchIntegers_Opcode_Body::U8(v) => Ok(usize::try_from(*v)?),
        }
    }
}

impl KStruct for SwitchIntegers_Opcode {
    type Root = SwitchIntegers;
    type Parent = SwitchIntegers;

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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        match *self_rc.code() {
            1 => {
                *self_rc.body.borrow_mut() = Some(_io.read_u1()?.into());
            }
            2 => {
                *self_rc.body.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            4 => {
                *self_rc.body.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            8 => {
                *self_rc.body.borrow_mut() = Some(_io.read_u8le()?.into());
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchIntegers_Opcode {
}
impl SwitchIntegers_Opcode {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchIntegers_Opcode {
    pub fn body(&self) -> u64 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.body.borrow().as_ref().and_then(|v| u64::try_from(v).ok()).unwrap_or(0)
    }
    pub fn body_enum(&self) -> Ref<'_, Option<SwitchIntegers_Opcode_Body>> {
        self.body.borrow()
    }
}
impl SwitchIntegers_Opcode {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
