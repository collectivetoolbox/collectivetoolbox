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
pub struct ParamsPassArrayStruct {
    pub(crate) _root: SharedType<ParamsPassArrayStruct>,
    pub(crate) _parent: SharedType<ParamsPassArrayStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<OptRc<ParamsPassArrayStruct_Foo>>,
    two: RefCell<OptRc<ParamsPassArrayStruct_Bar>>,
    pass_structs: RefCell<OptRc<ParamsPassArrayStruct_StructType>>,
    _io: RefCell<BytesReader>,
    f_one_two: Cell<bool>,
    one_two: RefCell<Vec<OptRc<Struct>>>,
}
impl KStruct for ParamsPassArrayStruct {
    type Root = ParamsPassArrayStruct;
    type Parent = ParamsPassArrayStruct;

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
        let t = Self::read_into::<_, ParamsPassArrayStruct_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.one.borrow_mut() = t;
        let t = Self::read_into::<_, ParamsPassArrayStruct_Bar>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.two.borrow_mut() = t;
        let f = |t : &mut ParamsPassArrayStruct_StructType| Ok(t.set_params(*self_rc.one_two()?.clone()));
        let t = Self::read_into_with_init::<_, ParamsPassArrayStruct_StructType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_structs.borrow_mut() = t;
        Ok(())
    }
}
impl ParamsPassArrayStruct {
    pub fn one_two(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<Struct>>>> {
        let _io = self._io.borrow();
        if self.f_one_two.get() {
            return Ok(self.one_two.borrow());
        }
        self.f_one_two.set(true);
        *self.one_two.borrow_mut() = vec![self.one(), self.two()];
        Ok(self.one_two.borrow())
    }
}
impl ParamsPassArrayStruct {
    pub fn one(&self) -> Ref<'_, OptRc<ParamsPassArrayStruct_Foo>> {
        self.one.borrow()
    }
}
impl ParamsPassArrayStruct {
    pub fn two(&self) -> Ref<'_, OptRc<ParamsPassArrayStruct_Bar>> {
        self.two.borrow()
    }
}
impl ParamsPassArrayStruct {
    pub fn pass_structs(&self) -> Ref<'_, OptRc<ParamsPassArrayStruct_StructType>> {
        self.pass_structs.borrow()
    }
}
impl ParamsPassArrayStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassArrayStruct_Bar {
    pub(crate) _root: SharedType<ParamsPassArrayStruct>,
    pub(crate) _parent: SharedType<ParamsPassArrayStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    b: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassArrayStruct_Bar {
    type Root = ParamsPassArrayStruct;
    type Parent = ParamsPassArrayStruct;

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
        *self_rc.b.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl ParamsPassArrayStruct_Bar {
}
impl ParamsPassArrayStruct_Bar {
    pub fn b(&self) -> Ref<'_, u8> {
        self.b.borrow()
    }
}
impl ParamsPassArrayStruct_Bar {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassArrayStruct_Foo {
    pub(crate) _root: SharedType<ParamsPassArrayStruct>,
    pub(crate) _parent: SharedType<ParamsPassArrayStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    f: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassArrayStruct_Foo {
    type Root = ParamsPassArrayStruct;
    type Parent = ParamsPassArrayStruct;

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
        *self_rc.f.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl ParamsPassArrayStruct_Foo {
}
impl ParamsPassArrayStruct_Foo {
    pub fn f(&self) -> Ref<'_, u8> {
        self.f.borrow()
    }
}
impl ParamsPassArrayStruct_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassArrayStruct_StructType {
    pub(crate) _root: SharedType<ParamsPassArrayStruct>,
    pub(crate) _parent: SharedType<ParamsPassArrayStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    structs: RefCell<OptRc<Struct>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassArrayStruct_StructType {
    type Root = ParamsPassArrayStruct;
    type Parent = ParamsPassArrayStruct;

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
impl ParamsPassArrayStruct_StructType {
    pub fn structs(&self) -> Ref<'_, OptRc<Struct>> {
        self.structs.borrow()
    }
}
impl ParamsPassArrayStruct_StructType {
    pub fn set_params(&mut self, structs: OptRc<Struct>) {
        *self.structs.borrow_mut() = structs;
    }
}
impl ParamsPassArrayStruct_StructType {
}
impl ParamsPassArrayStruct_StructType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
