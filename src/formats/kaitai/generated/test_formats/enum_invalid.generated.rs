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
pub struct EnumInvalid {
    pub(crate) _root: SharedType<EnumInvalid>,
    pub(crate) _parent: SharedType<EnumInvalid>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<EnumInvalid_Animal>,
    pet_2: RefCell<EnumInvalid_Animal>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumInvalid {
    type Root = EnumInvalid;
    type Parent = EnumInvalid;

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
        *self_rc.pet_1.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.pet_2.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumInvalid {
}
impl EnumInvalid {
    pub fn pet_1(&self) -> Ref<'_, EnumInvalid_Animal> {
        self.pet_1.borrow()
    }
}
impl EnumInvalid {
    pub fn pet_2(&self) -> Ref<'_, EnumInvalid_Animal> {
        self.pet_2.borrow()
    }
}
impl EnumInvalid {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumInvalid_Animal {
    Dog,
    Cat,
    Unknown(i64),
}

impl TryFrom<i64> for EnumInvalid_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumInvalid_Animal> {
        match flag {
            102 => Ok(EnumInvalid_Animal::Dog),
            124 => Ok(EnumInvalid_Animal::Cat),
            _ => Ok(EnumInvalid_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumInvalid_Animal> for i64 {
    fn from(v: &EnumInvalid_Animal) -> Self {
        match *v {
            EnumInvalid_Animal::Dog => 102,
            EnumInvalid_Animal::Cat => 124,
            EnumInvalid_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumInvalid_Animal {
    fn default() -> Self { EnumInvalid_Animal::Unknown(0) }
}

