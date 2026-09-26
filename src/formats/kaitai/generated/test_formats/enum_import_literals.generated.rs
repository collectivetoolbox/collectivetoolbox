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
use super::enum_deep::EnumDeep;
use super::enum_0::Enum0;

#[derive(Default, Debug, Clone)]
pub struct EnumImportLiterals {
    pub(crate) _root: SharedType<EnumImportLiterals>,
    pub(crate) _parent: SharedType<EnumImportLiterals>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_pet_1_eq: Cell<bool>,
    pet_1_eq: RefCell<bool>,
    f_pet_1_to_i: Cell<bool>,
    pet_1_to_i: RefCell<i32>,
    f_pet_2: Cell<bool>,
    pet_2: RefCell<i32>,
}
impl KStruct for EnumImportLiterals {
    type Root = EnumImportLiterals;
    type Parent = EnumImportLiterals;

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
impl EnumImportLiterals {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_1_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_pet_1_eq.get() {
            return Ok(self.pet_1_eq.borrow());
        }
        self.f_pet_1_eq.set(true);
        *self.pet_1_eq.borrow_mut() = (if true { EnumImportLiterals_Animal::Chicken } else { EnumImportLiterals_Animal::Dog } == EnumImportLiterals_Animal::Chicken).try_into()?;
        Ok(self.pet_1_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_1_to_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_pet_1_to_i.get() {
            return Ok(self.pet_1_to_i.borrow());
        }
        self.f_pet_1_to_i.set(true);
        *self.pet_1_to_i.borrow_mut() = (EnumImportLiterals_Animal::Cat.parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.pet_1_to_i.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pet_2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_pet_2.get() {
            return Ok(self.pet_2.borrow());
        }
        self.f_pet_2.set(true);
        *self.pet_2.borrow_mut() = (EnumImportLiterals_Animal::Hare).try_into()?;
        Ok(self.pet_2.borrow())
    }
}
impl EnumImportLiterals {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
