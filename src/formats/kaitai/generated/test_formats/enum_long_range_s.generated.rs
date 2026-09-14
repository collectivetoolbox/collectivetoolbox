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
pub struct EnumLongRangeS {
    pub(crate) _root: SharedType<EnumLongRangeS>,
    pub(crate) _parent: SharedType<EnumLongRangeS>,
    pub(crate) _self_shared: SharedType<Self>,
    f1: RefCell<EnumLongRangeS_Constants>,
    f2: RefCell<EnumLongRangeS_Constants>,
    f3: RefCell<EnumLongRangeS_Constants>,
    f4: RefCell<EnumLongRangeS_Constants>,
    f5: RefCell<EnumLongRangeS_Constants>,
    f6: RefCell<EnumLongRangeS_Constants>,
    f7: RefCell<EnumLongRangeS_Constants>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumLongRangeS {
    type Root = EnumLongRangeS;
    type Parent = EnumLongRangeS;

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
        *self_rc.f1.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f2.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f3.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f4.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f5.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f6.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc.f7.borrow_mut() = _io.read_s8be()?.try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumLongRangeS {
}
impl EnumLongRangeS {
    pub fn f1(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f1.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f2(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f2.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f3(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f3.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f4(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f4.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f5(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f5.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f6(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f6.borrow()
    }
}
impl EnumLongRangeS {
    pub fn f7(&self) -> Ref<'_, EnumLongRangeS_Constants> {
        self.f7.borrow()
    }
}
impl EnumLongRangeS {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumLongRangeS_Constants {
    LongMin,
    IntBelowMin,
    IntMin,
    Zero,
    IntMax,
    IntOverMax,
    LongMax,
    Unknown(i64),
}

impl TryFrom<i64> for EnumLongRangeS_Constants {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumLongRangeS_Constants> {
        match flag {
            -9223372036854775808 => Ok(EnumLongRangeS_Constants::LongMin),
            -2147483649 => Ok(EnumLongRangeS_Constants::IntBelowMin),
            -2147483648 => Ok(EnumLongRangeS_Constants::IntMin),
            0 => Ok(EnumLongRangeS_Constants::Zero),
            2147483647 => Ok(EnumLongRangeS_Constants::IntMax),
            2147483648 => Ok(EnumLongRangeS_Constants::IntOverMax),
            9223372036854775807 => Ok(EnumLongRangeS_Constants::LongMax),
            _ => Ok(EnumLongRangeS_Constants::Unknown(flag)),
        }
    }
}

impl From<&EnumLongRangeS_Constants> for i64 {
    fn from(v: &EnumLongRangeS_Constants) -> Self {
        match *v {
            EnumLongRangeS_Constants::LongMin => -9223372036854775808,
            EnumLongRangeS_Constants::IntBelowMin => -2147483649,
            EnumLongRangeS_Constants::IntMin => -2147483648,
            EnumLongRangeS_Constants::Zero => 0,
            EnumLongRangeS_Constants::IntMax => 2147483647,
            EnumLongRangeS_Constants::IntOverMax => 2147483648,
            EnumLongRangeS_Constants::LongMax => 9223372036854775807,
            EnumLongRangeS_Constants::Unknown(v) => v
        }
    }
}

impl Default for EnumLongRangeS_Constants {
    fn default() -> Self { EnumLongRangeS_Constants::Unknown(0) }
}

