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
pub struct ExprIfIntOps {
    pub(crate) _root: SharedType<ExprIfIntOps>,
    pub(crate) _parent: SharedType<ExprIfIntOps>,
    pub(crate) _self_shared: SharedType<Self>,
    key: RefCell<u64>,
    skip: RefCell<Vec<u8>>,
    bytes: RefCell<Vec<u8>>,
    items: RefCell<Vec<i8>>,
    _io: RefCell<BytesReader>,
    f_bytes_sub_key: Cell<bool>,
    bytes_sub_key: RefCell<i32>,
    f_items_sub_key: Cell<bool>,
    items_sub_key: RefCell<i8>,
}
impl KStruct for ExprIfIntOps {
    type Root = ExprIfIntOps;
    type Parent = ExprIfIntOps;

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
        if true {
            *self_rc.key.borrow_mut() = _io.read_u8le()?;
        }
        *self_rc.skip.borrow_mut() = _io.read_bytes(8_usize)?;
        *self_rc.bytes.borrow_mut() = _io.read_bytes(8_usize)?;
        *self_rc.items.borrow_mut() = Vec::new();
        let l_items = 4_usize;
        for _i in 0_usize..l_items {
            self_rc.items.borrow_mut().push(_io.read_s1()?);
        }
        Ok(())
    }
}
impl ExprIfIntOps {
    pub fn bytes_sub_key(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_bytes_sub_key.get() {
            return Ok(self.bytes_sub_key.borrow());
        }
        self.f_bytes_sub_key.set(true);
        *self.bytes_sub_key.borrow_mut() = (*(self.bytes().get(usize::try_from(*self.key())?).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.bytes_sub_key.borrow())
    }
    pub fn items_sub_key(
        &self
    ) -> KResult<Ref<'_, i8>> {
        let _io = self._io.borrow();
        if self.f_items_sub_key.get() {
            return Ok(self.items_sub_key.borrow());
        }
        self.f_items_sub_key.set(true);
        *self.items_sub_key.borrow_mut() = (*(self.items().get(usize::try_from(*self.key())?).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.items_sub_key.borrow())
    }
}
impl ExprIfIntOps {
    pub fn key(&self) -> Ref<'_, u64> {
        self.key.borrow()
    }
}
impl ExprIfIntOps {
    pub fn skip(&self) -> Ref<'_, Vec<u8>> {
        self.skip.borrow()
    }
}
impl ExprIfIntOps {
    pub fn bytes(&self) -> Ref<'_, Vec<u8>> {
        self.bytes.borrow()
    }
}
impl ExprIfIntOps {
    pub fn items(&self) -> Ref<'_, Vec<i8>> {
        self.items.borrow()
    }
}
impl ExprIfIntOps {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
