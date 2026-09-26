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
pub struct EnumDeepLiterals {
    pub(crate) _root: SharedType<EnumDeepLiterals>,
    pub(crate) _parent: SharedType<EnumDeepLiterals>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<EnumDeepLiterals_Container1_Animal>,
    pet_2: RefCell<EnumDeepLiterals_Container1_Container2_Animal>,
    _io: RefCell<BytesReader>,
    f_is_pet_1_ok: Cell<bool>,
    is_pet_1_ok: RefCell<bool>,
    f_is_pet_2_ok: Cell<bool>,
    is_pet_2_ok: RefCell<bool>,
}
impl KStruct for EnumDeepLiterals {
    type Root = EnumDeepLiterals;
    type Parent = EnumDeepLiterals;

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
impl EnumDeepLiterals {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_pet_1_ok(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_pet_1_ok.get() {
            return Ok(self.is_pet_1_ok.borrow());
        }
        self.f_is_pet_1_ok.set(true);
        *self.is_pet_1_ok.borrow_mut() = (*self.pet_1() == EnumDeepLiterals_Container1_Animal::Cat).try_into()?;
        Ok(self.is_pet_1_ok.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_pet_2_ok(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_pet_2_ok.get() {
            return Ok(self.is_pet_2_ok.borrow());
        }
        self.f_is_pet_2_ok.set(true);
        *self.is_pet_2_ok.borrow_mut() = (*self.pet_2() == EnumDeepLiterals_Container1_Animal::Hare).try_into()?;
        Ok(self.is_pet_2_ok.borrow())
    }
}
impl EnumDeepLiterals {
    pub fn pet_1(&self) -> Ref<'_, EnumDeepLiterals_Container1_Animal> {
        self.pet_1.borrow()
    }
}
impl EnumDeepLiterals {
    pub fn pet_2(&self) -> Ref<'_, EnumDeepLiterals_Container1_Container2_Animal> {
        self.pet_2.borrow()
    }
}
impl EnumDeepLiterals {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EnumDeepLiterals_Container1 {
    pub(crate) _root: SharedType<EnumDeepLiterals>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumDeepLiterals_Container1 {
    type Root = EnumDeepLiterals;
    type Parent = KStructUnit;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumDeepLiterals_Container1 {
}
impl EnumDeepLiterals_Container1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumDeepLiterals_Container1_Animal {
    Dog,
    Cat,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for EnumDeepLiterals_Container1_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumDeepLiterals_Container1_Animal> {
        match flag {
            4 => Ok(EnumDeepLiterals_Container1_Animal::Dog),
            7 => Ok(EnumDeepLiterals_Container1_Animal::Cat),
            12 => Ok(EnumDeepLiterals_Container1_Animal::Chicken),
            _ => Ok(EnumDeepLiterals_Container1_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumDeepLiterals_Container1_Animal> for i64 {
    fn from(v: &EnumDeepLiterals_Container1_Animal) -> Self {
        match *v {
            EnumDeepLiterals_Container1_Animal::Dog => 4,
            EnumDeepLiterals_Container1_Animal::Cat => 7,
            EnumDeepLiterals_Container1_Animal::Chicken => 12,
            EnumDeepLiterals_Container1_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumDeepLiterals_Container1_Animal {
    fn default() -> Self { EnumDeepLiterals_Container1_Animal::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct EnumDeepLiterals_Container1_Container2 {
    pub(crate) _root: SharedType<EnumDeepLiterals>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for EnumDeepLiterals_Container1_Container2 {
    type Root = EnumDeepLiterals;
    type Parent = KStructUnit;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EnumDeepLiterals_Container1_Container2 {
}
impl EnumDeepLiterals_Container1_Container2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EnumDeepLiterals_Container1_Container2_Animal {
    Canary,
    Turtle,
    Hare,
    Unknown(i64),
}

impl TryFrom<i64> for EnumDeepLiterals_Container1_Container2_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<EnumDeepLiterals_Container1_Container2_Animal> {
        match flag {
            4 => Ok(EnumDeepLiterals_Container1_Container2_Animal::Canary),
            7 => Ok(EnumDeepLiterals_Container1_Container2_Animal::Turtle),
            12 => Ok(EnumDeepLiterals_Container1_Container2_Animal::Hare),
            _ => Ok(EnumDeepLiterals_Container1_Container2_Animal::Unknown(flag)),
        }
    }
}

impl From<&EnumDeepLiterals_Container1_Container2_Animal> for i64 {
    fn from(v: &EnumDeepLiterals_Container1_Container2_Animal) -> Self {
        match *v {
            EnumDeepLiterals_Container1_Container2_Animal::Canary => 4,
            EnumDeepLiterals_Container1_Container2_Animal::Turtle => 7,
            EnumDeepLiterals_Container1_Container2_Animal::Hare => 12,
            EnumDeepLiterals_Container1_Container2_Animal::Unknown(v) => v
        }
    }
}

impl Default for EnumDeepLiterals_Container1_Container2_Animal {
    fn default() -> Self { EnumDeepLiterals_Container1_Container2_Animal::Unknown(0) }
}

