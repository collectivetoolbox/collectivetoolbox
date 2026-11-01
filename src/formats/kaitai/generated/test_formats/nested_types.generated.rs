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
pub struct NestedTypes {
    pub(crate) _root: SharedType<NestedTypes>,
    pub(crate) _parent: SharedType<NestedTypes>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<OptRc<NestedTypes_SubtypeA>>,
    two: RefCell<OptRc<NestedTypes_SubtypeB>>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&NestedTypes> for OptRc<NestedTypes> {
    type Error = KError;
    fn try_from(v: &NestedTypes) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&NestedTypes> for OptRc<NestedTypes> {
    type Error = KError;
    fn try_from(v: &&NestedTypes) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<NestedTypes> for NestedTypes {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<NestedTypes> for &NestedTypes {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<NestedTypes> for OptRc<NestedTypes> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<NestedTypes> for &OptRc<NestedTypes> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for NestedTypes {
    type Root = NestedTypes;
    type Parent = NestedTypes;

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
        let t = Self::read_into::<_, NestedTypes_SubtypeA>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.one.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        let t = Self::read_into::<_, NestedTypes_SubtypeB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.two.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes {
}
impl NestedTypes {
    pub fn one(&self) -> Ref<'_, OptRc<NestedTypes_SubtypeA>> {
        self.one.borrow()
    }
}
impl NestedTypes {
    pub fn two(&self) -> Ref<'_, OptRc<NestedTypes_SubtypeB>> {
        self.two.borrow()
    }
}
impl NestedTypes {
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
pub struct NestedTypes_SubtypeA {
    pub(crate) _root: SharedType<NestedTypes>,
    pub(crate) _parent: SharedType<NestedTypes>,
    pub(crate) _self_shared: SharedType<Self>,
    typed_at_root: RefCell<OptRc<NestedTypes_SubtypeB>>,
    typed_here: RefCell<OptRc<NestedTypes_SubtypeA_SubtypeC>>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&NestedTypes_SubtypeA> for OptRc<NestedTypes_SubtypeA> {
    type Error = KError;
    fn try_from(v: &NestedTypes_SubtypeA) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&NestedTypes_SubtypeA> for OptRc<NestedTypes_SubtypeA> {
    type Error = KError;
    fn try_from(v: &&NestedTypes_SubtypeA) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA> for NestedTypes_SubtypeA {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA> for &NestedTypes_SubtypeA {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA> for OptRc<NestedTypes_SubtypeA> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA> for &OptRc<NestedTypes_SubtypeA> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for NestedTypes_SubtypeA {
    type Root = NestedTypes;
    type Parent = NestedTypes;

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
        let t = Self::read_into::<_, NestedTypes_SubtypeB>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.typed_at_root.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        let t = Self::read_into::<_, NestedTypes_SubtypeA_SubtypeC>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.typed_here.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes_SubtypeA {
}
impl NestedTypes_SubtypeA {
    pub fn typed_at_root(&self) -> Ref<'_, OptRc<NestedTypes_SubtypeB>> {
        self.typed_at_root.borrow()
    }
}
impl NestedTypes_SubtypeA {
    pub fn typed_here(&self) -> Ref<'_, OptRc<NestedTypes_SubtypeA_SubtypeC>> {
        self.typed_here.borrow()
    }
}
impl NestedTypes_SubtypeA {
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
pub struct NestedTypes_SubtypeA_SubtypeC {
    pub(crate) _root: SharedType<NestedTypes>,
    pub(crate) _parent: SharedType<NestedTypes_SubtypeA>,
    pub(crate) _self_shared: SharedType<Self>,
    value_c: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&NestedTypes_SubtypeA_SubtypeC> for OptRc<NestedTypes_SubtypeA_SubtypeC> {
    type Error = KError;
    fn try_from(v: &NestedTypes_SubtypeA_SubtypeC) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&NestedTypes_SubtypeA_SubtypeC> for OptRc<NestedTypes_SubtypeA_SubtypeC> {
    type Error = KError;
    fn try_from(v: &&NestedTypes_SubtypeA_SubtypeC) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA_SubtypeC> for NestedTypes_SubtypeA_SubtypeC {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA_SubtypeC>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA_SubtypeC> for &NestedTypes_SubtypeA_SubtypeC {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA_SubtypeC>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA_SubtypeC> for OptRc<NestedTypes_SubtypeA_SubtypeC> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA_SubtypeC>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<NestedTypes_SubtypeA_SubtypeC> for &OptRc<NestedTypes_SubtypeA_SubtypeC> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeA_SubtypeC>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for NestedTypes_SubtypeA_SubtypeC {
    type Root = NestedTypes;
    type Parent = NestedTypes_SubtypeA;

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
        *self_rc.value_c.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes_SubtypeA_SubtypeC {
}
impl NestedTypes_SubtypeA_SubtypeC {
    pub fn value_c(&self) -> Ref<'_, i8> {
        self.value_c.borrow()
    }
}
impl NestedTypes_SubtypeA_SubtypeC {
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
pub struct NestedTypes_SubtypeB {
    pub(crate) _root: SharedType<NestedTypes>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    value_b: RefCell<i8>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&NestedTypes_SubtypeB> for OptRc<NestedTypes_SubtypeB> {
    type Error = KError;
    fn try_from(v: &NestedTypes_SubtypeB) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&NestedTypes_SubtypeB> for OptRc<NestedTypes_SubtypeB> {
    type Error = KError;
    fn try_from(v: &&NestedTypes_SubtypeB) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeB> for NestedTypes_SubtypeB {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeB>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeB> for &NestedTypes_SubtypeB {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeB>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<NestedTypes_SubtypeB> for OptRc<NestedTypes_SubtypeB> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeB>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<NestedTypes_SubtypeB> for &OptRc<NestedTypes_SubtypeB> {
    fn downcast_optrc(&self) -> Result<OptRc<NestedTypes_SubtypeB>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for NestedTypes_SubtypeB {
    type Root = NestedTypes;
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
        *self_rc.value_b.borrow_mut() = _io.read_s1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl NestedTypes_SubtypeB {
}
impl NestedTypes_SubtypeB {
    pub fn value_b(&self) -> Ref<'_, i8> {
        self.value_b.borrow()
    }
}
impl NestedTypes_SubtypeB {
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
