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
pub struct ExprIoEofBits {
    pub(crate) _root: SharedType<ExprIoEofBits>,
    pub(crate) _parent: SharedType<ExprIoEofBits>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u64>,
    bar: RefCell<u64>,
    baz: RefCell<u64>,
    align: RefCell<Vec<u8>>,
    qux: RefCell<u64>,
    _io: RefCell<BytesReader>,
    align_raw: RefCell<Vec<u8>>,
}
impl KStruct for ExprIoEofBits {
    type Root = ExprIoEofBits;
    type Parent = ExprIoEofBits;

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
        *self_rc.foo.borrow_mut() = _io.read_bits_int_be(20)?;
        if !(_io.is_eof()) {
            *self_rc.bar.borrow_mut() = _io.read_bits_int_be(4)?;
        }
        if !(_io.is_eof()) {
            *self_rc.baz.borrow_mut() = _io.read_bits_int_be(16)?;
        }
        io.align_to_byte()?;
        *self_rc.align.borrow_mut() = _io.read_bytes(0_usize)?;
        if !(_io.is_eof()) {
            *self_rc.qux.borrow_mut() = _io.read_bits_int_be(16)?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprIoEofBits {
}
impl ExprIoEofBits {
    pub fn foo(&self) -> Ref<'_, u64> {
        self.foo.borrow()
    }
}
impl ExprIoEofBits {
    pub fn bar(&self) -> Ref<'_, u64> {
        self.bar.borrow()
    }
}
impl ExprIoEofBits {
    pub fn baz(&self) -> Ref<'_, u64> {
        self.baz.borrow()
    }
}
impl ExprIoEofBits {
    pub fn align(&self) -> Ref<'_, Vec<u8>> {
        self.align.borrow()
    }
}
impl ExprIoEofBits {
    pub fn qux(&self) -> Ref<'_, u64> {
        self.qux.borrow()
    }
}
impl ExprIoEofBits {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprIoEofBits {
    pub fn align_raw(&self) -> Ref<'_, Vec<u8>> {
        self.align_raw.borrow()
    }
}
