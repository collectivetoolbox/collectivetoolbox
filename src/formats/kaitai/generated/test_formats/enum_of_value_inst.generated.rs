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
pub struct EnumOfValueInst {
    pub(crate) _root: SharedType<EnumOfValueInst>,
    pub(crate) _parent: SharedType<EnumOfValueInst>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<EnumOfValueInst_Animal>,
    pet_2: RefCell<EnumOfValueInst_Animal>,
    _io: RefCell<BytesReader>,
    f_pet_3: Cell<bool>,
    pet_3: RefCell<EnumOfValueInst_Animal>,
    f_pet_4: Cell<bool>,
    pet_4: RefCell<EnumOfValueInst_Animal>,
}
impl KStruct for EnumOfValueInst {
    type Root = EnumOfValueInst;
    type Parent = EnumOfValueInst;

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
impl EnumOfValueInst {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_3(
        &self
    ) -> KResult<Ref<'_, EnumOfValueInst_Animal>> {
        let _io = self._io.borrow();
        if self.f_pet_3.get() {
            return Ok(self.pet_3.borrow());
        }
        self.f_pet_3.set(true);
        *self.pet_3.borrow_mut() = i64::from(if *self.pet_1() == EnumOfValueInst_Animal::Cat { 4_i32 } else { 12_i32 }).try_into()?;
        Ok(self.pet_3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_4(
        &self
    ) -> KResult<Ref<'_, EnumOfValueInst_Animal>> {
        let _io = self._io.borrow();
        if self.f_pet_4.get() {
            return Ok(self.pet_4.borrow());
        }
        self.f_pet_4.set(true);
        *self.pet_4.borrow_mut() = if *self.pet_1() == EnumOfValueInst_Animal::Cat { EnumOfValueInst_Animal::Dog.clone() } else { EnumOfValueInst_Animal::Chicken.clone() };
        Ok(self.pet_4.borrow())
    }
}
impl EnumOfValueInst {
    pub fn pet_1(&self) -> Ref<'_, EnumOfValueInst_Animal> {
        self.pet_1.borrow()
    }
}
impl EnumOfValueInst {
    pub fn pet_2(&self) -> Ref<'_, EnumOfValueInst_Animal> {
        self.pet_2.borrow()
    }
}
impl EnumOfValueInst {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumOfValueInst_Animal {
    Dog,
    Cat,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for EnumOfValueInst_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumOfValueInst_Animal> {
        match flag {
            4 => Ok(EnumOfValueInst_Animal::Dog),
            7 => Ok(EnumOfValueInst_Animal::Cat),
            12 => Ok(EnumOfValueInst_Animal::Chicken),
            _ => Ok(EnumOfValueInst_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumOfValueInst_Animal> for i64 {
    fn from(v: &EnumOfValueInst_Animal) -> Self {
        match *v {
            EnumOfValueInst_Animal::Dog => 4,
            EnumOfValueInst_Animal::Cat => 7,
            EnumOfValueInst_Animal::Chicken => 12,
            EnumOfValueInst_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumOfValueInst_Animal {
    fn default() -> Self { EnumOfValueInst_Animal::Unknown(0) }
}

