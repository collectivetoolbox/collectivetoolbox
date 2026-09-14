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
pub struct NavParentOverride {
    pub(crate) _root: SharedType<NavParentOverride>,
    pub(crate) _parent: SharedType<NavParentOverride>,
    pub(crate) _self_shared: SharedType<Self>,
    child_size: RefCell<u8>,
    child_1: RefCell<OptRc<NavParentOverride_Child>>,
    mediator_2: RefCell<OptRc<NavParentOverride_Mediator>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentOverride {
    type Root = NavParentOverride;
    type Parent = NavParentOverride;

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
        let t = Self::read_into::<_, NavParentOverride_Child>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.child_1.borrow_mut() = t;
        let t = Self::read_into::<_, NavParentOverride_Mediator>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.mediator_2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentOverride {
}
impl NavParentOverride {
    pub fn child_size(&self) -> Ref<'_, u8> {
        self.child_size.borrow()
    }
}
impl NavParentOverride {
    pub fn child_1(&self) -> Ref<'_, OptRc<NavParentOverride_Child>> {
        self.child_1.borrow()
    }
}
impl NavParentOverride {
    pub fn mediator_2(&self) -> Ref<'_, OptRc<NavParentOverride_Mediator>> {
        self.mediator_2.borrow()
    }
}
impl NavParentOverride {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentOverride_Child {
    pub(crate) _root: SharedType<NavParentOverride>,
    pub(crate) _parent: SharedType<NavParentOverride>,
    pub(crate) _self_shared: SharedType<Self>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentOverride_Child {
    type Root = NavParentOverride;
    type Parent = NavParentOverride;

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
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.child_size()))?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentOverride_Child {
}
impl NavParentOverride_Child {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl NavParentOverride_Child {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParentOverride_Mediator {
    pub(crate) _root: SharedType<NavParentOverride>,
    pub(crate) _parent: SharedType<NavParentOverride>,
    pub(crate) _self_shared: SharedType<Self>,
    child_2: RefCell<OptRc<NavParentOverride_Child>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParentOverride_Mediator {
    type Root = NavParentOverride;
    type Parent = NavParentOverride;

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
        let t = Self::read_into::<_, NavParentOverride_Child>(&*_io, Some(self_rc._root.clone()), Some(SharedType::new(self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.clone())))?.into();
        *self_rc.child_2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NavParentOverride_Mediator {
}
impl NavParentOverride_Mediator {
    pub fn child_2(&self) -> Ref<'_, OptRc<NavParentOverride_Child>> {
        self.child_2.borrow()
    }
}
impl NavParentOverride_Mediator {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
