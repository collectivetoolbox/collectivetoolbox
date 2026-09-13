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
pub struct ParamsPassStruct {
    pub(crate) _root: SharedType<ParamsPassStruct>,
    pub(crate) _parent: SharedType<ParamsPassStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<OptRc<ParamsPassStruct_Block>>,
    one: RefCell<OptRc<ParamsPassStruct_StructType>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassStruct {
    type Root = ParamsPassStruct;
    type Parent = ParamsPassStruct;

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
        let t = Self::read_into::<_, ParamsPassStruct_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.first.borrow_mut() = t;
        let f = |t : &mut ParamsPassStruct_StructType| Ok(t.set_params(self_rc.first().clone()));
        let t = Self::read_into_with_init::<_, ParamsPassStruct_StructType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.one.borrow_mut() = t;
        Ok(())
    }
}
impl ParamsPassStruct {
}
impl ParamsPassStruct {
    pub fn first(&self) -> Ref<'_, OptRc<ParamsPassStruct_Block>> {
        self.first.borrow()
    }
}
impl ParamsPassStruct {
    pub fn one(&self) -> Ref<'_, OptRc<ParamsPassStruct_StructType>> {
        self.one.borrow()
    }
}
impl ParamsPassStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassStruct_Block {
    pub(crate) _root: SharedType<ParamsPassStruct>,
    pub(crate) _parent: SharedType<ParamsPassStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassStruct_Block {
    type Root = ParamsPassStruct;
    type Parent = ParamsPassStruct;

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
        *self_rc.foo.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl ParamsPassStruct_Block {
}
impl ParamsPassStruct_Block {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl ParamsPassStruct_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassStruct_StructType {
    pub(crate) _root: SharedType<ParamsPassStruct>,
    pub(crate) _parent: SharedType<ParamsPassStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<OptRc<Struct>>,
    bar: RefCell<OptRc<ParamsPassStruct_StructType_Baz>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassStruct_StructType {
    type Root = ParamsPassStruct;
    type Parent = ParamsPassStruct;

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
        let f = |t : &mut ParamsPassStruct_StructType_Baz| Ok(t.set_params(self_rc.foo().clone()));
        let t = Self::read_into_with_init::<_, ParamsPassStruct_StructType_Baz>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.bar.borrow_mut() = t;
        Ok(())
    }
}
impl ParamsPassStruct_StructType {
    pub fn foo(&self) -> Ref<'_, OptRc<Struct>> {
        self.foo.borrow()
    }
}
impl ParamsPassStruct_StructType {
    pub fn set_params(&mut self, foo: OptRc<Struct>) {
        *self.foo.borrow_mut() = foo;
    }
}
impl ParamsPassStruct_StructType {
}
impl ParamsPassStruct_StructType {
    pub fn bar(&self) -> Ref<'_, OptRc<ParamsPassStruct_StructType_Baz>> {
        self.bar.borrow()
    }
}
impl ParamsPassStruct_StructType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassStruct_StructType_Baz {
    pub(crate) _root: SharedType<ParamsPassStruct>,
    pub(crate) _parent: SharedType<ParamsPassStruct_StructType>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<OptRc<Struct>>,
    qux: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassStruct_StructType_Baz {
    type Root = ParamsPassStruct;
    type Parent = ParamsPassStruct_StructType;

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
        *self_rc.qux.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl ParamsPassStruct_StructType_Baz {
    pub fn foo(&self) -> Ref<'_, OptRc<Struct>> {
        self.foo.borrow()
    }
}
impl ParamsPassStruct_StructType_Baz {
    pub fn set_params(&mut self, foo: OptRc<Struct>) {
        *self.foo.borrow_mut() = foo;
    }
}
impl ParamsPassStruct_StructType_Baz {
}
impl ParamsPassStruct_StructType_Baz {
    pub fn qux(&self) -> Ref<'_, u8> {
        self.qux.borrow()
    }
}
impl ParamsPassStruct_StructType_Baz {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
