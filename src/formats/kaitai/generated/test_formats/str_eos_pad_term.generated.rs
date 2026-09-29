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
pub struct StrEosPadTerm {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    str_pad: RefCell<OptRc<StrEosPadTerm_StrPadType>>,
    str_term: RefCell<OptRc<StrEosPadTerm_StrTermType>>,
    str_term_and_pad: RefCell<OptRc<StrEosPadTerm_StrTermAndPadType>>,
    str_term_include: RefCell<OptRc<StrEosPadTerm_StrTermIncludeType>>,
    _io: RefCell<BytesReader>,
    str_pad_raw: RefCell<Vec<u8>>,
    str_term_raw: RefCell<Vec<u8>>,
    str_term_and_pad_raw: RefCell<Vec<u8>>,
    str_term_include_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&StrEosPadTerm> for OptRc<StrEosPadTerm> {
    type Error = KError;
    fn try_from(v: &StrEosPadTerm) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&StrEosPadTerm> for OptRc<StrEosPadTerm> {
    type Error = KError;
    fn try_from(v: &&StrEosPadTerm) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm> for StrEosPadTerm {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm> for &StrEosPadTerm {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm> for OptRc<StrEosPadTerm> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<StrEosPadTerm> for &OptRc<StrEosPadTerm> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for StrEosPadTerm {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        let _raw_str_pad = _io.read_bytes(20_usize)?;
        *self_rc.str_pad_raw.borrow_mut() = _raw_str_pad.clone();
        let _io_str_pad = BytesReader::from(_raw_str_pad);
        let t = Self::read_into::<BytesReader, StrEosPadTerm_StrPadType>(&_io_str_pad, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_pad.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        let _raw_str_term = _io.read_bytes(20_usize)?;
        *self_rc.str_term_raw.borrow_mut() = _raw_str_term.clone();
        let _io_str_term = BytesReader::from(_raw_str_term);
        let t = Self::read_into::<BytesReader, StrEosPadTerm_StrTermType>(&_io_str_term, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        let _raw_str_term_and_pad = _io.read_bytes(20_usize)?;
        *self_rc.str_term_and_pad_raw.borrow_mut() = _raw_str_term_and_pad.clone();
        let _io_str_term_and_pad = BytesReader::from(_raw_str_term_and_pad);
        let t = Self::read_into::<BytesReader, StrEosPadTerm_StrTermAndPadType>(&_io_str_term_and_pad, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_and_pad.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        let _raw_str_term_include = _io.read_bytes(20_usize)?;
        *self_rc.str_term_include_raw.borrow_mut() = _raw_str_term_include.clone();
        let _io_str_term_include = BytesReader::from(_raw_str_term_include);
        let t = Self::read_into::<BytesReader, StrEosPadTerm_StrTermIncludeType>(&_io_str_term_include, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str_term_include.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTerm {
}
impl StrEosPadTerm {
    pub fn str_pad(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrPadType>> {
        self.str_pad.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermType>> {
        self.str_term.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_and_pad(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermAndPadType>> {
        self.str_term_and_pad.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_include(&self) -> Ref<'_, OptRc<StrEosPadTerm_StrTermIncludeType>> {
        self.str_term_include.borrow()
    }
}
impl StrEosPadTerm {
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
impl StrEosPadTerm {
    pub fn str_pad_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_pad_raw.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_raw.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_and_pad_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_and_pad_raw.borrow()
    }
}
impl StrEosPadTerm {
    pub fn str_term_include_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_term_include_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct StrEosPadTerm_StrPadType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&StrEosPadTerm_StrPadType> for OptRc<StrEosPadTerm_StrPadType> {
    type Error = KError;
    fn try_from(v: &StrEosPadTerm_StrPadType) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&StrEosPadTerm_StrPadType> for OptRc<StrEosPadTerm_StrPadType> {
    type Error = KError;
    fn try_from(v: &&StrEosPadTerm_StrPadType) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrPadType> for StrEosPadTerm_StrPadType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrPadType>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrPadType> for &StrEosPadTerm_StrPadType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrPadType>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrPadType> for OptRc<StrEosPadTerm_StrPadType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrPadType>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<StrEosPadTerm_StrPadType> for &OptRc<StrEosPadTerm_StrPadType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrPadType>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for StrEosPadTerm_StrPadType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, None, false, Some(64)), "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTerm_StrPadType {
}
impl StrEosPadTerm_StrPadType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrPadType {
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
pub struct StrEosPadTerm_StrTermAndPadType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&StrEosPadTerm_StrTermAndPadType> for OptRc<StrEosPadTerm_StrTermAndPadType> {
    type Error = KError;
    fn try_from(v: &StrEosPadTerm_StrTermAndPadType) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&StrEosPadTerm_StrTermAndPadType> for OptRc<StrEosPadTerm_StrTermAndPadType> {
    type Error = KError;
    fn try_from(v: &&StrEosPadTerm_StrTermAndPadType) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermAndPadType> for StrEosPadTerm_StrTermAndPadType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermAndPadType>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermAndPadType> for &StrEosPadTerm_StrTermAndPadType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermAndPadType>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermAndPadType> for OptRc<StrEosPadTerm_StrTermAndPadType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermAndPadType>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermAndPadType> for &OptRc<StrEosPadTerm_StrTermAndPadType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermAndPadType>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for StrEosPadTerm_StrTermAndPadType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(64), false, Some(43)), "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTerm_StrTermAndPadType {
}
impl StrEosPadTerm_StrTermAndPadType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermAndPadType {
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
pub struct StrEosPadTerm_StrTermIncludeType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&StrEosPadTerm_StrTermIncludeType> for OptRc<StrEosPadTerm_StrTermIncludeType> {
    type Error = KError;
    fn try_from(v: &StrEosPadTerm_StrTermIncludeType) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&StrEosPadTerm_StrTermIncludeType> for OptRc<StrEosPadTerm_StrTermIncludeType> {
    type Error = KError;
    fn try_from(v: &&StrEosPadTerm_StrTermIncludeType) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermIncludeType> for StrEosPadTerm_StrTermIncludeType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermIncludeType>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermIncludeType> for &StrEosPadTerm_StrTermIncludeType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermIncludeType>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermIncludeType> for OptRc<StrEosPadTerm_StrTermIncludeType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermIncludeType>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermIncludeType> for &OptRc<StrEosPadTerm_StrTermIncludeType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermIncludeType>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for StrEosPadTerm_StrTermIncludeType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(64), true, None), "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTerm_StrTermIncludeType {
}
impl StrEosPadTerm_StrTermIncludeType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermIncludeType {
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
pub struct StrEosPadTerm_StrTermType {
    pub(crate) _root: SharedType<StrEosPadTerm>,
    pub(crate) _parent: SharedType<StrEosPadTerm>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&StrEosPadTerm_StrTermType> for OptRc<StrEosPadTerm_StrTermType> {
    type Error = KError;
    fn try_from(v: &StrEosPadTerm_StrTermType) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&StrEosPadTerm_StrTermType> for OptRc<StrEosPadTerm_StrTermType> {
    type Error = KError;
    fn try_from(v: &&StrEosPadTerm_StrTermType) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermType> for StrEosPadTerm_StrTermType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermType>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermType> for &StrEosPadTerm_StrTermType {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermType>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermType> for OptRc<StrEosPadTerm_StrTermType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermType>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<StrEosPadTerm_StrTermType> for &OptRc<StrEosPadTerm_StrTermType> {
    fn downcast_optrc(&self) -> Result<OptRc<StrEosPadTerm_StrTermType>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for StrEosPadTerm_StrTermType {
    type Root = StrEosPadTerm;
    type Parent = StrEosPadTerm;

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
        *self_rc.value.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes_full()?, Some(64), false, None), "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl StrEosPadTerm_StrTermType {
}
impl StrEosPadTerm_StrTermType {
    pub fn value(&self) -> Ref<'_, String> {
        self.value.borrow()
    }
}
impl StrEosPadTerm_StrTermType {
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
