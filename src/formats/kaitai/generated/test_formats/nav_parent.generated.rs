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
pub struct NavParent {
    pub(crate) _root: SharedType<NavParent>,
    pub(crate) _parent: SharedType<NavParent>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<NavParent_HeaderObj>>,
    index: RefCell<OptRc<NavParent_IndexObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParent {
    type Root = NavParent;
    type Parent = NavParent;

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
        let t = Self::read_into::<_, NavParent_HeaderObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        let t = Self::read_into::<_, NavParent_IndexObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.index.borrow_mut() = t;
        Ok(())
    }
}
impl NavParent {
}
impl NavParent {
    pub fn header(&self) -> Ref<'_, OptRc<NavParent_HeaderObj>> {
        self.header.borrow()
    }
}
impl NavParent {
    pub fn index(&self) -> Ref<'_, OptRc<NavParent_IndexObj>> {
        self.index.borrow()
    }
}
impl NavParent {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParent_Entry {
    pub(crate) _root: SharedType<NavParent>,
    pub(crate) _parent: SharedType<NavParent_IndexObj>,
    pub(crate) _self_shared: SharedType<Self>,
    filename: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParent_Entry {
    type Root = NavParent;
    type Parent = NavParent_IndexObj;

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
        *self_rc.filename.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.header().filename_len())?)?, "UTF-8")?;
        Ok(())
    }
}
impl NavParent_Entry {
}
impl NavParent_Entry {
    pub fn filename(&self) -> Ref<'_, String> {
        self.filename.borrow()
    }
}
impl NavParent_Entry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParent_HeaderObj {
    pub(crate) _root: SharedType<NavParent>,
    pub(crate) _parent: SharedType<NavParent>,
    pub(crate) _self_shared: SharedType<Self>,
    qty_entries: RefCell<u32>,
    filename_len: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParent_HeaderObj {
    type Root = NavParent;
    type Parent = NavParent;

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
        *self_rc.qty_entries.borrow_mut() = _io.read_u4le()?;
        *self_rc.filename_len.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl NavParent_HeaderObj {
}
impl NavParent_HeaderObj {
    pub fn qty_entries(&self) -> Ref<'_, u32> {
        self.qty_entries.borrow()
    }
}
impl NavParent_HeaderObj {
    pub fn filename_len(&self) -> Ref<'_, u32> {
        self.filename_len.borrow()
    }
}
impl NavParent_HeaderObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavParent_IndexObj {
    pub(crate) _root: SharedType<NavParent>,
    pub(crate) _parent: SharedType<NavParent>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    entries: RefCell<Vec<OptRc<NavParent_Entry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavParent_IndexObj {
    type Root = NavParent;
    type Parent = NavParent;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.entries.borrow_mut() = Vec::new();
        let l_entries = usize::try_from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.header().qty_entries())?;
        for _i in 0_usize..l_entries {
            let t = Self::read_into::<_, NavParent_Entry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.entries.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl NavParent_IndexObj {
}
impl NavParent_IndexObj {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl NavParent_IndexObj {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<NavParent_Entry>>> {
        self.entries.borrow()
    }
}
impl NavParent_IndexObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
