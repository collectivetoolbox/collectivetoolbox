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
pub struct SwitchManualIntSizeEos {
    pub(crate) _root: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _self_shared: SharedType<Self>,
    chunks: RefCell<Vec<OptRc<SwitchManualIntSizeEos_Chunk>>>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&SwitchManualIntSizeEos> for OptRc<SwitchManualIntSizeEos> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&SwitchManualIntSizeEos> for OptRc<SwitchManualIntSizeEos> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos> for SwitchManualIntSizeEos {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos> for &SwitchManualIntSizeEos {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos> for OptRc<SwitchManualIntSizeEos> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos> for &OptRc<SwitchManualIntSizeEos> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for SwitchManualIntSizeEos {
    type Root = SwitchManualIntSizeEos;
    type Parent = SwitchManualIntSizeEos;

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
        *self_rc.chunks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, SwitchManualIntSizeEos_Chunk>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.chunks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualIntSizeEos {
}
impl SwitchManualIntSizeEos {
    pub fn chunks(&self) -> Ref<'_, Vec<OptRc<SwitchManualIntSizeEos_Chunk>>> {
        self.chunks.borrow()
    }
}
impl SwitchManualIntSizeEos {
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
pub struct SwitchManualIntSizeEos_Chunk {
    pub(crate) _root: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    size: RefCell<u32>,
    body: RefCell<OptRc<SwitchManualIntSizeEos_ChunkBody>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&SwitchManualIntSizeEos_Chunk> for OptRc<SwitchManualIntSizeEos_Chunk> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_Chunk) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_Chunk> for OptRc<SwitchManualIntSizeEos_Chunk> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_Chunk) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_Chunk> for SwitchManualIntSizeEos_Chunk {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_Chunk>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_Chunk> for &SwitchManualIntSizeEos_Chunk {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_Chunk>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_Chunk> for OptRc<SwitchManualIntSizeEos_Chunk> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_Chunk>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_Chunk> for &OptRc<SwitchManualIntSizeEos_Chunk> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_Chunk>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for SwitchManualIntSizeEos_Chunk {
    type Root = SwitchManualIntSizeEos;
    type Parent = SwitchManualIntSizeEos;

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
        *self_rc.code.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = _io.clone();
        let _raw_body = _io.read_bytes(usize::try_from(*self_rc.size())?)?;
        *self_rc.body_raw.borrow_mut() = _raw_body.clone();
        let _io_body = BytesReader::from(_raw_body);
        let t = Self::read_into::<BytesReader, SwitchManualIntSizeEos_ChunkBody>(&_io_body, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.body.borrow_mut() = t;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualIntSizeEos_Chunk {
}
impl SwitchManualIntSizeEos_Chunk {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchManualIntSizeEos_Chunk {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl SwitchManualIntSizeEos_Chunk {
    pub fn body(&self) -> Ref<'_, OptRc<SwitchManualIntSizeEos_ChunkBody>> {
        self.body.borrow()
    }
}
impl SwitchManualIntSizeEos_Chunk {
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
impl SwitchManualIntSizeEos_Chunk {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeEos_ChunkBody {
    pub(crate) _root: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeEos_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    body: RefCell<Option<SwitchManualIntSizeEos_ChunkBody_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualIntSizeEos_ChunkBody_Body {
    SwitchManualIntSizeEos_ChunkBody_ChunkMeta(OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>),
    SwitchManualIntSizeEos_ChunkBody_ChunkDir(OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>),
    Bytes(Vec<u8>),
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody_Body> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeEos_ChunkBody_Body::SwitchManualIntSizeEos_ChunkBody_ChunkMeta(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody_Body> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for SwitchManualIntSizeEos_ChunkBody_Body {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        OptRc::<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>::try_from(self)
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for &SwitchManualIntSizeEos_ChunkBody_Body {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>> for SwitchManualIntSizeEos_ChunkBody_Body {
    fn from(v: OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>) -> Self {
        Self::SwitchManualIntSizeEos_ChunkBody_ChunkMeta(v)
    }
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody_Body> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeEos_ChunkBody_Body::SwitchManualIntSizeEos_ChunkBody_ChunkDir(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody_Body> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for SwitchManualIntSizeEos_ChunkBody_Body {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        OptRc::<SwitchManualIntSizeEos_ChunkBody_ChunkDir>::try_from(self)
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for &SwitchManualIntSizeEos_ChunkBody_Body {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        (*self).downcast_optrc()
    }
}
impl From<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>> for SwitchManualIntSizeEos_ChunkBody_Body {
    fn from(v: OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>) -> Self {
        Self::SwitchManualIntSizeEos_ChunkBody_ChunkDir(v)
    }
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeEos_ChunkBody_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody_Body) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}
impl From<Vec<u8>> for SwitchManualIntSizeEos_ChunkBody_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody> for OptRc<SwitchManualIntSizeEos_ChunkBody> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody> for OptRc<SwitchManualIntSizeEos_ChunkBody> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody> for SwitchManualIntSizeEos_ChunkBody {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody> for &SwitchManualIntSizeEos_ChunkBody {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody> for OptRc<SwitchManualIntSizeEos_ChunkBody> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody> for &OptRc<SwitchManualIntSizeEos_ChunkBody> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for SwitchManualIntSizeEos_ChunkBody {
    type Root = SwitchManualIntSizeEos;
    type Parent = SwitchManualIntSizeEos_Chunk;

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
        match *self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.code() {
            17 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntSizeEos_ChunkBody_ChunkMeta>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            34 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes_full()?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntSizeEos_ChunkBody_ChunkDir>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualIntSizeEos_ChunkBody {
}
impl SwitchManualIntSizeEos_ChunkBody {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualIntSizeEos_ChunkBody_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualIntSizeEos_ChunkBody {
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
impl SwitchManualIntSizeEos_ChunkBody {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    pub(crate) _root: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeEos_ChunkBody>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
    entries_raw: RefCell<Vec<u8>>,
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody_ChunkDir> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody_ChunkDir) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody_ChunkDir> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody_ChunkDir) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for &SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> for &OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkDir>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    type Root = SwitchManualIntSizeEos;
    type Parent = SwitchManualIntSizeEos_ChunkBody;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                self_rc.entries.borrow_mut().push(bytes_to_str(&_io.read_bytes(4_usize)?, "UTF-8")?);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkDir {
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    pub fn entries(&self) -> Ref<'_, Vec<String>> {
        self.entries.borrow()
    }
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkDir {
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
impl SwitchManualIntSizeEos_ChunkBody_ChunkDir {
    pub fn entries_raw(&self) -> Ref<'_, Vec<u8>> {
        self.entries_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    pub(crate) _root: SharedType<SwitchManualIntSizeEos>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeEos_ChunkBody>,
    pub(crate) _self_shared: SharedType<Self>,
    title: RefCell<String>,
    author: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl TryFrom<&SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeEos_ChunkBody_ChunkMeta) -> Result<Self, Self::Error> {
        Ok(OptRc::from(v.clone()))
    }
}
impl TryFrom<&&SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    type Error = KError;
    fn try_from(v: &&SwitchManualIntSizeEos_ChunkBody_ChunkMeta) -> Result<Self, Self::Error> {
        Ok(OptRc::from((*v).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        Ok(OptRc::from(self.clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for &SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        Ok(OptRc::from((*self).clone()))
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        Ok(self.clone())
    }
}
impl DowncastOptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> for &OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta> {
    fn downcast_optrc(&self) -> Result<OptRc<SwitchManualIntSizeEos_ChunkBody_ChunkMeta>, KError> {
        Ok((*self).clone())
    }
}
impl KStruct for SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    type Root = SwitchManualIntSizeEos;
    type Parent = SwitchManualIntSizeEos_ChunkBody;

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
        *self_rc.title.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc.author.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = _io.clone();
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    pub fn title(&self) -> Ref<'_, String> {
        self.title.borrow()
    }
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
    pub fn author(&self) -> Ref<'_, String> {
        self.author.borrow()
    }
}
impl SwitchManualIntSizeEos_ChunkBody_ChunkMeta {
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
