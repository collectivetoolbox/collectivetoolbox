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
pub struct EosExceptionU4 {
    pub(crate) _root: SharedType<EosExceptionU4>,
    pub(crate) _parent: SharedType<EosExceptionU4>,
    pub(crate) _self_shared: SharedType<Self>,
    envelope: RefCell<OptRc<EosExceptionU4_Data>>,
    _io: RefCell<BytesReader>,
    envelope_raw: RefCell<Vec<u8>>,
}
impl KStruct for EosExceptionU4 {
    type Root = EosExceptionU4;
    type Parent = EosExceptionU4;

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
        let _raw_envelope = _io.read_bytes(6_usize)?;
        *self_rc.envelope_raw.borrow_mut() = _raw_envelope.clone();
        let _io_envelope = BytesReader::from(_raw_envelope);
        let t = Self::read_into::<BytesReader, EosExceptionU4_Data>(&_io_envelope, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.envelope.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EosExceptionU4 {
}
impl EosExceptionU4 {
    pub fn envelope(&self) -> Ref<'_, OptRc<EosExceptionU4_Data>> {
        self.envelope.borrow()
    }
}
impl EosExceptionU4 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl EosExceptionU4 {
    pub fn envelope_raw(&self) -> Ref<'_, Vec<u8>> {
        self.envelope_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EosExceptionU4_Data {
    pub(crate) _root: SharedType<EosExceptionU4>,
    pub(crate) _parent: SharedType<EosExceptionU4>,
    pub(crate) _self_shared: SharedType<Self>,
    prebuf: RefCell<Vec<u8>>,
    fail_int: RefCell<u32>,
    _io: RefCell<BytesReader>,
    prebuf_raw: RefCell<Vec<u8>>,
}
impl KStruct for EosExceptionU4_Data {
    type Root = EosExceptionU4;
    type Parent = EosExceptionU4;

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
        *self_rc.prebuf.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc.fail_int.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EosExceptionU4_Data {
}
impl EosExceptionU4_Data {
    pub fn prebuf(&self) -> Ref<'_, Vec<u8>> {
        self.prebuf.borrow()
    }
}
impl EosExceptionU4_Data {
    pub fn fail_int(&self) -> Ref<'_, u32> {
        self.fail_int.borrow()
    }
}
impl EosExceptionU4_Data {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl EosExceptionU4_Data {
    pub fn prebuf_raw(&self) -> Ref<'_, Vec<u8>> {
        self.prebuf_raw.borrow()
    }
}
