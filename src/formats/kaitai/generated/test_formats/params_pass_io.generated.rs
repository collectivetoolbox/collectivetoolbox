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
pub struct ParamsPassIo {
    pub(crate) _root: SharedType<ParamsPassIo>,
    pub(crate) _parent: SharedType<ParamsPassIo>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<OptRc<ParamsPassIo_Block>>,
    one: RefCell<OptRc<ParamsPassIo_ParamType>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassIo {
    type Root = ParamsPassIo;
    type Parent = ParamsPassIo;

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
        let t = Self::read_into::<_, ParamsPassIo_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.first.borrow_mut() = t;
        let f = |t : &mut ParamsPassIo_ParamType| Ok(t.set_params(if ((to_i128(*self_rc.first().foo())) == (to_i128(255))) { KStream::clone(&*self_rc.first()._io()) } else { KStream::clone(&*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io()) }));
        let t = Self::read_into_with_init::<_, ParamsPassIo_ParamType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.one.borrow_mut() = t;
        Ok(())
    }
}
impl ParamsPassIo {
}
impl ParamsPassIo {
    pub fn first(&self) -> Ref<'_, OptRc<ParamsPassIo_Block>> {
        self.first.borrow()
    }
}
impl ParamsPassIo {
    pub fn one(&self) -> Ref<'_, OptRc<ParamsPassIo_ParamType>> {
        self.one.borrow()
    }
}
impl ParamsPassIo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassIo_Block {
    pub(crate) _root: SharedType<ParamsPassIo>,
    pub(crate) _parent: SharedType<ParamsPassIo>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassIo_Block {
    type Root = ParamsPassIo;
    type Parent = ParamsPassIo;

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
        *self_rc.foo.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl ParamsPassIo_Block {
}
impl ParamsPassIo_Block {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl ParamsPassIo_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassIo_ParamType {
    pub(crate) _root: SharedType<ParamsPassIo>,
    pub(crate) _parent: SharedType<ParamsPassIo>,
    pub(crate) _self_shared: SharedType<Self>,
    arg_stream: RefCell<BytesReader>,
    buf: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassIo_ParamType {
    type Root = ParamsPassIo;
    type Parent = ParamsPassIo;

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
        *self_rc.buf.borrow_mut() = _io.read_bytes(usize::try_from((i64::try_from(self_rc.arg_stream().size())?))?)?;
        Ok(())
    }
}
impl ParamsPassIo_ParamType {
    pub fn arg_stream(&self) -> Ref<'_, BytesReader> {
        self.arg_stream.borrow()
    }
}
impl ParamsPassIo_ParamType {
    pub fn set_params(&mut self, arg_stream: BytesReader) {
        *self.arg_stream.borrow_mut() = arg_stream;
    }
}
impl ParamsPassIo_ParamType {
}
impl ParamsPassIo_ParamType {
    pub fn buf(&self) -> Ref<'_, Vec<u8>> {
        self.buf.borrow()
    }
}
impl ParamsPassIo_ParamType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
