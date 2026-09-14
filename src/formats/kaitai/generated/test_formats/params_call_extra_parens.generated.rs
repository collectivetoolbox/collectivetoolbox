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
pub struct ParamsCallExtraParens {
    pub(crate) _root: SharedType<ParamsCallExtraParens>,
    pub(crate) _parent: SharedType<ParamsCallExtraParens>,
    pub(crate) _self_shared: SharedType<Self>,
    buf1: RefCell<OptRc<ParamsCallExtraParens_MyStr1>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsCallExtraParens {
    type Root = ParamsCallExtraParens;
    type Parent = ParamsCallExtraParens;

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
        let f = |t : &mut ParamsCallExtraParens_MyStr1| Ok(t.set_params((5).try_into().map_err(|_| KError::CastError)?));
        let t = Self::read_into_with_init::<_, ParamsCallExtraParens_MyStr1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.buf1.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsCallExtraParens {
}
impl ParamsCallExtraParens {
    pub fn buf1(&self) -> Ref<'_, OptRc<ParamsCallExtraParens_MyStr1>> {
        self.buf1.borrow()
    }
}
impl ParamsCallExtraParens {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsCallExtraParens_MyStr1 {
    pub(crate) _root: SharedType<ParamsCallExtraParens>,
    pub(crate) _parent: SharedType<ParamsCallExtraParens>,
    pub(crate) _self_shared: SharedType<Self>,
    len: RefCell<u32>,
    body: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ParamsCallExtraParens_MyStr1 {
    type Root = ParamsCallExtraParens;
    type Parent = ParamsCallExtraParens;

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
        *self_rc.body.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.len())?)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsCallExtraParens_MyStr1 {
    pub fn len(&self) -> Ref<'_, u32> {
        self.len.borrow()
    }
}
impl ParamsCallExtraParens_MyStr1 {
    pub fn set_params(&mut self, len: u32) {
        *self.len.borrow_mut() = len;
    }
}
impl ParamsCallExtraParens_MyStr1 {
}
impl ParamsCallExtraParens_MyStr1 {
    pub fn body(&self) -> Ref<'_, String> {
        self.body.borrow()
    }
}
impl ParamsCallExtraParens_MyStr1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
