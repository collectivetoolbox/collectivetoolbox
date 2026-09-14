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
pub struct StrEncodingsEscapingEnc {
    pub(crate) _root: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _parent: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _self_shared: SharedType<Self>,
    len_of_1: RefCell<u16>,
    str1: RefCell<OptRc<StrEncodingsEscapingEnc_Str1Wrapper>>,
    len_of_2: RefCell<u16>,
    str2: RefCell<OptRc<StrEncodingsEscapingEnc_Str2Wrapper>>,
    len_of_3: RefCell<u16>,
    str3: RefCell<OptRc<StrEncodingsEscapingEnc_Str3Wrapper>>,
    len_of_4: RefCell<u16>,
    str4: RefCell<OptRc<StrEncodingsEscapingEnc_Str4Wrapper>>,
    _io: RefCell<BytesReader>,
    str1_raw: RefCell<Vec<u8>>,
    str2_raw: RefCell<Vec<u8>>,
    str3_raw: RefCell<Vec<u8>>,
    str4_raw: RefCell<Vec<u8>>,
}
impl KStruct for StrEncodingsEscapingEnc {
    type Root = StrEncodingsEscapingEnc;
    type Parent = StrEncodingsEscapingEnc;

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
        *self_rc.len_of_1.borrow_mut() = _io.read_u2le()?;
        let _raw_str1 = _io.read_bytes(usize::from(*self_rc.len_of_1()))?;
        *self_rc.str1_raw.borrow_mut() = _raw_str1.clone();
        let _io_str1 = BytesReader::from(_raw_str1);
        let t = Self::read_into::<BytesReader, StrEncodingsEscapingEnc_Str1Wrapper>(&_io_str1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str1.borrow_mut() = t;
        *self_rc.len_of_2.borrow_mut() = _io.read_u2le()?;
        let _raw_str2 = _io.read_bytes(usize::from(*self_rc.len_of_2()))?;
        *self_rc.str2_raw.borrow_mut() = _raw_str2.clone();
        let _io_str2 = BytesReader::from(_raw_str2);
        let t = Self::read_into::<BytesReader, StrEncodingsEscapingEnc_Str2Wrapper>(&_io_str2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str2.borrow_mut() = t;
        *self_rc.len_of_3.borrow_mut() = _io.read_u2le()?;
        let _raw_str3 = _io.read_bytes(usize::from(*self_rc.len_of_3()))?;
        *self_rc.str3_raw.borrow_mut() = _raw_str3.clone();
        let _io_str3 = BytesReader::from(_raw_str3);
        let t = Self::read_into::<BytesReader, StrEncodingsEscapingEnc_Str3Wrapper>(&_io_str3, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str3.borrow_mut() = t;
        *self_rc.len_of_4.borrow_mut() = _io.read_u2le()?;
        let _raw_str4 = _io.read_bytes(usize::from(*self_rc.len_of_4()))?;
        *self_rc.str4_raw.borrow_mut() = _raw_str4.clone();
        let _io_str4 = BytesReader::from(_raw_str4);
        let t = Self::read_into::<BytesReader, StrEncodingsEscapingEnc_Str4Wrapper>(&_io_str4, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str4.borrow_mut() = t;
        Ok(())
    }
}
impl StrEncodingsEscapingEnc {
}
impl StrEncodingsEscapingEnc {
    pub fn len_of_1(&self) -> Ref<'_, u16> {
        self.len_of_1.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str1(&self) -> Ref<'_, OptRc<StrEncodingsEscapingEnc_Str1Wrapper>> {
        self.str1.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn len_of_2(&self) -> Ref<'_, u16> {
        self.len_of_2.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str2(&self) -> Ref<'_, OptRc<StrEncodingsEscapingEnc_Str2Wrapper>> {
        self.str2.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn len_of_3(&self) -> Ref<'_, u16> {
        self.len_of_3.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str3(&self) -> Ref<'_, OptRc<StrEncodingsEscapingEnc_Str3Wrapper>> {
        self.str3.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn len_of_4(&self) -> Ref<'_, u16> {
        self.len_of_4.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str4(&self) -> Ref<'_, OptRc<StrEncodingsEscapingEnc_Str4Wrapper>> {
        self.str4.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str1_raw.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str2_raw.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str3_raw.borrow()
    }
}
impl StrEncodingsEscapingEnc {
    pub fn str4_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str4_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsEscapingEnc_Str1Wrapper {
    pub(crate) _root: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _parent: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_v: Cell<bool>,
    v: RefCell<String>,
}
impl KStruct for StrEncodingsEscapingEnc_Str1Wrapper {
    type Root = StrEncodingsEscapingEnc;
    type Parent = StrEncodingsEscapingEnc;

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
        Ok(())
    }
}
impl StrEncodingsEscapingEnc_Str1Wrapper {
    pub fn v(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v.get() {
            return Ok(self.v.borrow());
        }
        self.f_v.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.v.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "ASCII\\x")?;
        _io.seek(_pos)?;
        Ok(self.v.borrow())
    }
}
impl StrEncodingsEscapingEnc_Str1Wrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsEscapingEnc_Str2Wrapper {
    pub(crate) _root: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _parent: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_v: Cell<bool>,
    v: RefCell<String>,
}
impl KStruct for StrEncodingsEscapingEnc_Str2Wrapper {
    type Root = StrEncodingsEscapingEnc;
    type Parent = StrEncodingsEscapingEnc;

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
        Ok(())
    }
}
impl StrEncodingsEscapingEnc_Str2Wrapper {
    pub fn v(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v.get() {
            return Ok(self.v.borrow());
        }
        self.f_v.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.v.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-8\'x")?;
        _io.seek(_pos)?;
        Ok(self.v.borrow())
    }
}
impl StrEncodingsEscapingEnc_Str2Wrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsEscapingEnc_Str3Wrapper {
    pub(crate) _root: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _parent: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_v: Cell<bool>,
    v: RefCell<String>,
}
impl KStruct for StrEncodingsEscapingEnc_Str3Wrapper {
    type Root = StrEncodingsEscapingEnc;
    type Parent = StrEncodingsEscapingEnc;

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
        Ok(())
    }
}
impl StrEncodingsEscapingEnc_Str3Wrapper {
    pub fn v(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v.get() {
            return Ok(self.v.borrow());
        }
        self.f_v.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.v.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "SJIS\"x")?;
        _io.seek(_pos)?;
        Ok(self.v.borrow())
    }
}
impl StrEncodingsEscapingEnc_Str3Wrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsEscapingEnc_Str4Wrapper {
    pub(crate) _root: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _parent: SharedType<StrEncodingsEscapingEnc>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_v: Cell<bool>,
    v: RefCell<String>,
}
impl KStruct for StrEncodingsEscapingEnc_Str4Wrapper {
    type Root = StrEncodingsEscapingEnc;
    type Parent = StrEncodingsEscapingEnc;

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
        Ok(())
    }
}
impl StrEncodingsEscapingEnc_Str4Wrapper {
    pub fn v(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v.get() {
            return Ok(self.v.borrow());
        }
        self.f_v.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.v.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "IBM437\nx")?;
        _io.seek(_pos)?;
        Ok(self.v.borrow())
    }
}
impl StrEncodingsEscapingEnc_Str4Wrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
