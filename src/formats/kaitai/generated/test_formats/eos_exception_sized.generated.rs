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
pub struct EosExceptionSized {
    pub(crate) _root: SharedType<EosExceptionSized>,
    pub(crate) _parent: SharedType<EosExceptionSized>,
    pub(crate) _self_shared: SharedType<Self>,
    envelope: RefCell<OptRc<EosExceptionSized_Data>>,
    _io: RefCell<BytesReader>,
    envelope_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&EosExceptionSized> for OptRc<EosExceptionSized> {
    type Error = KError;
    fn try_from(v: &EosExceptionSized) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&EosExceptionSized> for OptRc<EosExceptionSized> {
    type Error = KError;
    fn try_from(v: &&EosExceptionSized) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized> for EosExceptionSized {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<EosExceptionSized> for &EosExceptionSized {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized> for OptRc<EosExceptionSized> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<EosExceptionSized> for &OptRc<EosExceptionSized> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for EosExceptionSized {
    type Root = EosExceptionSized;
    type Parent = EosExceptionSized;

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
        let _raw_envelope = _io.read_bytes(6_usize)?;
        *self_rc.envelope_raw.borrow_mut() = _raw_envelope.clone();
        let _io_envelope = BytesReader::from(_raw_envelope);
        let t = Self::read_into::<BytesReader, EosExceptionSized_Data>(&_io_envelope, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.envelope.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EosExceptionSized {
}
impl EosExceptionSized {
    pub fn envelope(&self) -> Ref<'_, OptRc<EosExceptionSized_Data>> {
        self.envelope.borrow()
    }
}
impl EosExceptionSized {
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
impl EosExceptionSized {
    pub fn envelope_raw(&self) -> Ref<'_, Vec<u8>> {
        self.envelope_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EosExceptionSized_Data {
    pub(crate) _root: SharedType<EosExceptionSized>,
    pub(crate) _parent: SharedType<EosExceptionSized>,
    pub(crate) _self_shared: SharedType<Self>,
    buf: RefCell<OptRc<EosExceptionSized_Foo>>,
    _io: RefCell<BytesReader>,
    buf_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&EosExceptionSized_Data> for OptRc<EosExceptionSized_Data> {
    type Error = KError;
    fn try_from(v: &EosExceptionSized_Data) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&EosExceptionSized_Data> for OptRc<EosExceptionSized_Data> {
    type Error = KError;
    fn try_from(v: &&EosExceptionSized_Data) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Data> for EosExceptionSized_Data {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Data>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Data> for &EosExceptionSized_Data {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Data>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Data> for OptRc<EosExceptionSized_Data> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Data>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<EosExceptionSized_Data> for &OptRc<EosExceptionSized_Data> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Data>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for EosExceptionSized_Data {
    type Root = EosExceptionSized;
    type Parent = EosExceptionSized;

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
        let _raw_buf = _io.read_bytes(7_usize)?;
        *self_rc.buf_raw.borrow_mut() = _raw_buf.clone();
        let _io_buf = BytesReader::from(_raw_buf);
        let t = Self::read_into::<BytesReader, EosExceptionSized_Foo>(&_io_buf, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.buf.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl EosExceptionSized_Data {
}
impl EosExceptionSized_Data {
    pub fn buf(&self) -> Ref<'_, OptRc<EosExceptionSized_Foo>> {
        self.buf.borrow()
    }
}
impl EosExceptionSized_Data {
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
impl EosExceptionSized_Data {
    pub fn buf_raw(&self) -> Ref<'_, Vec<u8>> {
        self.buf_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct EosExceptionSized_Foo {
    pub(crate) _root: SharedType<EosExceptionSized>,
    pub(crate) _parent: SharedType<EosExceptionSized_Data>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&EosExceptionSized_Foo> for OptRc<EosExceptionSized_Foo> {
    type Error = KError;
    fn try_from(v: &EosExceptionSized_Foo) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&EosExceptionSized_Foo> for OptRc<EosExceptionSized_Foo> {
    type Error = KError;
    fn try_from(v: &&EosExceptionSized_Foo) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Foo> for EosExceptionSized_Foo {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Foo>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Foo> for &EosExceptionSized_Foo {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Foo>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<EosExceptionSized_Foo> for OptRc<EosExceptionSized_Foo> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Foo>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<EosExceptionSized_Foo> for &OptRc<EosExceptionSized_Foo> {
    fn downcast_optrc(&self) -> Result<OptRc<EosExceptionSized_Foo>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for EosExceptionSized_Foo {
    type Root = EosExceptionSized;
    type Parent = EosExceptionSized_Data;

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
impl EosExceptionSized_Foo {
}
impl EosExceptionSized_Foo {
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
