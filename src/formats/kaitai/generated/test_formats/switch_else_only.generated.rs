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
pub struct SwitchElseOnly {
    pub(crate) _root: SharedType<SwitchElseOnly>,
    pub(crate) _parent: SharedType<SwitchElseOnly>,
    pub(crate) _self_shared: SharedType<Self>,
    opcode: RefCell<i8>,
    prim_byte: RefCell<Option<SwitchElseOnly_PrimByte>>,
    indicator: RefCell<Vec<u8>>,
    ut: RefCell<Option<SwitchElseOnly_Ut>>,
    _io: RefCell<BytesReader>,
    indicator_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchElseOnly_PrimByte {
    S1(i8),
}
impl From<i8> for SwitchElseOnly_PrimByte {
    fn from(v: i8) -> Self {
        Self::S1(v)
    }
}
impl TryFrom<&SwitchElseOnly_PrimByte> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchElseOnly_PrimByte) -> Result<Self, Self::Error> {
        match e {
            SwitchElseOnly_PrimByte::S1(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchElseOnly_PrimByte> for i8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchElseOnly_PrimByte) -> Result<Self, Self::Error> {
        match e {
            SwitchElseOnly_PrimByte::S1(v) => Ok(i8::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchElseOnly_PrimByte> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchElseOnly_PrimByte) -> Result<Self, Self::Error> {
        match e {
            SwitchElseOnly_PrimByte::S1(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchElseOnly_PrimByte> for usize {
    type Error = KError;
    fn try_from(e: &SwitchElseOnly_PrimByte) -> Result<Self, Self::Error> {
        match e {
            SwitchElseOnly_PrimByte::S1(v) => Ok(usize::try_from(*v)?),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SwitchElseOnly_Ut {
    SwitchElseOnly_Data(OptRc<SwitchElseOnly_Data>),
}
impl TryFrom<&SwitchElseOnly_Ut> for OptRc<SwitchElseOnly_Data> {
    type Error = KError;
    fn try_from(v: &SwitchElseOnly_Ut) -> Result<Self, Self::Error> {
        if let SwitchElseOnly_Ut::SwitchElseOnly_Data(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchElseOnly_Data>> for SwitchElseOnly_Ut {
    fn from(v: OptRc<SwitchElseOnly_Data>) -> Self {
        Self::SwitchElseOnly_Data(v)
    }
}
impl KStruct for SwitchElseOnly {
    type Root = SwitchElseOnly;
    type Parent = SwitchElseOnly;

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
        *self_rc.opcode.borrow_mut() = _io.read_s1()?;
        match *self_rc.opcode() {
            _ => {
                *self_rc.prim_byte.borrow_mut() = Some(_io.read_s1()?.into());
            }
        }
        *self_rc.indicator.borrow_mut() = _io.read_bytes(4_usize)?;
        match self_rc.indicator().as_slice() {
            _ => {
                let t = Self::read_into::<_, SwitchElseOnly_Data>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.ut.borrow_mut() = Some(t);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchElseOnly {
}
impl SwitchElseOnly {
    pub fn opcode(&self) -> Ref<'_, i8> {
        self.opcode.borrow()
    }
}
impl SwitchElseOnly {
    pub fn prim_byte(&self) -> i8 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.prim_byte.borrow().as_ref().and_then(|v| i8::try_from(v).ok()).unwrap_or(0)
    }
    pub fn prim_byte_enum(&self) -> Ref<'_, Option<SwitchElseOnly_PrimByte>> {
        self.prim_byte.borrow()
    }
}
impl SwitchElseOnly {
    pub fn indicator(&self) -> Ref<'_, Vec<u8>> {
        self.indicator.borrow()
    }
}
impl SwitchElseOnly {
    pub fn ut(&self) -> Ref<'_, Option<SwitchElseOnly_Ut>> {
        self.ut.borrow()
    }
}
impl SwitchElseOnly {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchElseOnly {
    pub fn indicator_raw(&self) -> Ref<'_, Vec<u8>> {
        self.indicator_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchElseOnly_Data {
    pub(crate) _root: SharedType<SwitchElseOnly>,
    pub(crate) _parent: SharedType<SwitchElseOnly>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    value_raw: RefCell<Vec<u8>>,
}
impl KStruct for SwitchElseOnly_Data {
    type Root = SwitchElseOnly;
    type Parent = SwitchElseOnly;

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
        *self_rc.value.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchElseOnly_Data {
}
impl SwitchElseOnly_Data {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl SwitchElseOnly_Data {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchElseOnly_Data {
    pub fn value_raw(&self) -> Ref<'_, Vec<u8>> {
        self.value_raw.borrow()
    }
}
