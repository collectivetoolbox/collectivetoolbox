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
pub struct ProcessCoerceUsertype2 {
    pub(crate) _root: SharedType<ProcessCoerceUsertype2>,
    pub(crate) _parent: SharedType<ProcessCoerceUsertype2>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<ProcessCoerceUsertype2_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ProcessCoerceUsertype2 {
    type Root = ProcessCoerceUsertype2;
    type Parent = ProcessCoerceUsertype2;

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
        let l_records = 2_usize;
        for _i in 0_usize..l_records {
            let t = Self::read_into::<_, ProcessCoerceUsertype2_Record>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.records.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl ProcessCoerceUsertype2 {
}
impl ProcessCoerceUsertype2 {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<ProcessCoerceUsertype2_Record>>> {
        self.records.borrow()
    }
}
impl ProcessCoerceUsertype2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessCoerceUsertype2_Foo {
    pub(crate) _root: SharedType<ProcessCoerceUsertype2>,
    pub(crate) _parent: SharedType<ProcessCoerceUsertype2_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ProcessCoerceUsertype2_Foo {
    type Root = ProcessCoerceUsertype2;
    type Parent = ProcessCoerceUsertype2_Record;

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
        *self_rc.value.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl ProcessCoerceUsertype2_Foo {
}
impl ProcessCoerceUsertype2_Foo {
    pub fn value(&self) -> Ref<'_, u32> {
        self.value.borrow()
    }
}
impl ProcessCoerceUsertype2_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProcessCoerceUsertype2_Record {
    pub(crate) _root: SharedType<ProcessCoerceUsertype2>,
    pub(crate) _parent: SharedType<ProcessCoerceUsertype2>,
    pub(crate) _self_shared: SharedType<Self>,
    flag: RefCell<u8>,
    buf_unproc: RefCell<OptRc<ProcessCoerceUsertype2_Foo>>,
    buf_proc: RefCell<OptRc<ProcessCoerceUsertype2_Foo>>,
    _io: RefCell<BytesReader>,
    buf_proc_raw: RefCell<Vec<u8>>,
    f_buf: Cell<bool>,
    buf: RefCell<OptRc<ProcessCoerceUsertype2_Foo>>,
}
impl KStruct for ProcessCoerceUsertype2_Record {
    type Root = ProcessCoerceUsertype2;
    type Parent = ProcessCoerceUsertype2;

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
        *self_rc.flag.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.flag())) == (to_i128(0))) {
            let t = Self::read_into::<_, ProcessCoerceUsertype2_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.buf_unproc.borrow_mut() = t;
        }
        if ((to_i128(*self_rc.flag())) != (to_i128(0))) {
            let _raw_buf_proc = _io.read_bytes(4_usize)?;
            *self_rc.buf_proc_raw.borrow_mut() = _raw_buf_proc.clone();
            let _processed_buf_proc = process_xor_one(&_raw_buf_proc, 170_u8);
            let _io_buf_proc = BytesReader::from(_processed_buf_proc);
            let t = Self::read_into::<BytesReader, ProcessCoerceUsertype2_Foo>(&_io_buf_proc, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.buf_proc.borrow_mut() = t;
        }
        Ok(())
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn buf(
        &self
    ) -> KResult<Ref<'_, OptRc<ProcessCoerceUsertype2_Foo>>> {
        let _io = self._io.borrow();
        if self.f_buf.get() {
            return Ok(self.buf.borrow());
        }
        *self.buf.borrow_mut() = if ((to_i128(*self.flag())) == (to_i128(0))) { self.buf_unproc().clone() } else { self.buf_proc().clone() }.clone();
        Ok(self.buf.borrow())
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn flag(&self) -> Ref<'_, u8> {
        self.flag.borrow()
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn buf_unproc(&self) -> Ref<'_, OptRc<ProcessCoerceUsertype2_Foo>> {
        self.buf_unproc.borrow()
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn buf_proc(&self) -> Ref<'_, OptRc<ProcessCoerceUsertype2_Foo>> {
        self.buf_proc.borrow()
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessCoerceUsertype2_Record {
    pub fn buf_proc_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_proc_raw.borrow()
    }
}
