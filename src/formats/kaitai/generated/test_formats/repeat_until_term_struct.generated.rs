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
pub struct RepeatUntilTermStruct {
    pub(crate) _root: SharedType<RepeatUntilTermStruct>,
    pub(crate) _parent: SharedType<RepeatUntilTermStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    records1: RefCell<Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>>,
    records2: RefCell<Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>>,
    records3: RefCell<Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>>,
    _io: RefCell<BytesReader>,
    records1_raw: RefCell<Vec<u8>>,
    records2_raw: RefCell<Vec<u8>>,
    records3_raw: RefCell<Vec<u8>>,
}
impl KStruct for RepeatUntilTermStruct {
    type Root = RepeatUntilTermStruct;
    type Parent = RepeatUntilTermStruct;

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
        *self_rc.records1.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let _raw_records1 = _io.read_bytes_term(170, false, true, true)?;
                let _io_records1 = BytesReader::from(_raw_records1);
                let t = Self::read_into::<BytesReader, RepeatUntilTermStruct_BytesWrapper>(&_io_records1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records1.borrow_mut().push(t);
                let _t_records1 = self_rc.records1.borrow();
                let Some(_tmpa) = _t_records1.last() else { break; };
                _i = _i.saturating_add(1);
                if _tmpa.value().len() == 0 { break; }
            }
        }
        *self_rc.records2.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let _raw_records2 = _io.read_bytes_term(170, true, true, true)?;
                let _io_records2 = BytesReader::from(_raw_records2);
                let t = Self::read_into::<BytesReader, RepeatUntilTermStruct_BytesWrapper>(&_io_records2, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records2.borrow_mut().push(t);
                let _t_records2 = self_rc.records2.borrow();
                let Some(_tmpa) = _t_records2.last() else { break; };
                _i = _i.saturating_add(1);
                if *_tmpa.value() != vec![0xaau8] { break; }
            }
        }
        *self_rc.records3.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let _raw_records3 = _io.read_bytes_term(85, false, false, true)?;
                let _io_records3 = BytesReader::from(_raw_records3);
                let t = Self::read_into::<BytesReader, RepeatUntilTermStruct_BytesWrapper>(&_io_records3, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.records3.borrow_mut().push(t);
                let _t_records3 = self_rc.records3.borrow();
                let Some(_tmpa) = _t_records3.last() else { break; };
                _i = _i.saturating_add(1);
                if *_tmpa.value() == *self_rc.records1().last().ok_or(KError::EmptyIterator)?.value() { break; }
            }
        }
        Ok(())
    }
}
impl RepeatUntilTermStruct {
}
impl RepeatUntilTermStruct {
    pub fn records1(&self) -> Ref<'_, Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>> {
        self.records1.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn records2(&self) -> Ref<'_, Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>> {
        self.records2.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn records3(&self) -> Ref<'_, Vec<OptRc<RepeatUntilTermStruct_BytesWrapper>>> {
        self.records3.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn records1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.records1_raw.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn records2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.records2_raw.borrow()
    }
}
impl RepeatUntilTermStruct {
    pub fn records3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.records3_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RepeatUntilTermStruct_BytesWrapper {
    pub(crate) _root: SharedType<RepeatUntilTermStruct>,
    pub(crate) _parent: SharedType<RepeatUntilTermStruct>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for RepeatUntilTermStruct_BytesWrapper {
    type Root = RepeatUntilTermStruct;
    type Parent = RepeatUntilTermStruct;

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
        Ok(())
    }
}
impl RepeatUntilTermStruct_BytesWrapper {
}
impl RepeatUntilTermStruct_BytesWrapper {
    pub fn value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }
}
impl RepeatUntilTermStruct_BytesWrapper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
