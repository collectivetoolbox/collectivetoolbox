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
pub struct TermStruct {
    pub(crate) _root: SharedType<TermStruct>,
    pub(crate) _parent: SharedType<TermStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    s1: RefCell<OptRc<TermStruct_BytesWrapper>>,
    s2: RefCell<OptRc<TermStruct_BytesWrapper>>,
    s3: RefCell<OptRc<TermStruct_BytesWrapper>>,
    _io: RefCell<BytesReader>,
    s1_raw: RefCell<Vec<u8>>,
    s2_raw: RefCell<Vec<u8>>,
    s3_raw: RefCell<Vec<u8>>,
}
impl KStruct for TermStruct {
    type Root = TermStruct;
    type Parent = TermStruct;

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
        let _raw_s1 = _io.read_bytes_term(124, false, true, true)?;
        *self_rc.s1_raw.borrow_mut() = _raw_s1.clone();
        let _io_s1 = BytesReader::from(_raw_s1);
        let t = Self::read_into::<BytesReader, TermStruct_BytesWrapper>(&_io_s1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s1.borrow_mut() = t;
        let _raw_s2 = _io.read_bytes_term(124, false, false, true)?;
        *self_rc.s2_raw.borrow_mut() = _raw_s2.clone();
        let _io_s2 = BytesReader::from(_raw_s2);
        let t = Self::read_into::<BytesReader, TermStruct_BytesWrapper>(&_io_s2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s2.borrow_mut() = t;
        let _raw_s3 = _io.read_bytes_term(64, true, true, true)?;
        *self_rc.s3_raw.borrow_mut() = _raw_s3.clone();
        let _io_s3 = BytesReader::from(_raw_s3);
        let t = Self::read_into::<BytesReader, TermStruct_BytesWrapper>(&_io_s3, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s3.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TermStruct {
}
impl TermStruct {
    pub fn s1(&self) -> Ref<'_, OptRc<TermStruct_BytesWrapper>> {
        self.s1.borrow()
    }
}
impl TermStruct {
    pub fn s2(&self) -> Ref<'_, OptRc<TermStruct_BytesWrapper>> {
        self.s2.borrow()
    }
}
impl TermStruct {
    pub fn s3(&self) -> Ref<'_, OptRc<TermStruct_BytesWrapper>> {
        self.s3.borrow()
    }
}
impl TermStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl TermStruct {
    pub fn s1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s1_raw.borrow()
    }
}
impl TermStruct {
    pub fn s2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s2_raw.borrow()
    }
}
impl TermStruct {
    pub fn s3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s3_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct TermStruct_BytesWrapper {
    pub(crate) _root: SharedType<TermStruct>,
    pub(crate) _parent: SharedType<TermStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for TermStruct_BytesWrapper {
    type Root = TermStruct;
    type Parent = TermStruct;

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
        *self_rc.value.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TermStruct_BytesWrapper {
}
impl TermStruct_BytesWrapper {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl TermStruct_BytesWrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
