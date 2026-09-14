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
pub struct Enum1 {
    pub(crate) _root: SharedType<Enum1>,
    pub(crate) _parent: SharedType<Enum1>,
    pub(crate) _self_shared: SharedType<Self>,
    main: RefCell<OptRc<Enum1_MainObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Enum1 {
    type Root = Enum1;
    type Parent = Enum1;

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
        let t = Self::read_into::<_, Enum1_MainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Enum1 {
}
impl Enum1 {
    pub fn main(&self) -> Ref<'_, OptRc<Enum1_MainObj>> {
        self.main.borrow()
    }
}
impl Enum1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Enum1_MainObj {
    pub(crate) _root: SharedType<Enum1>,
    pub(crate) _parent: SharedType<Enum1>,
    pub(crate) _self_shared: SharedType<Self>,
    submain: RefCell<OptRc<Enum1_MainObj_SubmainObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Enum1_MainObj {
    type Root = Enum1;
    type Parent = Enum1;

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
        let t = Self::read_into::<_, Enum1_MainObj_SubmainObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.submain.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Enum1_MainObj {
}
impl Enum1_MainObj {
    pub fn submain(&self) -> Ref<'_, OptRc<Enum1_MainObj_SubmainObj>> {
        self.submain.borrow()
    }
}
impl Enum1_MainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Enum1_MainObj_Animal {
    Dog,
    Cat,
    Chicken,
    Unknown(i64),
}

impl TryFrom<i64> for Enum1_MainObj_Animal {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Enum1_MainObj_Animal> {
        match flag {
            4 => Ok(Enum1_MainObj_Animal::Dog),
            7 => Ok(Enum1_MainObj_Animal::Cat),
            12 => Ok(Enum1_MainObj_Animal::Chicken),
            _ => Ok(Enum1_MainObj_Animal::Unknown(flag)),
        }
    }
}

impl From<&Enum1_MainObj_Animal> for i64 {
    fn from(v: &Enum1_MainObj_Animal) -> Self {
        match *v {
            Enum1_MainObj_Animal::Dog => 4,
            Enum1_MainObj_Animal::Cat => 7,
            Enum1_MainObj_Animal::Chicken => 12,
            Enum1_MainObj_Animal::Unknown(v) => v
        }
    }
}

impl Default for Enum1_MainObj_Animal {
    fn default() -> Self { Enum1_MainObj_Animal::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Enum1_MainObj_SubmainObj {
    pub(crate) _root: SharedType<Enum1>,
    pub(crate) _parent: SharedType<Enum1_MainObj>,
    pub(crate) _self_shared: SharedType<Self>,
    pet_1: RefCell<Enum1_MainObj_Animal>,
    pet_2: RefCell<Enum1_MainObj_Animal>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Enum1_MainObj_SubmainObj {
    type Root = Enum1;
    type Parent = Enum1_MainObj;

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
impl Enum1_MainObj_SubmainObj {
}
impl Enum1_MainObj_SubmainObj {
    pub fn pet_1(&self) -> Ref<'_, Enum1_MainObj_Animal> {
        self.pet_1.borrow()
    }
}
impl Enum1_MainObj_SubmainObj {
    pub fn pet_2(&self) -> Ref<'_, Enum1_MainObj_Animal> {
        self.pet_2.borrow()
    }
}
impl Enum1_MainObj_SubmainObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
