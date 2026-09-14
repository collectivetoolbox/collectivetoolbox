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
pub struct StrEncodingsUtf16 {
    pub(crate) _root: SharedType<StrEncodingsUtf16>,
    pub(crate) _parent: SharedType<StrEncodingsUtf16>,
    pub(crate) _self_shared: SharedType<Self>,
    len_be: RefCell<u32>,
    be_bom_removed: RefCell<OptRc<StrEncodingsUtf16_StrBeBomRemoved>>,
    len_le: RefCell<u32>,
    le_bom_removed: RefCell<OptRc<StrEncodingsUtf16_StrLeBomRemoved>>,
    _io: RefCell<BytesReader>,
    be_bom_removed_raw: RefCell<Vec<u8>>,
    le_bom_removed_raw: RefCell<Vec<u8>>,
}
impl KStruct for StrEncodingsUtf16 {
    type Root = StrEncodingsUtf16;
    type Parent = StrEncodingsUtf16;

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
        *self_rc.len_be.borrow_mut() = _io.read_u4le()?;
        let _raw_be_bom_removed = _io.read_bytes(usize::try_from(*self_rc.len_be())?)?;
        *self_rc.be_bom_removed_raw.borrow_mut() = _raw_be_bom_removed.clone();
        let _io_be_bom_removed = BytesReader::from(_raw_be_bom_removed);
        let t = Self::read_into::<BytesReader, StrEncodingsUtf16_StrBeBomRemoved>(&_io_be_bom_removed, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.be_bom_removed.borrow_mut() = t;
        *self_rc.len_le.borrow_mut() = _io.read_u4le()?;
        let _raw_le_bom_removed = _io.read_bytes(usize::try_from(*self_rc.len_le())?)?;
        *self_rc.le_bom_removed_raw.borrow_mut() = _raw_le_bom_removed.clone();
        let _io_le_bom_removed = BytesReader::from(_raw_le_bom_removed);
        let t = Self::read_into::<BytesReader, StrEncodingsUtf16_StrLeBomRemoved>(&_io_le_bom_removed, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.le_bom_removed.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEncodingsUtf16 {
}
impl StrEncodingsUtf16 {
    pub fn len_be(&self) -> Ref<'_, u32> {
        self.len_be.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn be_bom_removed(&self) -> Ref<'_, OptRc<StrEncodingsUtf16_StrBeBomRemoved>> {
        self.be_bom_removed.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn len_le(&self) -> Ref<'_, u32> {
        self.len_le.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn le_bom_removed(&self) -> Ref<'_, OptRc<StrEncodingsUtf16_StrLeBomRemoved>> {
        self.le_bom_removed.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn be_bom_removed_raw(&self) -> Ref<'_, Vec<u8>> {
        self.be_bom_removed_raw.borrow()
    }
}
impl StrEncodingsUtf16 {
    pub fn le_bom_removed_raw(&self) -> Ref<'_, Vec<u8>> {
        self.le_bom_removed_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsUtf16_StrBeBomRemoved {
    pub(crate) _root: SharedType<StrEncodingsUtf16>,
    pub(crate) _parent: SharedType<StrEncodingsUtf16>,
    pub(crate) _self_shared: SharedType<Self>,
    bom: RefCell<u16>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEncodingsUtf16_StrBeBomRemoved {
    type Root = StrEncodingsUtf16;
    type Parent = StrEncodingsUtf16;

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
        *self_rc.bom.borrow_mut() = _io.read_u2be()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-16BE")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEncodingsUtf16_StrBeBomRemoved {
}
impl StrEncodingsUtf16_StrBeBomRemoved {
    pub fn bom(&self) -> Ref<'_, u16> {
        self.bom.borrow()
    }
}
impl StrEncodingsUtf16_StrBeBomRemoved {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl StrEncodingsUtf16_StrBeBomRemoved {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEncodingsUtf16_StrLeBomRemoved {
    pub(crate) _root: SharedType<StrEncodingsUtf16>,
    pub(crate) _parent: SharedType<StrEncodingsUtf16>,
    pub(crate) _self_shared: SharedType<Self>,
    bom: RefCell<u16>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEncodingsUtf16_StrLeBomRemoved {
    type Root = StrEncodingsUtf16;
    type Parent = StrEncodingsUtf16;

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
        *self_rc.bom.borrow_mut() = _io.read_u2le()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-16LE")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEncodingsUtf16_StrLeBomRemoved {
}
impl StrEncodingsUtf16_StrLeBomRemoved {
    pub fn bom(&self) -> Ref<'_, u16> {
        self.bom.borrow()
    }
}
impl StrEncodingsUtf16_StrLeBomRemoved {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl StrEncodingsUtf16_StrLeBomRemoved {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
