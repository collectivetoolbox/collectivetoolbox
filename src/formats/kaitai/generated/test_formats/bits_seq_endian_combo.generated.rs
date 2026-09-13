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
pub struct BitsSeqEndianCombo {
    pub(crate) _root: SharedType<BitsSeqEndianCombo>,
    pub(crate) _parent: SharedType<BitsSeqEndianCombo>,
    pub(crate) _self_shared: SharedType<Self>,
    be1: RefCell<u64>,
    be2: RefCell<u64>,
    le3: RefCell<u64>,
    be4: RefCell<u64>,
    le5: RefCell<u64>,
    le6: RefCell<u64>,
    le7: RefCell<u64>,
    be8: RefCell<bool>,
    _io: RefCell<BytesReader>,
}
impl KStruct for BitsSeqEndianCombo {
    type Root = BitsSeqEndianCombo;
    type Parent = BitsSeqEndianCombo;

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
        *self_rc.be1.borrow_mut() = _io.read_bits_int_be(6)?;
        *self_rc.be2.borrow_mut() = _io.read_bits_int_be(10)?;
        *self_rc.le3.borrow_mut() = _io.read_bits_int_le(8)?;
        *self_rc.be4.borrow_mut() = _io.read_bits_int_be(8)?;
        *self_rc.le5.borrow_mut() = _io.read_bits_int_le(5)?;
        *self_rc.le6.borrow_mut() = _io.read_bits_int_le(6)?;
        *self_rc.le7.borrow_mut() = _io.read_bits_int_le(5)?;
        *self_rc.be8.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        Ok(())
    }
}
impl BitsSeqEndianCombo {
}
impl BitsSeqEndianCombo {
    pub fn be1(&self) -> Ref<'_, u64> {
        self.be1.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn be2(&self) -> Ref<'_, u64> {
        self.be2.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn le3(&self) -> Ref<'_, u64> {
        self.le3.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn be4(&self) -> Ref<'_, u64> {
        self.be4.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn le5(&self) -> Ref<'_, u64> {
        self.le5.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn le6(&self) -> Ref<'_, u64> {
        self.le6.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn le7(&self) -> Ref<'_, u64> {
        self.le7.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn be8(&self) -> Ref<'_, bool> {
        self.be8.borrow()
    }
}
impl BitsSeqEndianCombo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
