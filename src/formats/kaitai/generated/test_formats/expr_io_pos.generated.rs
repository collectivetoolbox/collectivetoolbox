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
pub struct ExprIoPos {
    pub(crate) _root: SharedType<ExprIoPos>,
    pub(crate) _parent: SharedType<ExprIoPos>,
    pub(crate) _self_shared: SharedType<Self>,
    substream1: RefCell<OptRc<ExprIoPos_AllPlusNumber>>,
    substream2: RefCell<OptRc<ExprIoPos_AllPlusNumber>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ExprIoPos {
    type Root = ExprIoPos;
    type Parent = ExprIoPos;

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
        let t = Self::read_into::<_, ExprIoPos_AllPlusNumber>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.substream1.borrow_mut() = t;
        let t = Self::read_into::<_, ExprIoPos_AllPlusNumber>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.substream2.borrow_mut() = t;
        Ok(())
    }
}
impl ExprIoPos {
}
impl ExprIoPos {
    pub fn substream1(&self) -> Ref<'_, OptRc<ExprIoPos_AllPlusNumber>> {
        self.substream1.borrow()
    }
}
impl ExprIoPos {
    pub fn substream2(&self) -> Ref<'_, OptRc<ExprIoPos_AllPlusNumber>> {
        self.substream2.borrow()
    }
}
impl ExprIoPos {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprIoPos_AllPlusNumber {
    pub(crate) _root: SharedType<ExprIoPos>,
    pub(crate) _parent: SharedType<ExprIoPos>,
    pub(crate) _self_shared: SharedType<Self>,
    my_str: RefCell<String>,
    body: RefCell<Vec<u8>>,
    number: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ExprIoPos_AllPlusNumber {
    type Root = ExprIoPos;
    type Parent = ExprIoPos;

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
        *self_rc.my_str.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(((usize::try_from((i64::try_from(_io.size())?))?).saturating_sub(_io.pos())).saturating_sub(2_i32))?)?;
        *self_rc.number.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl ExprIoPos_AllPlusNumber {
}
impl ExprIoPos_AllPlusNumber {
    pub fn my_str(&self) -> Ref<'_, String> {
        self.my_str.borrow()
    }
}
impl ExprIoPos_AllPlusNumber {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl ExprIoPos_AllPlusNumber {
    pub fn number(&self) -> Ref<'_, u16> {
        self.number.borrow()
    }
}
impl ExprIoPos_AllPlusNumber {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
