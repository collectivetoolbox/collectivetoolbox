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
pub struct NestedTypes2 {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<NestedTypes2>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<OptRc<NestedTypes2_SubtypeA>>,
    two: RefCell<OptRc<NestedTypes2_SubtypeB>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2 {
    type Root = NestedTypes2;
    type Parent = NestedTypes2;

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
        let t = Self::read_into::<_, NestedTypes2_SubtypeA>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.one.borrow_mut() = t;
        let t = Self::read_into::<_, NestedTypes2_SubtypeB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.two.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2 {
}
impl NestedTypes2 {
    pub fn one(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeA>> {
        self.one.borrow()
    }
}
impl NestedTypes2 {
    pub fn two(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeB>> {
        self.two.borrow()
    }
}
impl NestedTypes2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedTypes2_SubtypeA {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<NestedTypes2>,
    pub(crate) _self_shared: SharedType<Self>,
    typed_at_root: RefCell<OptRc<NestedTypes2_SubtypeB>>,
    typed_here1: RefCell<OptRc<NestedTypes2_SubtypeA_SubtypeC>>,
    typed_here2: RefCell<OptRc<NestedTypes2_SubtypeA_SubtypeCc>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2_SubtypeA {
    type Root = NestedTypes2;
    type Parent = NestedTypes2;

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
        let t = Self::read_into::<_, NestedTypes2_SubtypeB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.typed_at_root.borrow_mut() = t;
        let t = Self::read_into::<_, NestedTypes2_SubtypeA_SubtypeC>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.typed_here1.borrow_mut() = t;
        let t = Self::read_into::<_, NestedTypes2_SubtypeA_SubtypeCc>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.typed_here2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2_SubtypeA {
}
impl NestedTypes2_SubtypeA {
    pub fn typed_at_root(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeB>> {
        self.typed_at_root.borrow()
    }
}
impl NestedTypes2_SubtypeA {
    pub fn typed_here1(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeA_SubtypeC>> {
        self.typed_here1.borrow()
    }
}
impl NestedTypes2_SubtypeA {
    pub fn typed_here2(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeA_SubtypeCc>> {
        self.typed_here2.borrow()
    }
}
impl NestedTypes2_SubtypeA {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedTypes2_SubtypeA_SubtypeC {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<NestedTypes2_SubtypeA>,
    pub(crate) _self_shared: SharedType<Self>,
    value_c: RefCell<i8>,
    typed_here: RefCell<OptRc<NestedTypes2_SubtypeA_SubtypeC_SubtypeD>>,
    typed_parent: RefCell<OptRc<NestedTypes2_SubtypeA_SubtypeCc>>,
    typed_root: RefCell<OptRc<NestedTypes2_SubtypeB>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2_SubtypeA_SubtypeC {
    type Root = NestedTypes2;
    type Parent = NestedTypes2_SubtypeA;

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
        *self_rc.value_c.borrow_mut() = _io.read_s1()?;
        let t = Self::read_into::<_, NestedTypes2_SubtypeA_SubtypeC_SubtypeD>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.typed_here.borrow_mut() = t;
        let t = Self::read_into::<_, NestedTypes2_SubtypeA_SubtypeCc>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.typed_parent.borrow_mut() = t;
        let t = Self::read_into::<_, NestedTypes2_SubtypeB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.typed_root.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2_SubtypeA_SubtypeC {
}
impl NestedTypes2_SubtypeA_SubtypeC {
    pub fn value_c(&self) -> Ref<'_, i8> {
        self.value_c.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeC {
    pub fn typed_here(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeA_SubtypeC_SubtypeD>> {
        self.typed_here.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeC {
    pub fn typed_parent(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeA_SubtypeCc>> {
        self.typed_parent.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeC {
    pub fn typed_root(&self) -> Ref<'_, OptRc<NestedTypes2_SubtypeB>> {
        self.typed_root.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeC {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedTypes2_SubtypeA_SubtypeC_SubtypeD {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<NestedTypes2_SubtypeA_SubtypeC>,
    pub(crate) _self_shared: SharedType<Self>,
    value_d: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2_SubtypeA_SubtypeC_SubtypeD {
    type Root = NestedTypes2;
    type Parent = NestedTypes2_SubtypeA_SubtypeC;

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
        *self_rc.value_d.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2_SubtypeA_SubtypeC_SubtypeD {
}
impl NestedTypes2_SubtypeA_SubtypeC_SubtypeD {
    pub fn value_d(&self) -> Ref<'_, i8> {
        self.value_d.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeC_SubtypeD {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedTypes2_SubtypeA_SubtypeCc {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    value_cc: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2_SubtypeA_SubtypeCc {
    type Root = NestedTypes2;
    type Parent = KStructUnit;

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
        *self_rc.value_cc.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2_SubtypeA_SubtypeCc {
}
impl NestedTypes2_SubtypeA_SubtypeCc {
    pub fn value_cc(&self) -> Ref<'_, i8> {
        self.value_cc.borrow()
    }
}
impl NestedTypes2_SubtypeA_SubtypeCc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NestedTypes2_SubtypeB {
    pub(crate) _root: SharedType<NestedTypes2>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    value_b: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NestedTypes2_SubtypeB {
    type Root = NestedTypes2;
    type Parent = KStructUnit;

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
        *self_rc.value_b.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes2_SubtypeB {
}
impl NestedTypes2_SubtypeB {
    pub fn value_b(&self) -> Ref<'_, i8> {
        self.value_b.borrow()
    }
}
impl NestedTypes2_SubtypeB {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
