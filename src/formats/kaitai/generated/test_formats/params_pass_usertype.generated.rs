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
pub struct ParamsPassUsertype {
    pub(crate) _root: SharedType<ParamsPassUsertype>,
    pub(crate) _parent: SharedType<ParamsPassUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    first: RefCell<OptRc<ParamsPassUsertype_Block>>,
    one: RefCell<OptRc<ParamsPassUsertype_ParamType>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassUsertype {
    type Root = ParamsPassUsertype;
    type Parent = ParamsPassUsertype;

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
        let t = Self::read_into::<_, ParamsPassUsertype_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.first.borrow_mut() = t;
        let f = |t : &mut ParamsPassUsertype_ParamType| Ok(t.set_params(self_rc.first().clone()));
        let t = Self::read_into_with_init::<_, ParamsPassUsertype_ParamType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.one.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassUsertype {
}
impl ParamsPassUsertype {
    pub fn first(&self) -> Ref<'_, OptRc<ParamsPassUsertype_Block>> {
        self.first.borrow()
    }
}
impl ParamsPassUsertype {
    pub fn one(&self) -> Ref<'_, OptRc<ParamsPassUsertype_ParamType>> {
        self.one.borrow()
    }
}
impl ParamsPassUsertype {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassUsertype_Block {
    pub(crate) _root: SharedType<ParamsPassUsertype>,
    pub(crate) _parent: SharedType<ParamsPassUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsPassUsertype_Block {
    type Root = ParamsPassUsertype;
    type Parent = ParamsPassUsertype;

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
        *self_rc.foo.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassUsertype_Block {
}
impl ParamsPassUsertype_Block {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl ParamsPassUsertype_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassUsertype_ParamType {
    pub(crate) _root: SharedType<ParamsPassUsertype>,
    pub(crate) _parent: SharedType<ParamsPassUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<OptRc<ParamsPassUsertype_Block>>,
    buf: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    buf_raw: RefCell<Vec<u8>>,
}
impl KStruct for ParamsPassUsertype_ParamType {
    type Root = ParamsPassUsertype;
    type Parent = ParamsPassUsertype;

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
        *self_rc.buf.borrow_mut() = _io.read_bytes(usize::from(*self_rc.foo().foo()))?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassUsertype_ParamType {
    pub fn foo(&self) -> Ref<'_, OptRc<ParamsPassUsertype_Block>> {
        self.foo.borrow()
    }
}
impl ParamsPassUsertype_ParamType {
    pub fn set_params(&mut self, foo: OptRc<ParamsPassUsertype_Block>) {
        *self.foo.borrow_mut() = foo;
    }
}
impl ParamsPassUsertype_ParamType {
}
impl ParamsPassUsertype_ParamType {
    pub fn buf(&self) -> Ref<'_, Vec<u8>> {
        self.buf.borrow()
    }
}
impl ParamsPassUsertype_ParamType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ParamsPassUsertype_ParamType {
    pub fn buf_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_raw.borrow()
    }
}
