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
pub struct EnumToIInvalid {
    pub(crate) _root: SharedType<EnumToIInvalid>,
    pub(crate) _parent: SharedType<EnumToIInvalid>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<EnumToIInvalid_Animal>,
    pet_2: RefCell<EnumToIInvalid_Animal>,
    _io: RefCell<BytesReader>,
    f_one_lt_two: Cell<bool>,
    one_lt_two: RefCell<bool>,
    f_pet_2_eq_int_f: Cell<bool>,
    pet_2_eq_int_f: RefCell<bool>,
    f_pet_2_eq_int_t: Cell<bool>,
    pet_2_eq_int_t: RefCell<bool>,
    f_pet_2_i: Cell<bool>,
    pet_2_i: RefCell<i32>,
    f_pet_2_i_to_s: Cell<bool>,
    pet_2_i_to_s: RefCell<String>,
    f_pet_2_mod: Cell<bool>,
    pet_2_mod: RefCell<i32>,
}
impl KStruct for EnumToIInvalid {
    type Root = EnumToIInvalid;
    type Parent = EnumToIInvalid;

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
        *self_rc.pet_1.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.pet_2.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumToIInvalid {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_lt_two(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_one_lt_two.get() {
            return Ok(self.one_lt_two.borrow());
        }
        self.f_one_lt_two.set(true);
        *self.one_lt_two.borrow_mut() = (i64::from(&*self.pet_1()) < i64::from(&*self.pet_2())).try_into()?;
        Ok(self.one_lt_two.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2_eq_int_f(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_pet_2_eq_int_f.get() {
            return Ok(self.pet_2_eq_int_f.borrow());
        }
        self.f_pet_2_eq_int_f.set(true);
        *self.pet_2_eq_int_f.borrow_mut() = (((to_i128(i64::from(&*self.pet_2()))) == (to_i128(110)))).try_into()?;
        Ok(self.pet_2_eq_int_f.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2_eq_int_t(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_pet_2_eq_int_t.get() {
            return Ok(self.pet_2_eq_int_t.borrow());
        }
        self.f_pet_2_eq_int_t.set(true);
        *self.pet_2_eq_int_t.borrow_mut() = (((to_i128(i64::from(&*self.pet_2()))) == (to_i128(111)))).try_into()?;
        Ok(self.pet_2_eq_int_t.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_pet_2_i.get() {
            return Ok(self.pet_2_i.borrow());
        }
        self.f_pet_2_i.set(true);
        *self.pet_2_i.borrow_mut() = (i64::from(&*self.pet_2())).try_into()?;
        Ok(self.pet_2_i.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2_i_to_s(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_pet_2_i_to_s.get() {
            return Ok(self.pet_2_i_to_s.borrow());
        }
        self.f_pet_2_i_to_s.set(true);
        *self.pet_2_i_to_s.borrow_mut() = i64::from(&*self.pet_2()).to_string().to_string();
        Ok(self.pet_2_i_to_s.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2_mod(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_pet_2_mod.get() {
            return Ok(self.pet_2_mod.borrow());
        }
        self.f_pet_2_mod.set(true);
        *self.pet_2_mod.borrow_mut() = ((i64::from(&*self.pet_2())).saturating_add(32768_i64)).try_into()?;
        Ok(self.pet_2_mod.borrow())
    }
}
impl EnumToIInvalid {
    pub fn pet_1(&self) -> Ref<'_, EnumToIInvalid_Animal> {
        self.pet_1.borrow()
    }
}
impl EnumToIInvalid {
    pub fn pet_2(&self) -> Ref<'_, EnumToIInvalid_Animal> {
        self.pet_2.borrow()
    }
}
impl EnumToIInvalid {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumToIInvalid_Animal {
    Dog,
    Cat,
    Unknown(i64),
}

impl TryFrom<i64> for EnumToIInvalid_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumToIInvalid_Animal> {
        match flag {
            102 => Ok(EnumToIInvalid_Animal::Dog),
            124 => Ok(EnumToIInvalid_Animal::Cat),
            _ => Ok(EnumToIInvalid_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumToIInvalid_Animal> for i64 {
    fn from(v: &EnumToIInvalid_Animal) -> Self {
        match *v {
            EnumToIInvalid_Animal::Dog => 102,
            EnumToIInvalid_Animal::Cat => 124,
            EnumToIInvalid_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumToIInvalid_Animal {
    fn default() -> Self { EnumToIInvalid_Animal::Unknown(0) }
}

