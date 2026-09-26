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
pub struct CombineEnum {
    pub(crate) _root: SharedType<CombineEnum>,
    pub(crate) _parent: SharedType<CombineEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    enum_u4: RefCell<CombineEnum_Animal>,
    enum_u2: RefCell<CombineEnum_Animal>,
    _io: RefCell<BytesReader>,
    f_enum_u4_u2: Cell<bool>,
    enum_u4_u2: RefCell<CombineEnum_Animal>,
}
impl KStruct for CombineEnum {
    type Root = CombineEnum;
    type Parent = CombineEnum;

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
        *self_rc.enum_u4.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.enum_u2.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CombineEnum {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn enum_u4_u2(
        &self
    ) -> KResult<Ref<'_, CombineEnum_Animal>> {
        let _io = self._io.borrow();
        if self.f_enum_u4_u2.get() {
            return Ok(self.enum_u4_u2.borrow());
        }
        self.f_enum_u4_u2.set(true);
        *self.enum_u4_u2.borrow_mut() = if false { self.enum_u4().clone() } else { self.enum_u2().clone() };
        Ok(self.enum_u4_u2.borrow())
    }
}
impl CombineEnum {
    pub fn enum_u4(&self) -> Ref<'_, CombineEnum_Animal> {
        self.enum_u4.borrow()
    }
}
impl CombineEnum {
    pub fn enum_u2(&self) -> Ref<'_, CombineEnum_Animal> {
        self.enum_u2.borrow()
    }
}
impl CombineEnum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CombineEnum_Animal {
    Pig,
    Horse,
    Unknown(i64),
}

impl TryFrom<i64> for CombineEnum_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<CombineEnum_Animal> {
        match flag {
            7 => Ok(CombineEnum_Animal::Pig),
            12 => Ok(CombineEnum_Animal::Horse),
            _ => Ok(CombineEnum_Animal::Unknown(flag)),
        }
    }
}

impl From<&CombineEnum_Animal> for i64 {
    fn from(v: &CombineEnum_Animal) -> Self {
        match *v {
            CombineEnum_Animal::Pig => 7,
            CombineEnum_Animal::Horse => 12,
            CombineEnum_Animal::Unknown(v) => v
        }
    }
}

impl Default for CombineEnum_Animal {
    fn default() -> Self { CombineEnum_Animal::Unknown(0) }
}

