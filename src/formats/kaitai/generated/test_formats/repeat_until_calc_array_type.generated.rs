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
pub struct RepeatUntilCalcArrayType {
    pub(crate) _root: SharedType<RepeatUntilCalcArrayType>,
    pub(crate) _parent: SharedType<RepeatUntilCalcArrayType>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<RepeatUntilCalcArrayType_Record>>>,
    _io: RefCell<BytesReader>,
    records_raw: RefCell<Vec<u8>>,
    f_first_rec: Cell<bool>,
    first_rec: RefCell<OptRc<RepeatUntilCalcArrayType_Record>>,
    f_recs_accessor: Cell<bool>,
    recs_accessor: RefCell<Vec<OptRc<RepeatUntilCalcArrayType_Record>>>,
}
impl KStruct for RepeatUntilCalcArrayType {
    type Root = RepeatUntilCalcArrayType;
    type Parent = RepeatUntilCalcArrayType;

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
        *self_rc.records.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let _raw_records = _io.read_bytes(5_usize)?;
                let _io_records = BytesReader::from(_raw_records);
                let t = Self::read_into::<BytesReader, RepeatUntilCalcArrayType_Record>(&_io_records, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records.borrow_mut().push(t);
                let _t_records = self_rc.records.borrow();
                let Some(_tmpa) = _t_records.last() else { break; };
                _i = _i.saturating_add(1);
                if ((to_i128(*_tmpa.marker())) == (to_i128(170))) { break; }
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RepeatUntilCalcArrayType {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn first_rec(
        &self
    ) -> KResult<Ref<'_, OptRc<RepeatUntilCalcArrayType_Record>>> {
        let _io = self._io.borrow();
        if self.f_first_rec.get() {
            return Ok(self.first_rec.borrow());
        }
        *self.first_rec.borrow_mut() = self.recs_accessor()?.first().ok_or(KError::EmptyIterator)?.clone();
        Ok(self.first_rec.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn recs_accessor(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<RepeatUntilCalcArrayType_Record>>>> {
        let _io = self._io.borrow();
        if self.f_recs_accessor.get() {
            return Ok(self.recs_accessor.borrow());
        }
        self.f_recs_accessor.set(true);
        *self.recs_accessor.borrow_mut() = self.records().to_vec();
        Ok(self.recs_accessor.borrow())
    }
}
impl RepeatUntilCalcArrayType {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<RepeatUntilCalcArrayType_Record>>> {
        self.records.borrow()
    }
}
impl RepeatUntilCalcArrayType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RepeatUntilCalcArrayType {
    pub fn records_raw(&self) -> Ref<'_, Vec<u8>> {
        self.records_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RepeatUntilCalcArrayType_Record {
    pub(crate) _root: SharedType<RepeatUntilCalcArrayType>,
    pub(crate) _parent: SharedType<RepeatUntilCalcArrayType>,
    pub(crate) _self_shared: SharedType<Self>,
    marker: RefCell<u8>,
    body: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RepeatUntilCalcArrayType_Record {
    type Root = RepeatUntilCalcArrayType;
    type Parent = RepeatUntilCalcArrayType;

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
        *self_rc.marker.borrow_mut() = _io.read_u1()?;
        *self_rc.body.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl RepeatUntilCalcArrayType_Record {
}
impl RepeatUntilCalcArrayType_Record {
    pub fn marker(&self) -> Ref<'_, u8> {
        self.marker.borrow()
    }
}
impl RepeatUntilCalcArrayType_Record {
    pub fn body(&self) -> Ref<'_, u32> {
        self.body.borrow()
    }
}
impl RepeatUntilCalcArrayType_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
