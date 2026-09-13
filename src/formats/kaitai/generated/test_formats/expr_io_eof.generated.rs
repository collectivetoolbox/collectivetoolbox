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
pub struct ExprIoEof {
    pub(crate) _root: SharedType<ExprIoEof>,
    pub(crate) _parent: SharedType<ExprIoEof>,
    pub(crate) _self_shared: SharedType<Self>,
    substream1: RefCell<OptRc<ExprIoEof_OneOrTwo>>,
    substream2: RefCell<OptRc<ExprIoEof_OneOrTwo>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ExprIoEof {
    type Root = ExprIoEof;
    type Parent = ExprIoEof;

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
        let t = Self::read_into::<_, ExprIoEof_OneOrTwo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.substream1.borrow_mut() = t;
        let t = Self::read_into::<_, ExprIoEof_OneOrTwo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.substream2.borrow_mut() = t;
        Ok(())
    }
}
impl ExprIoEof {
}
impl ExprIoEof {
    pub fn substream1(&self) -> Ref<'_, OptRc<ExprIoEof_OneOrTwo>> {
        self.substream1.borrow()
    }
}
impl ExprIoEof {
    pub fn substream2(&self) -> Ref<'_, OptRc<ExprIoEof_OneOrTwo>> {
        self.substream2.borrow()
    }
}
impl ExprIoEof {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprIoEof_OneOrTwo {
    pub(crate) _root: SharedType<ExprIoEof>,
    pub(crate) _parent: SharedType<ExprIoEof>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u32>,
    two: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_reflect_eof: Cell<bool>,
    reflect_eof: RefCell<bool>,
}
impl KStruct for ExprIoEof_OneOrTwo {
    type Root = ExprIoEof;
    type Parent = ExprIoEof;

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
        *self_rc.one.borrow_mut() = _io.read_u4le()?;
        if !(_io.is_eof()) {
            *self_rc.two.borrow_mut() = _io.read_u4le()?;
        }
        Ok(())
    }
}
impl ExprIoEof_OneOrTwo {
    pub fn reflect_eof(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_reflect_eof.get() {
            return Ok(self.reflect_eof.borrow());
        }
        self.f_reflect_eof.set(true);
        *self.reflect_eof.borrow_mut() = (_io.is_eof()).try_into()?;
        Ok(self.reflect_eof.borrow())
    }
}
impl ExprIoEof_OneOrTwo {
    pub fn one(&self) -> Ref<'_, u32> {
        self.one.borrow()
    }
}
impl ExprIoEof_OneOrTwo {
    pub fn two(&self) -> Ref<'_, u32> {
        self.two.borrow()
    }
}
impl ExprIoEof_OneOrTwo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
