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
pub struct BitsEnum {
    pub(crate) _root: SharedType<BitsEnum>,
    pub(crate) _parent: SharedType<BitsEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<BitsEnum_Animal>,
    two: RefCell<BitsEnum_Animal>,
    three: RefCell<BitsEnum_Animal>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BitsEnum {
    type Root = BitsEnum;
    type Parent = BitsEnum;

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
        *self_rc.one.borrow_mut() = i64::try_from(_io.read_bits_int_be(4)?)?.try_into()?;
        *self_rc.two.borrow_mut() = i64::try_from(_io.read_bits_int_be(8)?)?.try_into()?;
        *self_rc.three.borrow_mut() = i64::try_from(_io.read_bits_int_be(1)?)?.try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BitsEnum {
}
impl BitsEnum {
    pub fn one(&self) -> Ref<'_, BitsEnum_Animal> {
        self.one.borrow()
    }
}
impl BitsEnum {
    pub fn two(&self) -> Ref<'_, BitsEnum_Animal> {
        self.two.borrow()
    }
}
impl BitsEnum {
    pub fn three(&self) -> Ref<'_, BitsEnum_Animal> {
        self.three.borrow()
    }
}
impl BitsEnum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BitsEnum_Animal {
    Cat,
    Dog,
    Horse,
    Platypus,
    Unknown(i64),
}

impl TryFrom<i64> for BitsEnum_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<BitsEnum_Animal> {
        match flag {
            0 => Ok(BitsEnum_Animal::Cat),
            1 => Ok(BitsEnum_Animal::Dog),
            4 => Ok(BitsEnum_Animal::Horse),
            5 => Ok(BitsEnum_Animal::Platypus),
            _ => Ok(BitsEnum_Animal::Unknown(flag)),
        }
    }
}

impl From<&BitsEnum_Animal> for i64 {
    fn from(v: &BitsEnum_Animal) -> Self {
        match *v {
            BitsEnum_Animal::Cat => 0,
            BitsEnum_Animal::Dog => 1,
            BitsEnum_Animal::Horse => 4,
            BitsEnum_Animal::Platypus => 5,
            BitsEnum_Animal::Unknown(v) => v
        }
    }
}

impl Default for BitsEnum_Animal {
    fn default() -> Self { BitsEnum_Animal::Unknown(0) }
}

