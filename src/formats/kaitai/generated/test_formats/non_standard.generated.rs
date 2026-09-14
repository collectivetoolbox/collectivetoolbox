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
pub struct NonStandard {
    pub(crate) _root: SharedType<NonStandard>,
    pub(crate) _parent: SharedType<NonStandard>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    bar: RefCell<Option<NonStandard_Bar>>,
    _io: RefCell<BytesReader>,
    f_pi: Cell<bool>,
    pi: RefCell<u8>,
    f_vi: Cell<bool>,
    vi: RefCell<u8>,
}
#[derive(Debug, Clone)]
pub enum NonStandard_Bar {
    U2(u16),
    U4(u32),
}
impl From<u16> for NonStandard_Bar {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<u32> for NonStandard_Bar {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl TryFrom<&NonStandard_Bar> for i64 {
    type Error = KError;
    fn try_from(e: &NonStandard_Bar) -> Result<Self, Self::Error> {
        match e {
            NonStandard_Bar::U2(v) => Ok(i64::try_from(*v)?),
            NonStandard_Bar::U4(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&NonStandard_Bar> for u16 {
    type Error = KError;
    fn try_from(e: &NonStandard_Bar) -> Result<Self, Self::Error> {
        match e {
            NonStandard_Bar::U2(v) => Ok(u16::try_from(*v)?),
            NonStandard_Bar::U4(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&NonStandard_Bar> for u32 {
    type Error = KError;
    fn try_from(e: &NonStandard_Bar) -> Result<Self, Self::Error> {
        match e {
            NonStandard_Bar::U2(v) => Ok(u32::try_from(*v)?),
            NonStandard_Bar::U4(v) => Ok(u32::try_from(*v)?),
        }
    }
}
impl TryFrom<&NonStandard_Bar> for u64 {
    type Error = KError;
    fn try_from(e: &NonStandard_Bar) -> Result<Self, Self::Error> {
        match e {
            NonStandard_Bar::U2(v) => Ok(u64::try_from(*v)?),
            NonStandard_Bar::U4(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&NonStandard_Bar> for usize {
    type Error = KError;
    fn try_from(e: &NonStandard_Bar) -> Result<Self, Self::Error> {
        match e {
            NonStandard_Bar::U2(v) => Ok(usize::from(*v)),
            NonStandard_Bar::U4(v) => Ok(usize::try_from(*v)?),
        }
    }
}

impl KStruct for NonStandard {
    type Root = NonStandard;
    type Parent = NonStandard;

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
        *self_rc.foo.borrow_mut() = _io.read_u1()?;
        match *self_rc.foo() {
            42 => {
                *self_rc.bar.borrow_mut() = Some(_io.read_u2le()?.into());
            }
            43 => {
                *self_rc.bar.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NonStandard {
    pub fn pi(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_pi.get() {
            return Ok(self.pi.borrow());
        }
        self.f_pi.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.pi.borrow_mut() = _io.read_u1()?;
        _io.seek(_pos)?;
        Ok(self.pi.borrow())
    }
    pub fn vi(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_vi.get() {
            return Ok(self.vi.borrow());
        }
        self.f_vi.set(true);
        *self.vi.borrow_mut() = (*self.foo()).try_into()?;
        Ok(self.vi.borrow())
    }
}
impl NonStandard {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl NonStandard {
    pub fn bar(&self) -> u32 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.bar.borrow().as_ref().and_then(|v| u32::try_from(v).ok()).unwrap_or(0)
    }
    pub fn bar_enum(&self) -> Ref<'_, Option<NonStandard_Bar>> {
        self.bar.borrow()
    }
}
impl NonStandard {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
