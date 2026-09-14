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
pub struct NestedSameName2 {
    pub(crate) _root: SharedType<NestedSameName2>,
    pub(crate) _parent: SharedType<NestedSameName2>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u32>,
    main_data: RefCell<OptRc<NestedSameName2_Main>>,
    dummy: RefCell<OptRc<NestedSameName2_DummyObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedSameName2 {
    type Root = NestedSameName2;
    type Parent = NestedSameName2;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.version.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, NestedSameName2_Main>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.main_data.borrow_mut() = t;
        let t = Self::read_into::<_, NestedSameName2_DummyObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.dummy.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedSameName2 {
}
impl NestedSameName2 {
    pub fn version(&self) -> Ref<'_, u32> {
        self.version.borrow()
    }
}
impl NestedSameName2 {
    pub fn main_data(&self) -> Ref<'_, OptRc<NestedSameName2_Main>> {
        self.main_data.borrow()
    }
}
impl NestedSameName2 {
    pub fn dummy(&self) -> Ref<'_, OptRc<NestedSameName2_DummyObj>> {
        self.dummy.borrow()
    }
}
impl NestedSameName2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedSameName2_DummyObj {
    pub(crate) _root: SharedType<NestedSameName2>,
    pub(crate) _parent: SharedType<NestedSameName2>,
    pub(crate) _self_shared: SharedType<Self>,
    dummy_size: RefCell<i32>,
    foo: RefCell<OptRc<NestedSameName2_DummyObj_FooObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedSameName2_DummyObj {
    type Root = NestedSameName2;
    type Parent = NestedSameName2;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.dummy_size.borrow_mut() = _io.read_s4le()?;
        let t = Self::read_into::<_, NestedSameName2_DummyObj_FooObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.foo.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedSameName2_DummyObj {
}
impl NestedSameName2_DummyObj {
    pub fn dummy_size(&self) -> Ref<'_, i32> {
        self.dummy_size.borrow()
    }
}
impl NestedSameName2_DummyObj {
    pub fn foo(&self) -> Ref<'_, OptRc<NestedSameName2_DummyObj_FooObj>> {
        self.foo.borrow()
    }
}
impl NestedSameName2_DummyObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedSameName2_DummyObj_FooObj {
    pub(crate) _root: SharedType<NestedSameName2>,
    pub(crate) _parent: SharedType<NestedSameName2_DummyObj>,
    pub(crate) _self_shared: SharedType<Self>,
    data2: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data2_raw: RefCell<Vec<u8>>,
}
impl KStruct for NestedSameName2_DummyObj_FooObj {
    type Root = NestedSameName2;
    type Parent = NestedSameName2_DummyObj;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.data2.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.dummy_size()).saturating_mul(2_i32))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedSameName2_DummyObj_FooObj {
}
impl NestedSameName2_DummyObj_FooObj {
    pub fn data2(&self) -> Ref<'_, Vec<u8>> {
        self.data2.borrow()
    }
}
impl NestedSameName2_DummyObj_FooObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl NestedSameName2_DummyObj_FooObj {
    pub fn data2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data2_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedSameName2_Main {
    pub(crate) _root: SharedType<NestedSameName2>,
    pub(crate) _parent: SharedType<NestedSameName2>,
    pub(crate) _self_shared: SharedType<Self>,
    main_size: RefCell<i32>,
    foo: RefCell<OptRc<NestedSameName2_Main_FooObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedSameName2_Main {
    type Root = NestedSameName2;
    type Parent = NestedSameName2;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.main_size.borrow_mut() = _io.read_s4le()?;
        let t = Self::read_into::<_, NestedSameName2_Main_FooObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.foo.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedSameName2_Main {
}
impl NestedSameName2_Main {
    pub fn main_size(&self) -> Ref<'_, i32> {
        self.main_size.borrow()
    }
}
impl NestedSameName2_Main {
    pub fn foo(&self) -> Ref<'_, OptRc<NestedSameName2_Main_FooObj>> {
        self.foo.borrow()
    }
}
impl NestedSameName2_Main {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedSameName2_Main_FooObj {
    pub(crate) _root: SharedType<NestedSameName2>,
    pub(crate) _parent: SharedType<NestedSameName2_Main>,
    pub(crate) _self_shared: SharedType<Self>,
    data1: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data1_raw: RefCell<Vec<u8>>,
}
impl KStruct for NestedSameName2_Main_FooObj {
    type Root = NestedSameName2;
    type Parent = NestedSameName2_Main;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.data1.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.main_size()).saturating_mul(2_i32))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedSameName2_Main_FooObj {
}
impl NestedSameName2_Main_FooObj {
    pub fn data1(&self) -> Ref<'_, Vec<u8>> {
        self.data1.borrow()
    }
}
impl NestedSameName2_Main_FooObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl NestedSameName2_Main_FooObj {
    pub fn data1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data1_raw.borrow()
    }
}
