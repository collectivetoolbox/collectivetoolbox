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
pub struct EnumLongRangeU {
    pub(crate) _root: SharedType<EnumLongRangeU>,
    pub(crate) _parent: SharedType<EnumLongRangeU>,
    pub(crate) _self_shared: SharedType<Self>,
    f1: RefCell<EnumLongRangeU_Constants>,
    f2: RefCell<EnumLongRangeU_Constants>,
    f3: RefCell<EnumLongRangeU_Constants>,
    f4: RefCell<EnumLongRangeU_Constants>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumLongRangeU {
    type Root = EnumLongRangeU;
    type Parent = EnumLongRangeU;

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
        *self_rc.f1.borrow_mut() = i64::try_from(_io.read_u8be()?)?.try_into()?;
        *self_rc.f2.borrow_mut() = i64::try_from(_io.read_u8be()?)?.try_into()?;
        *self_rc.f3.borrow_mut() = i64::try_from(_io.read_u8be()?)?.try_into()?;
        *self_rc.f4.borrow_mut() = i64::try_from(_io.read_u8be()?)?.try_into()?;
        Ok(())
    }
}
impl EnumLongRangeU {
}
impl EnumLongRangeU {
    pub fn f1(&self) -> Ref<'_, EnumLongRangeU_Constants> {
        self.f1.borrow()
    }
}
impl EnumLongRangeU {
    pub fn f2(&self) -> Ref<'_, EnumLongRangeU_Constants> {
        self.f2.borrow()
    }
}
impl EnumLongRangeU {
    pub fn f3(&self) -> Ref<'_, EnumLongRangeU_Constants> {
        self.f3.borrow()
    }
}
impl EnumLongRangeU {
    pub fn f4(&self) -> Ref<'_, EnumLongRangeU_Constants> {
        self.f4.borrow()
    }
}
impl EnumLongRangeU {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumLongRangeU_Constants {
    LongMax,
    IntMax,
    IntOverMax,
    Unknown(i64),
}

impl TryFrom<i64> for EnumLongRangeU_Constants {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumLongRangeU_Constants> {
        match flag {
            0 => Ok(EnumLongRangeU_Constants::LongMax),
            4294967295 => Ok(EnumLongRangeU_Constants::IntMax),
            4294967296 => Ok(EnumLongRangeU_Constants::IntOverMax),
            _ => Ok(EnumLongRangeU_Constants::Unknown(flag)),
        }
    }
}

impl From<&EnumLongRangeU_Constants> for i64 {
    fn from(v: &EnumLongRangeU_Constants) -> Self {
        match *v {
            EnumLongRangeU_Constants::LongMax => 0,
            EnumLongRangeU_Constants::IntMax => 4294967295,
            EnumLongRangeU_Constants::IntOverMax => 4294967296,
            EnumLongRangeU_Constants::Unknown(v) => v
        }
    }
}

impl Default for EnumLongRangeU_Constants {
    fn default() -> Self { EnumLongRangeU_Constants::Unknown(0) }
}

