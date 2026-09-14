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
pub struct BitsByteAligned {
    pub(crate) _root: SharedType<BitsByteAligned>,
    pub(crate) _parent: SharedType<BitsByteAligned>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u64>,
    byte_1: RefCell<u8>,
    two: RefCell<u64>,
    three: RefCell<bool>,
    byte_2: RefCell<Vec<u8>>,
    four: RefCell<u64>,
    byte_3: RefCell<OptRc<BitsByteAligned_Foo>>,
    full_byte: RefCell<u64>,
    byte_4: RefCell<u8>,
    five: RefCell<u64>,
    bytes_term: RefCell<Vec<u8>>,
    six: RefCell<u64>,
    _io: RefCell<BytesReader>,
    byte_3_raw: RefCell<Vec<u8>>,
}
impl KStruct for BitsByteAligned {
    type Root = BitsByteAligned;
    type Parent = BitsByteAligned;

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
        *self_rc.one.borrow_mut() = _io.read_bits_int_be(6)?;
        io.align_to_byte()?;
        *self_rc.byte_1.borrow_mut() = _io.read_u1()?;
        *self_rc.two.borrow_mut() = _io.read_bits_int_be(3)?;
        *self_rc.three.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        io.align_to_byte()?;
        *self_rc.byte_2.borrow_mut() = _io.read_bytes(1_usize)?;
        *self_rc.four.borrow_mut() = _io.read_bits_int_be(14)?;
        io.align_to_byte()?;
        let _raw_byte_3 = _io.read_bytes(3_usize)?;
        *self_rc.byte_3_raw.borrow_mut() = _raw_byte_3.clone();
        let _io_byte_3 = BytesReader::from(_raw_byte_3);
        let t = Self::read_into::<BytesReader, BitsByteAligned_Foo>(&_io_byte_3, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.byte_3.borrow_mut() = t;
        *self_rc.full_byte.borrow_mut() = _io.read_bits_int_be(8)?;
        io.align_to_byte()?;
        *self_rc.byte_4.borrow_mut() = _io.read_u1()?;
        *self_rc.five.borrow_mut() = _io.read_bits_int_be(22)?;
        io.align_to_byte()?;
        *self_rc.bytes_term.borrow_mut() = _io.read_bytes_term(69, true, true, true)?;
        *self_rc.six.borrow_mut() = _io.read_bits_int_be(8)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BitsByteAligned {
}
impl BitsByteAligned {
    pub fn one(&self) -> Ref<'_, u64> {
        self.one.borrow()
    }
}
impl BitsByteAligned {
    pub fn byte_1(&self) -> Ref<'_, u8> {
        self.byte_1.borrow()
    }
}
impl BitsByteAligned {
    pub fn two(&self) -> Ref<'_, u64> {
        self.two.borrow()
    }
}
impl BitsByteAligned {
    pub fn three(&self) -> Ref<'_, bool> {
        self.three.borrow()
    }
}
impl BitsByteAligned {
    pub fn byte_2(&self) -> Ref<'_, Vec<u8>> {
        self.byte_2.borrow()
    }
}
impl BitsByteAligned {
    pub fn four(&self) -> Ref<'_, u64> {
        self.four.borrow()
    }
}
impl BitsByteAligned {
    pub fn byte_3(&self) -> Ref<'_, OptRc<BitsByteAligned_Foo>> {
        self.byte_3.borrow()
    }
}
impl BitsByteAligned {
    pub fn full_byte(&self) -> Ref<'_, u64> {
        self.full_byte.borrow()
    }
}
impl BitsByteAligned {
    pub fn byte_4(&self) -> Ref<'_, u8> {
        self.byte_4.borrow()
    }
}
impl BitsByteAligned {
    pub fn five(&self) -> Ref<'_, u64> {
        self.five.borrow()
    }
}
impl BitsByteAligned {
    pub fn bytes_term(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_term.borrow()
    }
}
impl BitsByteAligned {
    pub fn six(&self) -> Ref<'_, u64> {
        self.six.borrow()
    }
}
impl BitsByteAligned {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl BitsByteAligned {
    pub fn byte_3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.byte_3_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BitsByteAligned_Foo {
    pub(crate) _root: SharedType<BitsByteAligned>,
    pub(crate) _parent: SharedType<BitsByteAligned>,
    pub(crate) _self_shared: SharedType<Self>,
    inner: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BitsByteAligned_Foo {
    type Root = BitsByteAligned;
    type Parent = BitsByteAligned;

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
        *self_rc.inner.borrow_mut() = _io.read_bits_int_be(19)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BitsByteAligned_Foo {
}
impl BitsByteAligned_Foo {
    pub fn inner(&self) -> Ref<'_, u64> {
        self.inner.borrow()
    }
}
impl BitsByteAligned_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
