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
pub struct SwitchManualIntSizeElse {
    pub(crate) _root: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _self_shared: SharedType<Self>,
    chunks: RefCell<Vec<OptRc<SwitchManualIntSizeElse_Chunk>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntSizeElse {
    type Root = SwitchManualIntSizeElse;
    type Parent = SwitchManualIntSizeElse;

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
                let t = Self::read_into::<_, SwitchManualIntSizeElse_Chunk>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.chunks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl SwitchManualIntSizeElse {
}
impl SwitchManualIntSizeElse {
    pub fn chunks(&self) -> Ref<'_, Vec<OptRc<SwitchManualIntSizeElse_Chunk>>> {
        self.chunks.borrow()
    }
}
impl SwitchManualIntSizeElse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeElse_Chunk {
    pub(crate) _root: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<u8>,
    size: RefCell<u32>,
    body: RefCell<Option<SwitchManualIntSizeElse_Chunk_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum SwitchManualIntSizeElse_Chunk_Body {
    SwitchManualIntSizeElse_Chunk_ChunkMeta(OptRc<SwitchManualIntSizeElse_Chunk_ChunkMeta>),
    SwitchManualIntSizeElse_Chunk_ChunkDir(OptRc<SwitchManualIntSizeElse_Chunk_ChunkDir>),
    SwitchManualIntSizeElse_Chunk_Dummy(OptRc<SwitchManualIntSizeElse_Chunk_Dummy>),
}
impl TryFrom<&SwitchManualIntSizeElse_Chunk_Body> for OptRc<SwitchManualIntSizeElse_Chunk_ChunkMeta> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeElse_Chunk_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeElse_Chunk_Body::SwitchManualIntSizeElse_Chunk_ChunkMeta(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntSizeElse_Chunk_ChunkMeta>> for SwitchManualIntSizeElse_Chunk_Body {
    fn from(v: OptRc<SwitchManualIntSizeElse_Chunk_ChunkMeta>) -> Self {
        Self::SwitchManualIntSizeElse_Chunk_ChunkMeta(v)
    }
}
impl TryFrom<&SwitchManualIntSizeElse_Chunk_Body> for OptRc<SwitchManualIntSizeElse_Chunk_ChunkDir> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeElse_Chunk_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeElse_Chunk_Body::SwitchManualIntSizeElse_Chunk_ChunkDir(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntSizeElse_Chunk_ChunkDir>> for SwitchManualIntSizeElse_Chunk_Body {
    fn from(v: OptRc<SwitchManualIntSizeElse_Chunk_ChunkDir>) -> Self {
        Self::SwitchManualIntSizeElse_Chunk_ChunkDir(v)
    }
}
impl TryFrom<&SwitchManualIntSizeElse_Chunk_Body> for OptRc<SwitchManualIntSizeElse_Chunk_Dummy> {
    type Error = KError;
    fn try_from(v: &SwitchManualIntSizeElse_Chunk_Body) -> Result<Self, Self::Error> {
        if let SwitchManualIntSizeElse_Chunk_Body::SwitchManualIntSizeElse_Chunk_Dummy(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<SwitchManualIntSizeElse_Chunk_Dummy>> for SwitchManualIntSizeElse_Chunk_Body {
    fn from(v: OptRc<SwitchManualIntSizeElse_Chunk_Dummy>) -> Self {
        Self::SwitchManualIntSizeElse_Chunk_Dummy(v)
    }
}
impl KStruct for SwitchManualIntSizeElse_Chunk {
    type Root = SwitchManualIntSizeElse;
    type Parent = SwitchManualIntSizeElse;

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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        match *self_rc.code() {
            17 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.size())?)?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntSizeElse_Chunk_ChunkMeta>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            34 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.size())?)?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntSizeElse_Chunk_ChunkDir>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.size())?)?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, SwitchManualIntSizeElse_Chunk_Dummy>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
        }
        Ok(())
    }
}
impl SwitchManualIntSizeElse_Chunk {
}
impl SwitchManualIntSizeElse_Chunk {
    pub fn code(&self) -> Ref<'_, u8> {
        self.code.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk {
    pub fn body(&self) -> Ref<'_, Option<SwitchManualIntSizeElse_Chunk_Body>> {
        self.body.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeElse_Chunk_ChunkDir {
    pub(crate) _root: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeElse_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<String>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntSizeElse_Chunk_ChunkDir {
    type Root = SwitchManualIntSizeElse;
    type Parent = SwitchManualIntSizeElse_Chunk;

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
        Ok(())
    }
}
impl SwitchManualIntSizeElse_Chunk_ChunkDir {
}
impl SwitchManualIntSizeElse_Chunk_ChunkDir {
    pub fn entries(&self) -> Ref<'_, Vec<String>> {
        self.entries.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk_ChunkDir {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeElse_Chunk_ChunkMeta {
    pub(crate) _root: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeElse_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    title: RefCell<String>,
    author: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntSizeElse_Chunk_ChunkMeta {
    type Root = SwitchManualIntSizeElse;
    type Parent = SwitchManualIntSizeElse_Chunk;

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
        *self_rc.author.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        Ok(())
    }
}
impl SwitchManualIntSizeElse_Chunk_ChunkMeta {
}
impl SwitchManualIntSizeElse_Chunk_ChunkMeta {
    pub fn title(&self) -> Ref<'_, String> {
        self.title.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk_ChunkMeta {
    pub fn author(&self) -> Ref<'_, String> {
        self.author.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk_ChunkMeta {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SwitchManualIntSizeElse_Chunk_Dummy {
    pub(crate) _root: SharedType<SwitchManualIntSizeElse>,
    pub(crate) _parent: SharedType<SwitchManualIntSizeElse_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    rest: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for SwitchManualIntSizeElse_Chunk_Dummy {
    type Root = SwitchManualIntSizeElse;
    type Parent = SwitchManualIntSizeElse_Chunk;

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
        *self_rc.rest.borrow_mut() = _io.read_bytes_full()?;
        Ok(())
    }
}
impl SwitchManualIntSizeElse_Chunk_Dummy {
}
impl SwitchManualIntSizeElse_Chunk_Dummy {
    pub fn rest(&self) -> Ref<'_, Vec<u8>> {
        self.rest.borrow()
    }
}
impl SwitchManualIntSizeElse_Chunk_Dummy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
