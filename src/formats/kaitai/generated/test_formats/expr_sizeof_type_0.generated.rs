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
pub struct ExprSizeofType0 {
    pub(crate) _root: SharedType<ExprSizeofType0>,
    pub(crate) _parent: SharedType<ExprSizeofType0>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_sizeof_block: Cell<bool>,
    sizeof_block: RefCell<i32>,
}
impl TryFrom<&ExprSizeofType0> for OptRc<ExprSizeofType0> {
    type Error = KError;
    fn try_from(v: &ExprSizeofType0) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&ExprSizeofType0> for OptRc<ExprSizeofType0> {
    type Error = KError;
    fn try_from(v: &&ExprSizeofType0) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0> for ExprSizeofType0 {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0> for &ExprSizeofType0 {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0> for OptRc<ExprSizeofType0> {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<ExprSizeofType0> for &OptRc<ExprSizeofType0> {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for ExprSizeofType0 {
    type Root = ExprSizeofType0;
    type Parent = ExprSizeofType0;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofType0 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn sizeof_block(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sizeof_block.get() {
            return Ok(self.sizeof_block.borrow());
        }
        self.f_sizeof_block.set(true);
        *self.sizeof_block.borrow_mut() = (7_i32).try_into()?;
        Ok(self.sizeof_block.borrow())
    }
}
impl ExprSizeofType0 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExprSizeofType0_Block {
    pub(crate) _root: SharedType<ExprSizeofType0>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    a: RefCell<u8>,
    b: RefCell<u32>,
    c: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    c_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&ExprSizeofType0_Block> for OptRc<ExprSizeofType0_Block> {
    type Error = KError;
    fn try_from(v: &ExprSizeofType0_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&ExprSizeofType0_Block> for OptRc<ExprSizeofType0_Block> {
    type Error = KError;
    fn try_from(v: &&ExprSizeofType0_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0_Block> for ExprSizeofType0_Block {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0_Block>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0_Block> for &ExprSizeofType0_Block {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0_Block>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<ExprSizeofType0_Block> for OptRc<ExprSizeofType0_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0_Block>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<ExprSizeofType0_Block> for &OptRc<ExprSizeofType0_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<ExprSizeofType0_Block>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for ExprSizeofType0_Block {
    type Root = ExprSizeofType0;
    type Parent = KStructUnit;

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
        *self_rc.a.borrow_mut() = _io.read_u1()?;
        *self_rc.b.borrow_mut() = _io.read_u4le()?;
        *self_rc.c.borrow_mut() = _io.read_bytes(2_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprSizeofType0_Block {
}
impl ExprSizeofType0_Block {
    pub fn a(&self) -> Ref<'_, u8> {
        self.a.borrow()
    }
}
impl ExprSizeofType0_Block {
    pub fn b(&self) -> Ref<'_, u32> {
        self.b.borrow()
    }
}
impl ExprSizeofType0_Block {
    pub fn c(&self) -> Ref<'_, Vec<u8>> {
        self.c.borrow()
    }
}
impl ExprSizeofType0_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
    pub fn _parent(&self) -> Option<OptRc<<Self as KStruct>::Parent>> {
        self._parent.get().ok()
    }
    pub fn _root(&self) -> Option<OptRc<<Self as KStruct>::Root>> {
        self._root.get().ok()
    }
}
impl ExprSizeofType0_Block {
    pub fn c_raw(&self) -> Ref<'_, Vec<u8>> {
        self.c_raw.borrow()
    }
}
