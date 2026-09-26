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
pub struct StrEosPadTermEqual {
    pub(crate) _root: SharedType<StrEosPadTermEqual>,
    pub(crate) _parent: SharedType<StrEosPadTermEqual>,
    pub(crate) _self_shared: SharedType<Self>,
    s1: RefCell<OptRc<StrEosPadTermEqual_S1Type>>,
    s2: RefCell<OptRc<StrEosPadTermEqual_S2Type>>,
    s3: RefCell<OptRc<StrEosPadTermEqual_S3Type>>,
    s4: RefCell<OptRc<StrEosPadTermEqual_S4Type>>,
    _io: RefCell<BytesReader>,
    s1_raw: RefCell<Vec<u8>>,
    s2_raw: RefCell<Vec<u8>>,
    s3_raw: RefCell<Vec<u8>>,
    s4_raw: RefCell<Vec<u8>>,
}
impl KStruct for StrEosPadTermEqual {
    type Root = StrEosPadTermEqual;
    type Parent = StrEosPadTermEqual;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
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
        let _raw_s1 = _io.read_bytes(20_usize)?;
        *self_rc.s1_raw.borrow_mut() = _raw_s1.clone();
        let _io_s1 = BytesReader::from(_raw_s1);
        let t = Self::read_into::<BytesReader, StrEosPadTermEqual_S1Type>(&_io_s1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s1.borrow_mut() = t;
        let _raw_s2 = _io.read_bytes(20_usize)?;
        *self_rc.s2_raw.borrow_mut() = _raw_s2.clone();
        let _io_s2 = BytesReader::from(_raw_s2);
        let t = Self::read_into::<BytesReader, StrEosPadTermEqual_S2Type>(&_io_s2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s2.borrow_mut() = t;
        let _raw_s3 = _io.read_bytes(20_usize)?;
        *self_rc.s3_raw.borrow_mut() = _raw_s3.clone();
        let _io_s3 = BytesReader::from(_raw_s3);
        let t = Self::read_into::<BytesReader, StrEosPadTermEqual_S3Type>(&_io_s3, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s3.borrow_mut() = t;
        let _raw_s4 = _io.read_bytes(20_usize)?;
        *self_rc.s4_raw.borrow_mut() = _raw_s4.clone();
        let _io_s4 = BytesReader::from(_raw_s4);
        let t = Self::read_into::<BytesReader, StrEosPadTermEqual_S4Type>(&_io_s4, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.s4.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTermEqual {
}
impl StrEosPadTermEqual {
    pub fn s1(&self) -> Ref<'_, OptRc<StrEosPadTermEqual_S1Type>> {
        self.s1.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s2(&self) -> Ref<'_, OptRc<StrEosPadTermEqual_S2Type>> {
        self.s2.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s3(&self) -> Ref<'_, OptRc<StrEosPadTermEqual_S3Type>> {
        self.s3.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s4(&self) -> Ref<'_, OptRc<StrEosPadTermEqual_S4Type>> {
        self.s4.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s1_raw.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s2_raw.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s3_raw.borrow()
    }
}
impl StrEosPadTermEqual {
    pub fn s4_raw(&self) -> Ref<'_, Vec<u8>> {
        self.s4_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTermEqual_S1Type {
    pub(crate) _root: SharedType<StrEosPadTermEqual>,
    pub(crate) _parent: SharedType<StrEosPadTermEqual>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTermEqual_S1Type {
    type Root = StrEosPadTermEqual;
    type Parent = StrEosPadTermEqual;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(64), false, Some(64)), "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTermEqual_S1Type {
}
impl StrEosPadTermEqual_S1Type {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTermEqual_S1Type {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTermEqual_S2Type {
    pub(crate) _root: SharedType<StrEosPadTermEqual>,
    pub(crate) _parent: SharedType<StrEosPadTermEqual>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTermEqual_S2Type {
    type Root = StrEosPadTermEqual;
    type Parent = StrEosPadTermEqual;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(64), true, Some(43)), "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTermEqual_S2Type {
}
impl StrEosPadTermEqual_S2Type {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTermEqual_S2Type {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTermEqual_S3Type {
    pub(crate) _root: SharedType<StrEosPadTermEqual>,
    pub(crate) _parent: SharedType<StrEosPadTermEqual>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTermEqual_S3Type {
    type Root = StrEosPadTermEqual;
    type Parent = StrEosPadTermEqual;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(43), false, Some(43)), "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTermEqual_S3Type {
}
impl StrEosPadTermEqual_S3Type {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTermEqual_S3Type {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTermEqual_S4Type {
    pub(crate) _root: SharedType<StrEosPadTermEqual>,
    pub(crate) _parent: SharedType<StrEosPadTermEqual>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for StrEosPadTermEqual_S4Type {
    type Root = StrEosPadTermEqual;
    type Parent = StrEosPadTermEqual;

    #[allow(clippy::unnecessary_fallible_conversions, clippy::absurd_extreme_comparisons, reason = "Generic validation value conversion")]
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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(46), true, Some(46)), "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTermEqual_S4Type {
}
impl StrEosPadTermEqual_S4Type {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTermEqual_S4Type {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
