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
pub struct BitsSimpleLe {
    pub(crate) _root: SharedType<BitsSimpleLe>,
    pub(crate) _parent: SharedType<BitsSimpleLe>,
    pub(crate) _self_shared: SharedType<Self>,
    byte_1: RefCell<u64>,
    byte_2: RefCell<u64>,
    bits_a: RefCell<bool>,
    bits_b: RefCell<u64>,
    bits_c: RefCell<u64>,
    large_bits_1: RefCell<u64>,
    spacer: RefCell<u64>,
    large_bits_2: RefCell<u64>,
    normal_s2: RefCell<i16>,
    byte_8_9_10: RefCell<u64>,
    byte_11_to_14: RefCell<u64>,
    byte_15_to_19: RefCell<u64>,
    byte_20_to_27: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_test_if_b1: Cell<bool>,
    test_if_b1: RefCell<i32>,
}
impl KStruct for BitsSimpleLe {
    type Root = BitsSimpleLe;
    type Parent = BitsSimpleLe;

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
        *self_rc.byte_1.borrow_mut() = _io.read_bits_int_le(8)?;
        *self_rc.byte_2.borrow_mut() = _io.read_bits_int_le(8)?;
        *self_rc.bits_a.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.bits_b.borrow_mut() = _io.read_bits_int_le(3)?;
        *self_rc.bits_c.borrow_mut() = _io.read_bits_int_le(4)?;
        *self_rc.large_bits_1.borrow_mut() = _io.read_bits_int_le(10)?;
        *self_rc.spacer.borrow_mut() = _io.read_bits_int_le(3)?;
        *self_rc.large_bits_2.borrow_mut() = _io.read_bits_int_le(11)?;
        io.align_to_byte()?;
        *self_rc.normal_s2.borrow_mut() = _io.read_s2be()?;
        *self_rc.byte_8_9_10.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.byte_11_to_14.borrow_mut() = _io.read_bits_int_le(32)?;
        *self_rc.byte_15_to_19.borrow_mut() = _io.read_bits_int_le(40)?;
        *self_rc.byte_20_to_27.borrow_mut() = _io.read_bits_int_le(64)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BitsSimpleLe {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn test_if_b1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_test_if_b1.get() {
            return Ok(self.test_if_b1.borrow());
        }
        self.f_test_if_b1.set(true);
        if *self.bits_a() == true {
            *self.test_if_b1.borrow_mut() = (123).try_into()?;
        }
        Ok(self.test_if_b1.borrow())
    }
}
impl BitsSimpleLe {
    pub fn byte_1(&self) -> Ref<'_, u64> {
        self.byte_1.borrow()
    }
}
impl BitsSimpleLe {
    pub fn byte_2(&self) -> Ref<'_, u64> {
        self.byte_2.borrow()
    }
}
impl BitsSimpleLe {
    pub fn bits_a(&self) -> Ref<'_, bool> {
        self.bits_a.borrow()
    }
}
impl BitsSimpleLe {
    pub fn bits_b(&self) -> Ref<'_, u64> {
        self.bits_b.borrow()
    }
}
impl BitsSimpleLe {
    pub fn bits_c(&self) -> Ref<'_, u64> {
        self.bits_c.borrow()
    }
}
impl BitsSimpleLe {
    pub fn large_bits_1(&self) -> Ref<'_, u64> {
        self.large_bits_1.borrow()
    }
}
impl BitsSimpleLe {
    pub fn spacer(&self) -> Ref<'_, u64> {
        self.spacer.borrow()
    }
}
impl BitsSimpleLe {
    pub fn large_bits_2(&self) -> Ref<'_, u64> {
        self.large_bits_2.borrow()
    }
}
impl BitsSimpleLe {
    pub fn normal_s2(&self) -> Ref<'_, i16> {
        self.normal_s2.borrow()
    }
}
impl BitsSimpleLe {
    pub fn byte_8_9_10(&self) -> Ref<'_, u64> {
        self.byte_8_9_10.borrow()
    }
}
impl BitsSimpleLe {
    pub fn byte_11_to_14(&self) -> Ref<'_, u64> {
        self.byte_11_to_14.borrow()
    }
}
impl BitsSimpleLe {
    pub fn byte_15_to_19(&self) -> Ref<'_, u64> {
        self.byte_15_to_19.borrow()
    }
}
impl BitsSimpleLe {
    pub fn byte_20_to_27(&self) -> Ref<'_, u64> {
        self.byte_20_to_27.borrow()
    }
}
impl BitsSimpleLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
