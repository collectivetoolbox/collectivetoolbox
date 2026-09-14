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
pub struct NavParentFalse {
    pub(crate) _root: SharedType<NavParentFalse>,
    pub(crate) _parent: SharedType<NavParentFalse>,
    pub(crate) _self_shared: SharedType<Self>,
    child_size: RefCell<u8>,
    element_a: RefCell<OptRc<NavParentFalse_ParentA>>,
    element_b: RefCell<OptRc<NavParentFalse_ParentB>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentFalse {
    type Root = NavParentFalse;
    type Parent = NavParentFalse;

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
        *self_rc.child_size.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, NavParentFalse_ParentA>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.element_a.borrow_mut() = t;
        let t = Self::read_into::<_, NavParentFalse_ParentB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.element_b.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentFalse {
}
impl NavParentFalse {
    pub fn child_size(&self) -> Ref<'_, u8> {
        self.child_size.borrow()
    }
}
impl NavParentFalse {
    pub fn element_a(&self) -> Ref<'_, OptRc<NavParentFalse_ParentA>> {
        self.element_a.borrow()
    }
}
impl NavParentFalse {
    pub fn element_b(&self) -> Ref<'_, OptRc<NavParentFalse_ParentB>> {
        self.element_b.borrow()
    }
}
impl NavParentFalse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentFalse_Child {
    pub(crate) _root: SharedType<NavParentFalse>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    more: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentFalse_Child {
    type Root = NavParentFalse;
    type Parent = KStructUnit;

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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.code())) == (to_i128(73))) {
            *self_rc.more.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.child_size())?)?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentFalse_Child {
}
impl NavParentFalse_Child {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl NavParentFalse_Child {
    pub fn more(&self) -> Ref<'_, Vec<u8>> {
        self.more.borrow()
    }
}
impl NavParentFalse_Child {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentFalse_ParentA {
    pub(crate) _root: SharedType<NavParentFalse>,
    pub(crate) _parent: SharedType<NavParentFalse>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<OptRc<NavParentFalse_Child>>,
    bar: RefCell<OptRc<NavParentFalse_ParentB>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentFalse_ParentA {
    type Root = NavParentFalse;
    type Parent = NavParentFalse;

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
        let t = Self::read_into::<_, NavParentFalse_Child>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.foo.borrow_mut() = t;
        let t = Self::read_into::<_, NavParentFalse_ParentB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.bar.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentFalse_ParentA {
}
impl NavParentFalse_ParentA {
    pub fn foo(&self) -> Ref<'_, OptRc<NavParentFalse_Child>> {
        self.foo.borrow()
    }
}
impl NavParentFalse_ParentA {
    pub fn bar(&self) -> Ref<'_, OptRc<NavParentFalse_ParentB>> {
        self.bar.borrow()
    }
}
impl NavParentFalse_ParentA {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentFalse_ParentB {
    pub(crate) _root: SharedType<NavParentFalse>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<OptRc<NavParentFalse_Child>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentFalse_ParentB {
    type Root = NavParentFalse;
    type Parent = KStructUnit;

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
        let t = Self::read_into::<_, NavParentFalse_Child>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.foo.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentFalse_ParentB {
}
impl NavParentFalse_ParentB {
    pub fn foo(&self) -> Ref<'_, OptRc<NavParentFalse_Child>> {
        self.foo.borrow()
    }
}
impl NavParentFalse_ParentB {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
