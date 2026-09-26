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
pub struct EnumFancy {
    pub(crate) _root: SharedType<EnumFancy>,
    pub(crate) _parent: SharedType<EnumFancy>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<EnumFancy_Animal>,
    pet_2: RefCell<EnumFancy_Animal>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumFancy {
    type Root = EnumFancy;
    type Parent = EnumFancy;

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
        *self_rc.pet_1.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.pet_2.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumFancy {
}
impl EnumFancy {
    pub fn pet_1(&self) -> Ref<'_, EnumFancy_Animal> {
        self.pet_1.borrow()
    }
}
impl EnumFancy {
    pub fn pet_2(&self) -> Ref<'_, EnumFancy_Animal> {
        self.pet_2.borrow()
    }
}
impl EnumFancy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumFancy_Animal {

    /**
     * A member of genus Canis.
     */
    Dog,

    /**
     * Small, typically furry, carnivorous mammal.
     */
    Cat,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for EnumFancy_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumFancy_Animal> {
        match flag {
            4 => Ok(EnumFancy_Animal::Dog),
            7 => Ok(EnumFancy_Animal::Cat),
            12 => Ok(EnumFancy_Animal::Chicken),
            _ => Ok(EnumFancy_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumFancy_Animal> for i64 {
    fn from(v: &EnumFancy_Animal) -> Self {
        match *v {
            EnumFancy_Animal::Dog => 4,
            EnumFancy_Animal::Cat => 7,
            EnumFancy_Animal::Chicken => 12,
            EnumFancy_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumFancy_Animal {
    fn default() -> Self { EnumFancy_Animal::Unknown(0) }
}

