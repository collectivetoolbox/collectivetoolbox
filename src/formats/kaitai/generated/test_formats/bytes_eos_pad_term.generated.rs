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
pub struct BytesEosPadTerm {
    pub(crate) _root: SharedType<BytesEosPadTerm>,
    pub(crate) _parent: SharedType<BytesEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    str_pad: RefCell<OptRc<BytesEosPadTerm_StrPadType>>,
    str_term: RefCell<OptRc<BytesEosPadTerm_StrTermType>>,
    str_term_and_pad: RefCell<OptRc<BytesEosPadTerm_StrTermAndPadType>>,
    str_term_include: RefCell<OptRc<BytesEosPadTerm_StrTermIncludeType>>,
    _io: RefCell<BytesReader>,
    str_pad_raw: RefCell<Vec<u8>>,
    str_term_raw: RefCell<Vec<u8>>,
    str_term_and_pad_raw: RefCell<Vec<u8>>,
    str_term_include_raw: RefCell<Vec<u8>>,
}
impl KStruct for BytesEosPadTerm {
    type Root = BytesEosPadTerm;
    type Parent = BytesEosPadTerm;

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
        let _raw_str_pad = _io.read_bytes(20_usize)?;
        *self_rc.str_pad_raw.borrow_mut() = _raw_str_pad.clone();
        let _io_str_pad = BytesReader::from(_raw_str_pad);
        let t = Self::read_into::<BytesReader, BytesEosPadTerm_StrPadType>(&_io_str_pad, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_pad.borrow_mut() = t;
        let _raw_str_term = _io.read_bytes(20_usize)?;
        *self_rc.str_term_raw.borrow_mut() = _raw_str_term.clone();
        let _io_str_term = BytesReader::from(_raw_str_term);
        let t = Self::read_into::<BytesReader, BytesEosPadTerm_StrTermType>(&_io_str_term, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term.borrow_mut() = t;
        let _raw_str_term_and_pad = _io.read_bytes(20_usize)?;
        *self_rc.str_term_and_pad_raw.borrow_mut() = _raw_str_term_and_pad.clone();
        let _io_str_term_and_pad = BytesReader::from(_raw_str_term_and_pad);
        let t = Self::read_into::<BytesReader, BytesEosPadTerm_StrTermAndPadType>(&_io_str_term_and_pad, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_and_pad.borrow_mut() = t;
        let _raw_str_term_include = _io.read_bytes(20_usize)?;
        *self_rc.str_term_include_raw.borrow_mut() = _raw_str_term_include.clone();
        let _io_str_term_include = BytesReader::from(_raw_str_term_include);
        let t = Self::read_into::<BytesReader, BytesEosPadTerm_StrTermIncludeType>(&_io_str_term_include, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_include.borrow_mut() = t;
        Ok(())
    }
}
impl BytesEosPadTerm {
}
impl BytesEosPadTerm {
    pub fn str_pad(&self) -> Ref<'_, OptRc<BytesEosPadTerm_StrPadType>> {
        self.str_pad.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term(&self) -> Ref<'_, OptRc<BytesEosPadTerm_StrTermType>> {
        self.str_term.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term_and_pad(&self) -> Ref<'_, OptRc<BytesEosPadTerm_StrTermAndPadType>> {
        self.str_term_and_pad.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term_include(&self) -> Ref<'_, OptRc<BytesEosPadTerm_StrTermIncludeType>> {
        self.str_term_include.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_pad_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_pad_raw.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_raw.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term_and_pad_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_and_pad_raw.borrow()
    }
}
impl BytesEosPadTerm {
    pub fn str_term_include_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_include_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BytesEosPadTerm_StrPadType {
    pub(crate) _root: SharedType<BytesEosPadTerm>,
    pub(crate) _parent: SharedType<BytesEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BytesEosPadTerm_StrPadType {
    type Root = BytesEosPadTerm;
    type Parent = BytesEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_strip_right(&_io.read_bytes_full()?, 64);
        Ok(())
    }
}
impl BytesEosPadTerm_StrPadType {
}
impl BytesEosPadTerm_StrPadType {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl BytesEosPadTerm_StrPadType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BytesEosPadTerm_StrTermAndPadType {
    pub(crate) _root: SharedType<BytesEosPadTerm>,
    pub(crate) _parent: SharedType<BytesEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BytesEosPadTerm_StrTermAndPadType {
    type Root = BytesEosPadTerm;
    type Parent = BytesEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_terminate(&bytes_strip_right(&_io.read_bytes_full()?, 43), 64, false);
        Ok(())
    }
}
impl BytesEosPadTerm_StrTermAndPadType {
}
impl BytesEosPadTerm_StrTermAndPadType {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl BytesEosPadTerm_StrTermAndPadType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BytesEosPadTerm_StrTermIncludeType {
    pub(crate) _root: SharedType<BytesEosPadTerm>,
    pub(crate) _parent: SharedType<BytesEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BytesEosPadTerm_StrTermIncludeType {
    type Root = BytesEosPadTerm;
    type Parent = BytesEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_terminate(&_io.read_bytes_full()?, 64, true);
        Ok(())
    }
}
impl BytesEosPadTerm_StrTermIncludeType {
}
impl BytesEosPadTerm_StrTermIncludeType {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl BytesEosPadTerm_StrTermIncludeType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BytesEosPadTerm_StrTermType {
    pub(crate) _root: SharedType<BytesEosPadTerm>,
    pub(crate) _parent: SharedType<BytesEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BytesEosPadTerm_StrTermType {
    type Root = BytesEosPadTerm;
    type Parent = BytesEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_terminate(&_io.read_bytes_full()?, 64, false);
        Ok(())
    }
}
impl BytesEosPadTerm_StrTermType {
}
impl BytesEosPadTerm_StrTermType {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl BytesEosPadTerm_StrTermType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
