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
pub struct OpaqueExternalType02Child {
    pub(crate) _root: SharedType<OpaqueExternalType02Child>,
    pub(crate) _parent: SharedType<OpaqueExternalType02Child>,
    pub(crate) _self_shared: SharedType<Self>,
    s1: RefCell<String>,
    s2: RefCell<String>,
    s3: RefCell<OptRc<OpaqueExternalType02Child_OpaqueExternalType02ChildChild>>,
    _io: RefCell<BytesReader>,
    f_some_method: Cell<bool>,
    some_method: RefCell<bool>,
}
impl KStruct for OpaqueExternalType02Child {
    type Root = OpaqueExternalType02Child;
    type Parent = OpaqueExternalType02Child;

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
        *self_rc.s1.borrow_mut() = bytes_to_str(&_io.read_bytes_term(124, false, true, true)?, "UTF-8")?;
        *self_rc.s2.borrow_mut() = bytes_to_str(&_io.read_bytes_term(124, false, false, true)?, "UTF-8")?;
        let t = Self::read_into::<_, OpaqueExternalType02Child_OpaqueExternalType02ChildChild>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s3.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl OpaqueExternalType02Child {
    pub fn some_method(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_some_method.get() {
            return Ok(self.some_method.borrow());
        }
        self.f_some_method.set(true);
        *self.some_method.borrow_mut() = (true).try_into()?;
        Ok(self.some_method.borrow())
    }
}
impl OpaqueExternalType02Child {
    pub fn s1(&self) -> Ref<'_, String> {
        self.s1.borrow()
    }
}
impl OpaqueExternalType02Child {
    pub fn s2(&self) -> Ref<'_, String> {
        self.s2.borrow()
    }
}
impl OpaqueExternalType02Child {
    pub fn s3(&self) -> Ref<'_, OptRc<OpaqueExternalType02Child_OpaqueExternalType02ChildChild>> {
        self.s3.borrow()
    }
}
impl OpaqueExternalType02Child {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct OpaqueExternalType02Child_OpaqueExternalType02ChildChild {
    pub(crate) _root: SharedType<OpaqueExternalType02Child>,
    pub(crate) _parent: SharedType<OpaqueExternalType02Child>,
    pub(crate) _self_shared: SharedType<Self>,
    s3: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for OpaqueExternalType02Child_OpaqueExternalType02ChildChild {
    type Root = OpaqueExternalType02Child;
    type Parent = OpaqueExternalType02Child;

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
        if *self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.some_method()? {
            *self_rc.s3.borrow_mut() = bytes_to_str(&_io.read_bytes_term(64, true, true, true)?, "UTF-8")?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl OpaqueExternalType02Child_OpaqueExternalType02ChildChild {
}
impl OpaqueExternalType02Child_OpaqueExternalType02ChildChild {
    pub fn s3(&self) -> Ref<'_, String> {
        self.s3.borrow()
    }
}
impl OpaqueExternalType02Child_OpaqueExternalType02ChildChild {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
