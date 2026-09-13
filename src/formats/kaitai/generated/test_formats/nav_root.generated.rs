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
pub struct NavRoot {
    pub(crate) _root: SharedType<NavRoot>,
    pub(crate) _parent: SharedType<NavRoot>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<NavRoot_HeaderObj>>,
    index: RefCell<OptRc<NavRoot_IndexObj>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavRoot {
    type Root = NavRoot;
    type Parent = NavRoot;

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
        let t = Self::read_into::<_, NavRoot_HeaderObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        let t = Self::read_into::<_, NavRoot_IndexObj>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.index.borrow_mut() = t;
        Ok(())
    }
}
impl NavRoot {
}
impl NavRoot {
    pub fn header(&self) -> Ref<'_, OptRc<NavRoot_HeaderObj>> {
        self.header.borrow()
    }
}
impl NavRoot {
    pub fn index(&self) -> Ref<'_, OptRc<NavRoot_IndexObj>> {
        self.index.borrow()
    }
}
impl NavRoot {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavRoot_Entry {
    pub(crate) _root: SharedType<NavRoot>,
    pub(crate) _parent: SharedType<NavRoot_IndexObj>,
    pub(crate) _self_shared: SharedType<Self>,
    filename: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavRoot_Entry {
    type Root = NavRoot;
    type Parent = NavRoot_IndexObj;

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
        *self_rc.filename.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header().filename_len())?)?, "UTF-8")?;
        Ok(())
    }
}
impl NavRoot_Entry {
}
impl NavRoot_Entry {
    pub fn filename(&self) -> Ref<'_, String> {
        self.filename.borrow()
    }
}
impl NavRoot_Entry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavRoot_HeaderObj {
    pub(crate) _root: SharedType<NavRoot>,
    pub(crate) _parent: SharedType<NavRoot>,
    pub(crate) _self_shared: SharedType<Self>,
    qty_entries: RefCell<u32>,
    filename_len: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavRoot_HeaderObj {
    type Root = NavRoot;
    type Parent = NavRoot;

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
impl NavRoot_HeaderObj {
}
impl NavRoot_HeaderObj {
    pub fn qty_entries(&self) -> Ref<'_, u32> {
        self.qty_entries.borrow()
    }
}
impl NavRoot_HeaderObj {
    pub fn filename_len(&self) -> Ref<'_, u32> {
        self.filename_len.borrow()
    }
}
impl NavRoot_HeaderObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NavRoot_IndexObj {
    pub(crate) _root: SharedType<NavRoot>,
    pub(crate) _parent: SharedType<NavRoot>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    entries: RefCell<Vec<OptRc<NavRoot_Entry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for NavRoot_IndexObj {
    type Root = NavRoot;
    type Parent = NavRoot;

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
        let l_entries = usize::try_from(*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header().qty_entries())?;
        for _i in 0_usize..l_entries {
            let t = Self::read_into::<_, NavRoot_Entry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.entries.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl NavRoot_IndexObj {
}
impl NavRoot_IndexObj {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl NavRoot_IndexObj {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<NavRoot_Entry>>> {
        self.entries.borrow()
    }
}
impl NavRoot_IndexObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
