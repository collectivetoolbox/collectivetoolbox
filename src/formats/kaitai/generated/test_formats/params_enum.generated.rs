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
pub struct ParamsEnum {
    pub(crate) _root: SharedType<ParamsEnum>,
    pub(crate) _parent: SharedType<ParamsEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<ParamsEnum_Animal>,
    invoke_with_param: RefCell<OptRc<ParamsEnum_WithParam>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsEnum {
    type Root = ParamsEnum;
    type Parent = ParamsEnum;

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
        *self_rc.one.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        let f = |t : &mut ParamsEnum_WithParam| Ok(t.set_params(*self_rc.one()));
        let t = Self::read_into_with_init::<_, ParamsEnum_WithParam>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.invoke_with_param.borrow_mut() = t;
        Ok(())
    }
}
impl ParamsEnum {
}
impl ParamsEnum {
    pub fn one(&self) -> Ref<'_, ParamsEnum_Animal> {
        self.one.borrow()
    }
}
impl ParamsEnum {
    pub fn invoke_with_param(&self) -> Ref<'_, OptRc<ParamsEnum_WithParam>> {
        self.invoke_with_param.borrow()
    }
}
impl ParamsEnum {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ParamsEnum_Animal {
    Dog,
    Cat,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for ParamsEnum_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ParamsEnum_Animal> {
        match flag {
            4 => Ok(ParamsEnum_Animal::Dog),
            7 => Ok(ParamsEnum_Animal::Cat),
            12 => Ok(ParamsEnum_Animal::Chicken),
            _ => Ok(ParamsEnum_Animal::Unknown(flag)),
        }
    }
}

impl From<&ParamsEnum_Animal> for i64 {
    fn from(v: &ParamsEnum_Animal) -> Self {
        match *v {
            ParamsEnum_Animal::Dog => 4,
            ParamsEnum_Animal::Cat => 7,
            ParamsEnum_Animal::Chicken => 12,
            ParamsEnum_Animal::Unknown(v) => v
        }
    }
}

impl Default for ParamsEnum_Animal {
    fn default() -> Self { ParamsEnum_Animal::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct ParamsEnum_WithParam {
    pub(crate) _root: SharedType<ParamsEnum>,
    pub(crate) _parent: SharedType<ParamsEnum>,
    pub(crate) _self_shared: SharedType<Self>,
    enumerated_one: RefCell<ParamsEnum_Animal>,
    _io: RefCell<BytesReader>,
    f_is_cat: Cell<bool>,
    is_cat: RefCell<bool>,
}
impl KStruct for ParamsEnum_WithParam {
    type Root = ParamsEnum;
    type Parent = ParamsEnum;

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
        Ok(())
    }
}
impl ParamsEnum_WithParam {
    pub fn enumerated_one(&self) -> Ref<'_, ParamsEnum_Animal> {
        self.enumerated_one.borrow()
    }
}
impl ParamsEnum_WithParam {
    pub fn set_params(&mut self, enumerated_one: ParamsEnum_Animal) {
        *self.enumerated_one.borrow_mut() = enumerated_one;
    }
}
impl ParamsEnum_WithParam {
    pub fn is_cat(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_cat.get() {
            return Ok(self.is_cat.borrow());
        }
        self.f_is_cat.set(true);
        *self.is_cat.borrow_mut() = (*self.enumerated_one() == ParamsEnum_Animal::Cat).try_into()?;
        Ok(self.is_cat.borrow())
    }
}
impl ParamsEnum_WithParam {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
