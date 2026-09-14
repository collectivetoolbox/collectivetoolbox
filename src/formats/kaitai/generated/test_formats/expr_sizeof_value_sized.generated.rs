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
pub struct ExprSizeofValueSized {
    pub(crate) _root: SharedType<ExprSizeofValueSized>,
    pub(crate) _parent: SharedType<ExprSizeofValueSized>,
    pub(crate) _self_shared: SharedType<Self>,
    block1: RefCell<OptRc<ExprSizeofValueSized_Block>>,
    more: RefCell<u16>,
    _io: RefCell<BytesReader>,
    block1_raw: RefCell<Vec<u8>>,
    f_self_sizeof: Cell<bool>,
    self_sizeof: RefCell<i32>,
    f_sizeof_block: Cell<bool>,
    sizeof_block: RefCell<i32>,
    f_sizeof_block_a: Cell<bool>,
    sizeof_block_a: RefCell<i32>,
    f_sizeof_block_b: Cell<bool>,
    sizeof_block_b: RefCell<i32>,
    f_sizeof_block_c: Cell<bool>,
    sizeof_block_c: RefCell<i32>,
}
impl KStruct for ExprSizeofValueSized {
    type Root = ExprSizeofValueSized;
    type Parent = ExprSizeofValueSized;

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
        let _raw_block1 = _io.read_bytes(12_usize)?;
        *self_rc.block1_raw.borrow_mut() = _raw_block1.clone();
        let _io_block1 = BytesReader::from(_raw_block1);
        let t = Self::read_into::<BytesReader, ExprSizeofValueSized_Block>(&_io_block1, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.block1.borrow_mut() = t;
        *self_rc.more.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofValueSized {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn self_sizeof(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_self_sizeof.get() {
            return Ok(self.self_sizeof.borrow());
        }
        self.f_self_sizeof.set(true);
        *self.self_sizeof.borrow_mut() = (14_i32).try_into()?;
        Ok(self.self_sizeof.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block.get() {
            return Ok(self.sizeof_block.borrow());
        }
        self.f_sizeof_block.set(true);
        *self.sizeof_block.borrow_mut() = (12_i32).try_into()?;
        Ok(self.sizeof_block.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block_a(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block_a.get() {
            return Ok(self.sizeof_block_a.borrow());
        }
        self.f_sizeof_block_a.set(true);
        *self.sizeof_block_a.borrow_mut() = (1_i32).try_into()?;
        Ok(self.sizeof_block_a.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block_b(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block_b.get() {
            return Ok(self.sizeof_block_b.borrow());
        }
        self.f_sizeof_block_b.set(true);
        *self.sizeof_block_b.borrow_mut() = (4_i32).try_into()?;
        Ok(self.sizeof_block_b.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block_c(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block_c.get() {
            return Ok(self.sizeof_block_c.borrow());
        }
        self.f_sizeof_block_c.set(true);
        *self.sizeof_block_c.borrow_mut() = (2_i32).try_into()?;
        Ok(self.sizeof_block_c.borrow())
    }
}
impl ExprSizeofValueSized {
    pub fn block1(&self) -> Ref<'_, OptRc<ExprSizeofValueSized_Block>> {
        self.block1.borrow()
    }
}
impl ExprSizeofValueSized {
    pub fn more(&self) -> Ref<'_, u16> {
        self.more.borrow()
    }
}
impl ExprSizeofValueSized {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprSizeofValueSized {
    pub fn block1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.block1_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprSizeofValueSized_Block {
    pub(crate) _root: SharedType<ExprSizeofValueSized>,
    pub(crate) _parent: SharedType<ExprSizeofValueSized>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u8>,
    b: RefCell<u32>,
    c: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    c_raw: RefCell<Vec<u8>>,
}
impl KStruct for ExprSizeofValueSized_Block {
    type Root = ExprSizeofValueSized;
    type Parent = ExprSizeofValueSized;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofValueSized_Block {
}
impl ExprSizeofValueSized_Block {
    pub fn a(&self) -> Ref<'_, u8> {
        self.a.borrow()
    }
}
impl ExprSizeofValueSized_Block {
    pub fn b(&self) -> Ref<'_, u32> {
        self.b.borrow()
    }
}
impl ExprSizeofValueSized_Block {
    pub fn c(&self) -> Ref<'_, Vec<u8>> {
        self.c.borrow()
    }
}
impl ExprSizeofValueSized_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprSizeofValueSized_Block {
    pub fn c_raw(&self) -> Ref<'_, Vec<u8>> {
        self.c_raw.borrow()
    }
}
