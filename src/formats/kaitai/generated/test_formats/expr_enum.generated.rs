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
pub struct ExprEnum {
    pub(crate) _root: SharedType<ExprEnum>,
    pub(crate) _parent: SharedType<ExprEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_const_dog: Cell<bool>,
    const_dog: RefCell<ExprEnum_Animal>,
    f_derived_boom: Cell<bool>,
    derived_boom: RefCell<ExprEnum_Animal>,
    f_derived_dog: Cell<bool>,
    derived_dog: RefCell<ExprEnum_Animal>,
}
impl KStruct for ExprEnum {
    type Root = ExprEnum;
    type Parent = ExprEnum;

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
        *self_rc.one.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprEnum {
    pub fn const_dog(
        &self
    ) -> KResult<Ref<'_, ExprEnum_Animal>> {
        let _io = self._io.borrow();
        if self.f_const_dog.get() {
            return Ok(self.const_dog.borrow());
        }
        self.f_const_dog.set(true);
        *self.const_dog.borrow_mut() = i64::from(4).try_into()?;
        Ok(self.const_dog.borrow())
    }
    pub fn derived_boom(
        &self
    ) -> KResult<Ref<'_, ExprEnum_Animal>> {
        let _io = self._io.borrow();
        if self.f_derived_boom.get() {
            return Ok(self.derived_boom.borrow());
        }
        self.f_derived_boom.set(true);
        *self.derived_boom.borrow_mut() = i64::from(*self.one()).try_into()?;
        Ok(self.derived_boom.borrow())
    }
    pub fn derived_dog(
        &self
    ) -> KResult<Ref<'_, ExprEnum_Animal>> {
        let _io = self._io.borrow();
        if self.f_derived_dog.get() {
            return Ok(self.derived_dog.borrow());
        }
        self.f_derived_dog.set(true);
        *self.derived_dog.borrow_mut() = i64::from((i32::from(*self.one())).saturating_sub(98_i32)).try_into()?;
        Ok(self.derived_dog.borrow())
    }
}
impl ExprEnum {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}
impl ExprEnum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ExprEnum_Animal {
    Dog,
    Cat,
    Chicken,
    Boom,
    Unknown(i64),
}

impl TryFrom<i64> for ExprEnum_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ExprEnum_Animal> {
        match flag {
            4 => Ok(ExprEnum_Animal::Dog),
            7 => Ok(ExprEnum_Animal::Cat),
            12 => Ok(ExprEnum_Animal::Chicken),
            102 => Ok(ExprEnum_Animal::Boom),
            _ => Ok(ExprEnum_Animal::Unknown(flag)),
        }
    }
}

impl From<&ExprEnum_Animal> for i64 {
    fn from(v: &ExprEnum_Animal) -> Self {
        match *v {
            ExprEnum_Animal::Dog => 4,
            ExprEnum_Animal::Cat => 7,
            ExprEnum_Animal::Chicken => 12,
            ExprEnum_Animal::Boom => 102,
            ExprEnum_Animal::Unknown(v) => v
        }
    }
}

impl Default for ExprEnum_Animal {
    fn default() -> Self { ExprEnum_Animal::Unknown(0) }
}

