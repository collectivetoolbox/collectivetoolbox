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
pub struct EnumIntRangeU {
    pub(crate) _root: SharedType<EnumIntRangeU>,
    pub(crate) _parent: SharedType<EnumIntRangeU>,
    pub(crate) _self_shared: SharedType<Self>,
    f1: RefCell<EnumIntRangeU_Constants>,
    f2: RefCell<EnumIntRangeU_Constants>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumIntRangeU {
    type Root = EnumIntRangeU;
    type Parent = EnumIntRangeU;

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
        *self_rc.f1.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc.f2.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumIntRangeU {
}
impl EnumIntRangeU {
    pub fn f1(&self) -> Ref<'_, EnumIntRangeU_Constants> {
        self.f1.borrow()
    }
}
impl EnumIntRangeU {
    pub fn f2(&self) -> Ref<'_, EnumIntRangeU_Constants> {
        self.f2.borrow()
    }
}
impl EnumIntRangeU {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumIntRangeU_Constants {
    Zero,
    IntMax,
    Unknown(i64),
}

impl TryFrom<i64> for EnumIntRangeU_Constants {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumIntRangeU_Constants> {
        match flag {
            0 => Ok(EnumIntRangeU_Constants::Zero),
            4294967295 => Ok(EnumIntRangeU_Constants::IntMax),
            _ => Ok(EnumIntRangeU_Constants::Unknown(flag)),
        }
    }
}

impl From<&EnumIntRangeU_Constants> for i64 {
    fn from(v: &EnumIntRangeU_Constants) -> Self {
        match *v {
            EnumIntRangeU_Constants::Zero => 0,
            EnumIntRangeU_Constants::IntMax => 4294967295,
            EnumIntRangeU_Constants::Unknown(v) => v
        }
    }
}

impl Default for EnumIntRangeU_Constants {
    fn default() -> Self { EnumIntRangeU_Constants::Unknown(0) }
}

