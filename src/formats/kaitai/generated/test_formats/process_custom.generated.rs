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
pub struct ProcessCustom {
    pub(crate) _root: SharedType<ProcessCustom>,
    pub(crate) _parent: SharedType<ProcessCustom>,
    pub(crate) _self_shared: SharedType<Self>,
    buf1: RefCell<Vec<u8>>,
    buf2: RefCell<Vec<u8>>,
    key: RefCell<u8>,
    buf3: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    buf1_raw: RefCell<Vec<u8>>,
    buf2_raw: RefCell<Vec<u8>>,
    buf3_raw: RefCell<Vec<u8>>,
}
impl KStruct for ProcessCustom {
    type Root = ProcessCustom;
    type Parent = ProcessCustom;

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
        *self_rc.buf1.borrow_mut() = crate::my_custom_fx::MyCustomFx::new(u8::try_from(i64::try_from(7)? & 0xff)?, true, &[32u8, 48u8, 64u8]).decode(&_io.read_bytes(5_usize)?).map_err(|e| KError::BytesDecodingError { msg: e })?;
        *self_rc.buf2.borrow_mut() = crate::custom_fx::Nested::Deeply::CustomFx::new(u8::try_from(i64::try_from(7)? & 0xff)?).decode(&_io.read_bytes(5_usize)?).map_err(|e| KError::BytesDecodingError { msg: e })?;
        *self_rc.key.borrow_mut() = _io.read_u1()?;
        *self_rc.buf3.borrow_mut() = crate::my_custom_fx::MyCustomFx::new(u8::try_from(i64::try_from(*self_rc.key())? & 0xff)?, false, &[0u8]).decode(&_io.read_bytes(5_usize)?).map_err(|e| KError::BytesDecodingError { msg: e })?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ProcessCustom {
}
impl ProcessCustom {
    pub fn buf1(&self) -> Ref<'_, Vec<u8>> {
        self.buf1.borrow()
    }
}
impl ProcessCustom {
    pub fn buf2(&self) -> Ref<'_, Vec<u8>> {
        self.buf2.borrow()
    }
}
impl ProcessCustom {
    pub fn key(&self) -> Ref<'_, u8> {
        self.key.borrow()
    }
}
impl ProcessCustom {
    pub fn buf3(&self) -> Ref<'_, Vec<u8>> {
        self.buf3.borrow()
    }
}
impl ProcessCustom {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ProcessCustom {
    pub fn buf1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf1_raw.borrow()
    }
}
impl ProcessCustom {
    pub fn buf2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf2_raw.borrow()
    }
}
impl ProcessCustom {
    pub fn buf3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf3_raw.borrow()
    }
}
