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
use super::struct::Struct;

#[derive(Default, Debug, Clone)]
pub struct ExprIoTernary {
    pub(crate) _root: SharedType<ExprIoTernary>,
    pub(crate) _parent: SharedType<ExprIoTernary>,
    pub(crate) _self_shared: SharedType<Self>,
    flag: RefCell<u8>,
    obj1: RefCell<OptRc<ExprIoTernary_One>>,
    obj2: RefCell<OptRc<ExprIoTernary_Two>>,
    _io: RefCell<BytesReader>,
    obj1_raw: RefCell<Vec<u8>>,
    obj2_raw: RefCell<Vec<u8>>,
    f_one_or_two_io: Cell<bool>,
    one_or_two_io: RefCell<i32>,
    f_one_or_two_io_size1: Cell<bool>,
    one_or_two_io_size1: RefCell<i32>,
    f_one_or_two_io_size2: Cell<bool>,
    one_or_two_io_size2: RefCell<i32>,
    f_one_or_two_io_size_add_3: Cell<bool>,
    one_or_two_io_size_add_3: RefCell<i32>,
    f_one_or_two_obj: Cell<bool>,
    one_or_two_obj: RefCell<OptRc<Struct>>,
}
impl KStruct for ExprIoTernary {
    type Root = ExprIoTernary;
    type Parent = ExprIoTernary;

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
        *self_rc.flag.borrow_mut() = _io.read_u1()?;
        let _raw_obj1 = _io.read_bytes(4_usize)?;
        *self_rc.obj1_raw.borrow_mut() = _raw_obj1.clone();
        let _io_obj1 = BytesReader::from(_raw_obj1);
        let t = Self::read_into::<BytesReader, ExprIoTernary_One>(&_io_obj1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.obj1.borrow_mut() = t;
        let _raw_obj2 = _io.read_bytes(8_usize)?;
        *self_rc.obj2_raw.borrow_mut() = _raw_obj2.clone();
        let _io_obj2 = BytesReader::from(_raw_obj2);
        let t = Self::read_into::<BytesReader, ExprIoTernary_Two>(&_io_obj2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.obj2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprIoTernary {
    pub fn one_or_two_io(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_or_two_io.get() {
            return Ok(self.one_or_two_io.borrow());
        }
        self.f_one_or_two_io.set(true);
        *self.one_or_two_io.borrow_mut() = (if ((to_i128(*self.flag())) == (to_i128(64))) { self.obj1().clone() } else { self.obj2().clone() }._io()).try_into()?;
        Ok(self.one_or_two_io.borrow())
    }
    pub fn one_or_two_io_size1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_or_two_io_size1.get() {
            return Ok(self.one_or_two_io_size1.borrow());
        }
        self.f_one_or_two_io_size1.set(true);
        *self.one_or_two_io_size1.borrow_mut() = ((i64::try_from(if ((to_i128(*self.flag())) == (to_i128(64))) { self.obj1().clone() } else { self.obj2().clone() }._io().size())?)).try_into()?;
        Ok(self.one_or_two_io_size1.borrow())
    }
    pub fn one_or_two_io_size2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_or_two_io_size2.get() {
            return Ok(self.one_or_two_io_size2.borrow());
        }
        self.f_one_or_two_io_size2.set(true);
        *self.one_or_two_io_size2.borrow_mut() = ((i64::try_from(self.one_or_two_io()?.len())?)).try_into()?;
        Ok(self.one_or_two_io_size2.borrow())
    }
    pub fn one_or_two_io_size_add_3(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_or_two_io_size_add_3.get() {
            return Ok(self.one_or_two_io_size_add_3.borrow());
        }
        self.f_one_or_two_io_size_add_3.set(true);
        *self.one_or_two_io_size_add_3.borrow_mut() = (((i64::try_from(if ((to_i128(*self.flag())) == (to_i128(64))) { self.obj1().clone() } else { self.obj2().clone() }._io().size())?)).saturating_add(3_i32)).try_into()?;
        Ok(self.one_or_two_io_size_add_3.borrow())
    }
    pub fn one_or_two_obj(
        &self
    ) -> KResult<Ref<'_, OptRc<Struct>>> {
        let _io = self._io.borrow();
        if self.f_one_or_two_obj.get() {
            return Ok(self.one_or_two_obj.borrow());
        }
        *self.one_or_two_obj.borrow_mut() = if ((to_i128(*self.flag())) == (to_i128(64))) { self.obj1().clone() } else { self.obj2().clone() }.clone();
        Ok(self.one_or_two_obj.borrow())
    }
}
impl ExprIoTernary {
    pub fn flag(&self) -> Ref<'_, u8> {
        self.flag.borrow()
    }
}
impl ExprIoTernary {
    pub fn obj1(&self) -> Ref<'_, OptRc<ExprIoTernary_One>> {
        self.obj1.borrow()
    }
}
impl ExprIoTernary {
    pub fn obj2(&self) -> Ref<'_, OptRc<ExprIoTernary_Two>> {
        self.obj2.borrow()
    }
}
impl ExprIoTernary {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprIoTernary {
    pub fn obj1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.obj1_raw.borrow()
    }
}
impl ExprIoTernary {
    pub fn obj2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.obj2_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprIoTernary_One {
    pub(crate) _root: SharedType<ExprIoTernary>,
    pub(crate) _parent: SharedType<ExprIoTernary>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ExprIoTernary_One {
    type Root = ExprIoTernary;
    type Parent = ExprIoTernary;

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
impl ExprIoTernary_One {
}
impl ExprIoTernary_One {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}
impl ExprIoTernary_One {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprIoTernary_Two {
    pub(crate) _root: SharedType<ExprIoTernary>,
    pub(crate) _parent: SharedType<ExprIoTernary>,
    pub(crate) _self_shared: SharedType<Self>,
    two: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ExprIoTernary_Two {
    type Root = ExprIoTernary;
    type Parent = ExprIoTernary;

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
        *self_rc.two.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprIoTernary_Two {
}
impl ExprIoTernary_Two {
    pub fn two(&self) -> Ref<'_, u8> {
        self.two.borrow()
    }
}
impl ExprIoTernary_Two {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
