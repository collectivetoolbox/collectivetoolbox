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
pub struct ExprSizeofType1 {
    pub(crate) _root: SharedType<ExprSizeofType1>,
    pub(crate) _parent: SharedType<ExprSizeofType1>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_sizeof_block: Cell<bool>,
    sizeof_block: RefCell<i32>,
    f_sizeof_subblock: Cell<bool>,
    sizeof_subblock: RefCell<i32>,
}
impl KStruct for ExprSizeofType1 {
    type Root = ExprSizeofType1;
    type Parent = ExprSizeofType1;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofType1 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block.get() {
            return Ok(self.sizeof_block.borrow());
        }
        self.f_sizeof_block.set(true);
        *self.sizeof_block.borrow_mut() = (11_i32).try_into()?;
        Ok(self.sizeof_block.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_subblock(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_subblock.get() {
            return Ok(self.sizeof_subblock.borrow());
        }
        self.f_sizeof_subblock.set(true);
        *self.sizeof_subblock.borrow_mut() = (4_i32).try_into()?;
        Ok(self.sizeof_subblock.borrow())
    }
}
impl ExprSizeofType1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprSizeofType1_Block {
    pub(crate) _root: SharedType<ExprSizeofType1>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u8>,
    b: RefCell<u32>,
    c: RefCell<Vec<u8>>,
    d: RefCell<OptRc<ExprSizeofType1_Block_Subblock>>,
    _io: RefCell<BytesReader>,
    c_raw: RefCell<Vec<u8>>,
}
impl KStruct for ExprSizeofType1_Block {
    type Root = ExprSizeofType1;
    type Parent = KStructUnit;

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
        *self_rc.a.borrow_mut() = _io.read_u1()?;
        *self_rc.b.borrow_mut() = _io.read_u4le()?;
        *self_rc.c.borrow_mut() = _io.read_bytes(2_usize)?;
        let t = Self::read_into::<_, ExprSizeofType1_Block_Subblock>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.d.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofType1_Block {
}
impl ExprSizeofType1_Block {
    pub fn a(&self) -> Ref<'_, u8> {
        self.a.borrow()
    }
}
impl ExprSizeofType1_Block {
    pub fn b(&self) -> Ref<'_, u32> {
        self.b.borrow()
    }
}
impl ExprSizeofType1_Block {
    pub fn c(&self) -> Ref<'_, Vec<u8>> {
        self.c.borrow()
    }
}
impl ExprSizeofType1_Block {
    pub fn d(&self) -> Ref<'_, OptRc<ExprSizeofType1_Block_Subblock>> {
        self.d.borrow()
    }
}
impl ExprSizeofType1_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprSizeofType1_Block {
    pub fn c_raw(&self) -> Ref<'_, Vec<u8>> {
        self.c_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprSizeofType1_Block_Subblock {
    pub(crate) _root: SharedType<ExprSizeofType1>,
    pub(crate) _parent: SharedType<ExprSizeofType1_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    a_raw: RefCell<Vec<u8>>,
}
impl KStruct for ExprSizeofType1_Block_Subblock {
    type Root = ExprSizeofType1;
    type Parent = ExprSizeofType1_Block;

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
        *self_rc.a.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofType1_Block_Subblock {
}
impl ExprSizeofType1_Block_Subblock {
    pub fn a(&self) -> Ref<'_, Vec<u8>> {
        self.a.borrow()
    }
}
impl ExprSizeofType1_Block_Subblock {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprSizeofType1_Block_Subblock {
    pub fn a_raw(&self) -> Ref<'_, Vec<u8>> {
        self.a_raw.borrow()
    }
}
