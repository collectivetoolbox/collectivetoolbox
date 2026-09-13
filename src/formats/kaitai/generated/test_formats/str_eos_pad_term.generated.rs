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
pub struct StrEosPadTerm {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    str_pad: RefCell<OptRc<StrEosPadTerm_StrPadType>>,
    str_term: RefCell<OptRc<StrEosPadTerm_StrTermType>>,
    str_term_and_pad: RefCell<OptRc<StrEosPadTerm_StrTermAndPadType>>,
    str_term_include: RefCell<OptRc<StrEosPadTerm_StrTermIncludeType>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTerm {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        let t = Self::read_into::<_, StrEosPadTerm_StrPadType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_pad.borrow_mut() = t;
        let t = Self::read_into::<_, StrEosPadTerm_StrTermType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term.borrow_mut() = t;
        let t = Self::read_into::<_, StrEosPadTerm_StrTermAndPadType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_and_pad.borrow_mut() = t;
        let t = Self::read_into::<_, StrEosPadTerm_StrTermIncludeType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_include.borrow_mut() = t;
        Ok(())
    }
}
impl StrEosPadTerm {
}
impl StrEosPadTerm {
    pub fn str_pad(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrPadType>> {
        self.str_pad.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermType>> {
        self.str_term.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_and_pad(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermAndPadType>> {
        self.str_term_and_pad.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_include(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermIncludeType>> {
        self.str_term_include.borrow()
    }
}
impl StrEosPadTerm {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTerm_StrPadType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTerm_StrPadType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes_full()?, 64), "UTF-8")?;
        Ok(())
    }
}
impl StrEosPadTerm_StrPadType {
}
impl StrEosPadTerm_StrPadType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrPadType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTerm_StrTermAndPadType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTerm_StrTermAndPadType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate(&bytes_strip_right(&_io.read_bytes_full()?, 43), 64, false), "UTF-8")?;
        Ok(())
    }
}
impl StrEosPadTerm_StrTermAndPadType {
}
impl StrEosPadTerm_StrTermAndPadType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermAndPadType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTerm_StrTermIncludeType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTerm_StrTermIncludeType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes_full()?, 64, true), "UTF-8")?;
        Ok(())
    }
}
impl StrEosPadTerm_StrTermIncludeType {
}
impl StrEosPadTerm_StrTermIncludeType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermIncludeType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTerm_StrTermType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTerm_StrTermType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes_full()?, 64, false), "UTF-8")?;
        Ok(())
    }
}
impl StrEosPadTerm_StrTermType {
}
impl StrEosPadTerm_StrTermType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
