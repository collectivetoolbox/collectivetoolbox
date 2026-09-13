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
pub struct PositionInSeq {
    pub(crate) _root: SharedType<PositionInSeq>,
    pub(crate) _parent: SharedType<PositionInSeq>,
    pub(crate) _self_shared: SharedType<Self>,
    numbers: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_header: Cell<bool>,
    header: RefCell<OptRc<PositionInSeq_HeaderObj>>,
}
impl KStruct for PositionInSeq {
    type Root = PositionInSeq;
    type Parent = PositionInSeq;

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
        *self_rc.numbers.borrow_mut() = Vec::new();
        let l_numbers = usize::try_from(*self_rc.header()?.qty_numbers())?;
        for _i in 0_usize..l_numbers {
            self_rc.numbers.borrow_mut().push(_io.read_u1()?);
        }
        Ok(())
    }
}
impl PositionInSeq {
    pub fn header(
        &self
    ) -> KResult<Ref<'_, OptRc<PositionInSeq_HeaderObj>>> {
        let _io = self._io.borrow();
        if self.f_header.get() {
            return Ok(self.header.borrow());
        }
        let _pos = _io.pos();
        _io.seek(16_usize)?;
        let t = Self::read_into::<_, PositionInSeq_HeaderObj>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.header.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.header.borrow())
    }
}
impl PositionInSeq {
    pub fn numbers(&self) -> Ref<'_, Vec<u8>> {
        self.numbers.borrow()
    }
}
impl PositionInSeq {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct PositionInSeq_HeaderObj {
    pub(crate) _root: SharedType<PositionInSeq>,
    pub(crate) _parent: SharedType<PositionInSeq>,
    pub(crate) _self_shared: SharedType<Self>,
    qty_numbers: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for PositionInSeq_HeaderObj {
    type Root = PositionInSeq;
    type Parent = PositionInSeq;

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
        *self_rc.qty_numbers.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl PositionInSeq_HeaderObj {
}
impl PositionInSeq_HeaderObj {
    pub fn qty_numbers(&self) -> Ref<'_, u32> {
        self.qty_numbers.borrow()
    }
}
impl PositionInSeq_HeaderObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
