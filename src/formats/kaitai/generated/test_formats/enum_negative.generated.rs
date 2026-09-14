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
pub struct EnumNegative {
    pub(crate) _root: SharedType<EnumNegative>,
    pub(crate) _parent: SharedType<EnumNegative>,
    pub(crate) _self_shared: SharedType<Self>,
    f1: RefCell<EnumNegative_Constants>,
    f2: RefCell<EnumNegative_Constants>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumNegative {
    type Root = EnumNegative;
    type Parent = EnumNegative;

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
        *self_rc.f1.borrow_mut() = i64::from(_io.read_s1()?).try_into()?;
        *self_rc.f2.borrow_mut() = i64::from(_io.read_s1()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumNegative {
}
impl EnumNegative {
    pub fn f1(&self) -> Ref<'_, EnumNegative_Constants> {
        self.f1.borrow()
    }
}
impl EnumNegative {
    pub fn f2(&self) -> Ref<'_, EnumNegative_Constants> {
        self.f2.borrow()
    }
}
impl EnumNegative {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumNegative_Constants {
    NegativeOne,
    PositiveOne,
    Unknown(i64),
}

impl TryFrom<i64> for EnumNegative_Constants {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumNegative_Constants> {
        match flag {
            -1 => Ok(EnumNegative_Constants::NegativeOne),
            1 => Ok(EnumNegative_Constants::PositiveOne),
            _ => Ok(EnumNegative_Constants::Unknown(flag)),
        }
    }
}

impl From<&EnumNegative_Constants> for i64 {
    fn from(v: &EnumNegative_Constants) -> Self {
        match *v {
            EnumNegative_Constants::NegativeOne => -1,
            EnumNegative_Constants::PositiveOne => 1,
            EnumNegative_Constants::Unknown(v) => v
        }
    }
}

impl Default for EnumNegative_Constants {
    fn default() -> Self { EnumNegative_Constants::Unknown(0) }
}

