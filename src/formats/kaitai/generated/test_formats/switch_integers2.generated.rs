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
pub struct SwitchIntegers2 {
    pub(crate) _root: SharedType<SwitchIntegers2>,
    pub(crate) _parent: SharedType<SwitchIntegers2>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    len: RefCell<Option<SwitchIntegers2_Len>>,
    ham: RefCell<Vec<u8>>,
    padding: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_len_mod_str: Cell<bool>,
    len_mod_str: RefCell<String>,
}
#[derive(Debug, Clone)]
pub enum SwitchIntegers2_Len {
    U1(u8),
    U2(u16),
    U4(u32),
    U8(u64),
}
impl From<u8> for SwitchIntegers2_Len {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<u16> for SwitchIntegers2_Len {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<u32> for SwitchIntegers2_Len {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl From<u64> for SwitchIntegers2_Len {
    fn from(v: u64) -> Self {
        Self::U8(v)
    }
}
impl TryFrom<&SwitchIntegers2_Len> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers2_Len::U2(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers2_Len::U4(v) => Ok(i64::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers2_Len> for u16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers2_Len::U2(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers2_Len::U4(v) => Ok(u16::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers2_Len> for u32 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers2_Len::U2(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers2_Len::U4(v) => Ok(u32::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(u32::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers2_Len> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers2_Len::U2(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers2_Len::U4(v) => Ok(u64::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers2_Len> for u8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers2_Len::U2(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers2_Len::U4(v) => Ok(u8::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(u8::try_from(*v)?),
        }
    }
}
impl TryFrom<&SwitchIntegers2_Len> for usize {
    type Error = KError;
    fn try_from(e: &SwitchIntegers2_Len) -> Result<Self, Self::Error> {
        match e {
            SwitchIntegers2_Len::U1(v) => Ok(usize::from(*v)),
            SwitchIntegers2_Len::U2(v) => Ok(usize::from(*v)),
            SwitchIntegers2_Len::U4(v) => Ok(usize::try_from(*v)?),
            SwitchIntegers2_Len::U8(v) => Ok(usize::try_from(*v)?),
        }
    }
}

impl KStruct for SwitchIntegers2 {
    type Root = SwitchIntegers2;
    type Parent = SwitchIntegers2;

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
            1 => {
                *self_rc.len.borrow_mut() = Some(_io.read_u1()?.into());
            }
            2 => {
                *self_rc.len.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            4 => {
                *self_rc.len.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            8 => {
                *self_rc.len.borrow_mut() = Some(_io.read_u8le()?.into());
            }
            _ => {}
        }
        *self_rc.ham.borrow_mut() = _io.read_bytes(usize::try_from(self_rc.len())?)?;
        if ((to_i128(self_rc.len())) > (to_i128(3))) {
            *self_rc.padding.borrow_mut() = _io.read_u1()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchIntegers2 {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn len_mod_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_len_mod_str.get() {
            return Ok(self.len_mod_str.borrow());
        }
        self.f_len_mod_str.set(true);
        *self.len_mod_str.borrow_mut() = ((self.len()).saturating_mul(2_u64)).saturating_sub(1_u64).to_string().to_string();
        Ok(self.len_mod_str.borrow())
    }
}
impl SwitchIntegers2 {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchIntegers2 {
    pub fn len(&self) -> u64 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.len.borrow().as_ref().and_then(|v| u64::try_from(v).ok()).unwrap_or(0)
    }
    pub fn len_enum(&self) -> Ref<'_, Option<SwitchIntegers2_Len>> {
        self.len.borrow()
    }
}
impl SwitchIntegers2 {
    pub fn ham(&self) -> Ref<'_, Vec<u8>> {
        self.ham.borrow()
    }
}
impl SwitchIntegers2 {
    pub fn padding(&self) -> Ref<'_, u8> {
        self.padding.borrow()
    }
}
impl SwitchIntegers2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
