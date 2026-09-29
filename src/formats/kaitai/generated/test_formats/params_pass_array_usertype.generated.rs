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
pub struct ParamsPassArrayUsertype {
    pub(crate) _root: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _parent: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    blocks: RefCell<Vec<OptRc<ParamsPassArrayUsertype_Block>>>,
    pass_blocks: RefCell<OptRc<ParamsPassArrayUsertype_ParamType>>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&ParamsPassArrayUsertype> for OptRc<ParamsPassArrayUsertype> {
    type Error = KError;
    fn try_from(v: &ParamsPassArrayUsertype) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&ParamsPassArrayUsertype> for OptRc<ParamsPassArrayUsertype> {
    type Error = KError;
    fn try_from(v: &&ParamsPassArrayUsertype) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype> for ParamsPassArrayUsertype {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype> for &ParamsPassArrayUsertype {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype> for OptRc<ParamsPassArrayUsertype> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype> for &OptRc<ParamsPassArrayUsertype> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for ParamsPassArrayUsertype {
    type Root = ParamsPassArrayUsertype;
    type Parent = ParamsPassArrayUsertype;

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
        *self_rc.blocks.borrow_mut() = Vec::new();
        let l_blocks = 2_usize;
        for _i in 0_usize..l_blocks {
            let t = Self::read_into::<_, ParamsPassArrayUsertype_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.blocks.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = _io.clone();
        let f = |t : &mut ParamsPassArrayUsertype_ParamType| Ok(t.set_params((self_rc.blocks()).clone()));
        let t = Self::read_into_with_init::<_, ParamsPassArrayUsertype_ParamType>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.pass_blocks.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassArrayUsertype {
}
impl ParamsPassArrayUsertype {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<ParamsPassArrayUsertype_Block>>> {
        self.blocks.borrow()
    }
}
impl ParamsPassArrayUsertype {
    pub fn pass_blocks(&self) -> Ref<'_, OptRc<ParamsPassArrayUsertype_ParamType>> {
        self.pass_blocks.borrow()
    }
}
impl ParamsPassArrayUsertype {
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
pub struct ParamsPassArrayUsertype_Block {
    pub(crate) _root: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _parent: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    foo: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&ParamsPassArrayUsertype_Block> for OptRc<ParamsPassArrayUsertype_Block> {
    type Error = KError;
    fn try_from(v: &ParamsPassArrayUsertype_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&ParamsPassArrayUsertype_Block> for OptRc<ParamsPassArrayUsertype_Block> {
    type Error = KError;
    fn try_from(v: &&ParamsPassArrayUsertype_Block) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_Block> for ParamsPassArrayUsertype_Block {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_Block>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_Block> for &ParamsPassArrayUsertype_Block {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_Block>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_Block> for OptRc<ParamsPassArrayUsertype_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_Block>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_Block> for &OptRc<ParamsPassArrayUsertype_Block> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_Block>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for ParamsPassArrayUsertype_Block {
    type Root = ParamsPassArrayUsertype;
    type Parent = ParamsPassArrayUsertype;

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
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassArrayUsertype_Block {
}
impl ParamsPassArrayUsertype_Block {
    pub fn foo(&self) -> Ref<'_, u8> {
        self.foo.borrow()
    }
}
impl ParamsPassArrayUsertype_Block {
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
pub struct ParamsPassArrayUsertype_ParamType {
    pub(crate) _root: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _parent: SharedType<ParamsPassArrayUsertype>,
    pub(crate) _self_shared: SharedType<Self>,
    bar: RefCell<Vec<OptRc<ParamsPassArrayUsertype_Block>>>,
    one: RefCell<Vec<u8>>,
    two: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    one_raw: RefCell<Vec<u8>>,
    two_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&ParamsPassArrayUsertype_ParamType> for OptRc<ParamsPassArrayUsertype_ParamType> {
    type Error = KError;
    fn try_from(v: &ParamsPassArrayUsertype_ParamType) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&ParamsPassArrayUsertype_ParamType> for OptRc<ParamsPassArrayUsertype_ParamType> {
    type Error = KError;
    fn try_from(v: &&ParamsPassArrayUsertype_ParamType) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_ParamType> for ParamsPassArrayUsertype_ParamType {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_ParamType>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_ParamType> for &ParamsPassArrayUsertype_ParamType {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_ParamType>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_ParamType> for OptRc<ParamsPassArrayUsertype_ParamType> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_ParamType>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<ParamsPassArrayUsertype_ParamType> for &OptRc<ParamsPassArrayUsertype_ParamType> {
    fn downcast_optrc(&self) -> Result<OptRc<ParamsPassArrayUsertype_ParamType>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for ParamsPassArrayUsertype_ParamType {
    type Root = ParamsPassArrayUsertype;
    type Parent = ParamsPassArrayUsertype;

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
        *self_rc.one.borrow_mut() = _io.read_bytes(usize::from(*self_rc.bar().get(0_usize).ok_or(KError::CastError)?.foo()))?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.two.borrow_mut() = _io.read_bytes(usize::from(*self_rc.bar().get(1_usize).ok_or(KError::CastError)?.foo()))?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassArrayUsertype_ParamType {
    pub fn bar(&self) -> Ref<'_, Vec<OptRc<ParamsPassArrayUsertype_Block>>> {
        self.bar.borrow()
    }
}
impl ParamsPassArrayUsertype_ParamType {
    pub fn set_params(&mut self, bar: Vec<OptRc<ParamsPassArrayUsertype_Block>>) {
        *self.bar.borrow_mut() = bar;
    }
}
impl ParamsPassArrayUsertype_ParamType {
}
impl ParamsPassArrayUsertype_ParamType {
    pub fn one(&self) -> Ref<'_, Vec<u8>> {
        self.one.borrow()
    }
}
impl ParamsPassArrayUsertype_ParamType {
    pub fn two(&self) -> Ref<'_, Vec<u8>> {
        self.two.borrow()
    }
}
impl ParamsPassArrayUsertype_ParamType {
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
impl ParamsPassArrayUsertype_ParamType {
    pub fn one_raw(&self) -> Ref<'_, Vec<u8>> {
        self.one_raw.borrow()
    }
}
impl ParamsPassArrayUsertype_ParamType {
    pub fn two_raw(&self) -> Ref<'_, Vec<u8>> {
        self.two_raw.borrow()
    }
}
