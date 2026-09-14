// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * The Resource Interchange File Format (RIFF) is a generic file container format
 * for storing data in tagged chunks. It is primarily used to store multimedia
 * such as sound and video, though it may also be used to store any arbitrary data.
 *
 * The Microsoft implementation is mostly known through container formats
 * like AVI, ANI and WAV, which use RIFF as their basis.
 * \sa <https://www.johnloomis.org/cpe102/asgn/asgn1/riff.html> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Riff {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<Riff>,
    pub(crate) _self_shared: SharedType<Self>,
    chunk: RefCell<OptRc<Riff_Chunk>>,
    _io: RefCell<BytesReader>,
    f_chunk_id: Cell<bool>,
    chunk_id: RefCell<Riff_Fourcc>,
    f_is_riff_chunk: Cell<bool>,
    is_riff_chunk: RefCell<bool>,
    f_parent_chunk_data: Cell<bool>,
    parent_chunk_data: RefCell<OptRc<Riff_ParentChunkData>>,
    f_subchunks: Cell<bool>,
    subchunks: RefCell<Vec<OptRc<Riff_ChunkType>>>,
}
impl KStruct for Riff {
    type Root = Riff;
    type Parent = Riff;

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
        let t = Self::read_into::<_, Riff_Chunk>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.chunk.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_id(
        &self
    ) -> KResult<Ref<'_, Riff_Fourcc>> {
        let _io = self._io.borrow();
        if self.f_chunk_id.get() {
            return Ok(self.chunk_id.borrow());
        }
        self.f_chunk_id.set(true);
        *self.chunk_id.borrow_mut() = i64::from(*self.chunk().id()).try_into()?;
        Ok(self.chunk_id.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_riff_chunk(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_riff_chunk.get() {
            return Ok(self.is_riff_chunk.borrow());
        }
        self.f_is_riff_chunk.set(true);
        *self.is_riff_chunk.borrow_mut() = (*self.chunk_id()? == Riff_Fourcc::Riff).try_into()?;
        Ok(self.is_riff_chunk.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn parent_chunk_data(
        &self
    ) -> KResult<Ref<'_, OptRc<Riff_ParentChunkData>>> {
        let _io = self._io.borrow();
        if self.f_parent_chunk_data.get() {
            return Ok(self.parent_chunk_data.borrow());
        }
        if *self.is_riff_chunk()? {
            let io = KStream::clone(&*self.chunk().data_slot()._io());
            let _pos = io.pos();
            io.seek(0_usize)?;
            let t = Self::read_into::<_, Riff_ParentChunkData>(&io, Some(self._root.clone()), None)?.into();
            *self.parent_chunk_data.borrow_mut() = t;
            io.seek(_pos)?;
        }
        Ok(self.parent_chunk_data.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn subchunks(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<Riff_ChunkType>>>> {
        let _io = self._io.borrow();
        if self.f_subchunks.get() {
            return Ok(self.subchunks.borrow());
        }
        self.f_subchunks.set(true);
        if *self.is_riff_chunk()? {
            let io = KStream::clone(&*self.parent_chunk_data()?.subchunks_slot()._io());
            let _pos = io.pos();
            io.seek(0_usize)?;
            *self.subchunks.borrow_mut() = Vec::new();
            {
                let mut _i = 0_usize;
                while !_io.is_eof() {
                    let t = Self::read_into::<_, Riff_ChunkType>(&*_io, Some(self._root.clone()), None)?.into();
                    self.subchunks.borrow_mut().push(t);
                    _i = _i.saturating_add(1);
                }
            }
            io.seek(_pos)?;
        }
        Ok(self.subchunks.borrow())
    }
}
impl Riff {
    pub fn chunk(&self) -> Ref<'_, OptRc<Riff_Chunk>> {
        self.chunk.borrow()
    }
}
impl Riff {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Riff_Fourcc {
    Riff,
    Info,
    List,
    Unknown(i64),
}

impl TryFrom<i64> for Riff_Fourcc {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Riff_Fourcc> {
        match flag {
            1179011410 => Ok(Riff_Fourcc::Riff),
            1330007625 => Ok(Riff_Fourcc::Info),
            1414744396 => Ok(Riff_Fourcc::List),
            _ => Ok(Riff_Fourcc::Unknown(flag)),
        }
    }
}

impl From<&Riff_Fourcc> for i64 {
    fn from(v: &Riff_Fourcc) -> Self {
        match *v {
            Riff_Fourcc::Riff => 1179011410,
            Riff_Fourcc::Info => 1330007625,
            Riff_Fourcc::List => 1414744396,
            Riff_Fourcc::Unknown(v) => v
        }
    }
}

impl Default for Riff_Fourcc {
    fn default() -> Self { Riff_Fourcc::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Riff_Chunk {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    id: RefCell<u32>,
    len: RefCell<u32>,
    data_slot: RefCell<OptRc<Riff_Chunk_Slot>>,
    pad_byte: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data_slot_raw: RefCell<Vec<u8>>,
    pad_byte_raw: RefCell<Vec<u8>>,
}
impl KStruct for Riff_Chunk {
    type Root = Riff;
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
        *self_rc.id.borrow_mut() = _io.read_u4le()?;
        *self_rc.len.borrow_mut() = _io.read_u4le()?;
        let _raw_data_slot = _io.read_bytes(usize::try_from(*self_rc.len())?)?;
        *self_rc.data_slot_raw.borrow_mut() = _raw_data_slot.clone();
        let _io_data_slot = BytesReader::from(_raw_data_slot);
        let t = Self::read_into::<BytesReader, Riff_Chunk_Slot>(&_io_data_slot, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.data_slot.borrow_mut() = t;
        *self_rc.pad_byte.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc.len()).checked_rem(2_u32).ok_or(KError::CastError)?)?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff_Chunk {
}
impl Riff_Chunk {
    pub fn id(&self) -> Ref<'_, u32> {
        self.id.borrow()
    }
}
impl Riff_Chunk {
    pub fn len(&self) -> Ref<'_, u32> {
        self.len.borrow()
    }
}
impl Riff_Chunk {
    pub fn data_slot(&self) -> Ref<'_, OptRc<Riff_Chunk_Slot>> {
        self.data_slot.borrow()
    }
}
impl Riff_Chunk {
    pub fn pad_byte(&self) -> Ref<'_, Vec<u8>> {
        self.pad_byte.borrow()
    }
}
impl Riff_Chunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Riff_Chunk {
    pub fn data_slot_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_slot_raw.borrow()
    }
}
impl Riff_Chunk {
    pub fn pad_byte_raw(&self) -> Ref<'_, Vec<u8>> {
        self.pad_byte_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Riff_Chunk_Slot {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<Riff_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Riff_Chunk_Slot {
    type Root = Riff;
    type Parent = Riff_Chunk;

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
impl Riff_Chunk_Slot {
}
impl Riff_Chunk_Slot {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Riff_ChunkType {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    save_chunk_ofs: RefCell<Vec<u8>>,
    chunk: RefCell<OptRc<Riff_Chunk>>,
    _io: RefCell<BytesReader>,
    save_chunk_ofs_raw: RefCell<Vec<u8>>,
    f_chunk_data: Cell<bool>,
    chunk_data: RefCell<Option<Riff_ChunkType_ChunkData>>,
    f_chunk_id: Cell<bool>,
    chunk_id: RefCell<Riff_Fourcc>,
    f_chunk_id_readable: Cell<bool>,
    chunk_id_readable: RefCell<String>,
    f_chunk_ofs: Cell<bool>,
    chunk_ofs: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum Riff_ChunkType_ChunkData {
    Riff_ListChunkData(OptRc<Riff_ListChunkData>),
}
impl TryFrom<&Riff_ChunkType_ChunkData> for OptRc<Riff_ListChunkData> {
    type Error = KError;
    fn try_from(v: &Riff_ChunkType_ChunkData) -> Result<Self, Self::Error> {
        if let Riff_ChunkType_ChunkData::Riff_ListChunkData(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Riff_ListChunkData>> for Riff_ChunkType_ChunkData {
    fn from(v: OptRc<Riff_ListChunkData>) -> Self {
        Self::Riff_ListChunkData(v)
    }
}
impl KStruct for Riff_ChunkType {
    type Root = Riff;
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
        if *self_rc.chunk_ofs()? < 0 {
            *self_rc.save_chunk_ofs.borrow_mut() = _io.read_bytes(0_usize)?;
        }
        let t = Self::read_into::<_, Riff_Chunk>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.chunk.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff_ChunkType {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_data(
        &self
    ) -> KResult<Ref<'_, Option<Riff_ChunkType_ChunkData>>> {
        let _io = self._io.borrow();
        if self.f_chunk_data.get() {
            return Ok(self.chunk_data.borrow());
        }
        self.f_chunk_data.set(true);
        let io = KStream::clone(&*self.chunk().data_slot()._io());
        let _pos = io.pos();
        io.seek(0_usize)?;
        match *self.chunk_id()? {
            Riff_Fourcc::List => {
                let t = Self::read_into::<_, Riff_ListChunkData>(&io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                *self.chunk_data.borrow_mut() = Some(t);
            }
            _ => {}
        }
        io.seek(_pos)?;
        Ok(self.chunk_data.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_id(
        &self
    ) -> KResult<Ref<'_, Riff_Fourcc>> {
        let _io = self._io.borrow();
        if self.f_chunk_id.get() {
            return Ok(self.chunk_id.borrow());
        }
        self.f_chunk_id.set(true);
        *self.chunk_id.borrow_mut() = i64::from(*self.chunk().id()).try_into()?;
        Ok(self.chunk_id.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_id_readable(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_chunk_id_readable.get() {
            return Ok(self.chunk_id_readable.borrow());
        }
        self.f_chunk_id_readable.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.chunk_ofs()?)?)?;
        *self.chunk_id_readable.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        _io.seek(_pos)?;
        Ok(self.chunk_id_readable.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_ofs(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_chunk_ofs.get() {
            return Ok(self.chunk_ofs.borrow());
        }
        self.f_chunk_ofs.set(true);
        *self.chunk_ofs.borrow_mut() = (_io.pos()).try_into()?;
        Ok(self.chunk_ofs.borrow())
    }
}
impl Riff_ChunkType {
    pub fn save_chunk_ofs(&self) -> Ref<'_, Vec<u8>> {
        self.save_chunk_ofs.borrow()
    }
}
impl Riff_ChunkType {
    pub fn chunk(&self) -> Ref<'_, OptRc<Riff_Chunk>> {
        self.chunk.borrow()
    }
}
impl Riff_ChunkType {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Riff_ChunkType {
    pub fn save_chunk_ofs_raw(&self) -> Ref<'_, Vec<u8>> {
        self.save_chunk_ofs_raw.borrow()
    }
}

/**
 * All registered subchunks in the INFO chunk are NULL-terminated strings,
 * but the unregistered might not be. By convention, the registered
 * chunk IDs are in uppercase and the unregistered IDs are in lowercase.
 *
 * If the chunk ID of an INFO subchunk contains a lowercase
 * letter, this chunk is considered as unregistered and thus we can make
 * no assumptions about the type of data.
 */

#[derive(Default, Debug, Clone)]
pub struct Riff_InfoSubchunk {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<Riff_ListChunkData>,
    pub(crate) _self_shared: SharedType<Self>,
    save_chunk_ofs: RefCell<Vec<u8>>,
    chunk: RefCell<OptRc<Riff_Chunk>>,
    _io: RefCell<BytesReader>,
    save_chunk_ofs_raw: RefCell<Vec<u8>>,
    f_chunk_data: Cell<bool>,
    chunk_data: RefCell<Option<Riff_InfoSubchunk_ChunkData>>,
    f_chunk_id_readable: Cell<bool>,
    chunk_id_readable: RefCell<String>,
    f_chunk_ofs: Cell<bool>,
    chunk_ofs: RefCell<i32>,
    f_id_chars: Cell<bool>,
    id_chars: RefCell<Vec<u8>>,
    f_is_unregistered_tag: Cell<bool>,
    is_unregistered_tag: RefCell<bool>,
}
#[derive(Debug, Clone)]
pub enum Riff_InfoSubchunk_ChunkData {
    String(String),
}
impl TryFrom<&Riff_InfoSubchunk_ChunkData> for String {
    type Error = KError;
    fn try_from(v: &Riff_InfoSubchunk_ChunkData) -> Result<Self, Self::Error> {
        if let Riff_InfoSubchunk_ChunkData::String(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<String> for Riff_InfoSubchunk_ChunkData {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl KStruct for Riff_InfoSubchunk {
    type Root = Riff;
    type Parent = Riff_ListChunkData;

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
        if *self_rc.chunk_ofs()? < 0 {
            *self_rc.save_chunk_ofs.borrow_mut() = _io.read_bytes(0_usize)?;
        }
        let t = Self::read_into::<_, Riff_Chunk>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.chunk.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff_InfoSubchunk {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_data(
        &self
    ) -> KResult<Ref<'_, Option<Riff_InfoSubchunk_ChunkData>>> {
        let _io = self._io.borrow();
        if self.f_chunk_data.get() {
            return Ok(self.chunk_data.borrow());
        }
        self.f_chunk_data.set(true);
        let io = KStream::clone(&*self.chunk().data_slot()._io());
        let _pos = io.pos();
        io.seek(0_usize)?;
        match *self.is_unregistered_tag()? {
            false => {
            }
            _ => {}
        }
        io.seek(_pos)?;
        Ok(self.chunk_data.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_id_readable(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_chunk_id_readable.get() {
            return Ok(self.chunk_id_readable.borrow());
        }
        self.f_chunk_id_readable.set(true);
        *self.chunk_id_readable.borrow_mut() = bytes_to_str(&*self.id_chars()?, "ASCII")?.to_string();
        Ok(self.chunk_id_readable.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn chunk_ofs(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_chunk_ofs.get() {
            return Ok(self.chunk_ofs.borrow());
        }
        self.f_chunk_ofs.set(true);
        *self.chunk_ofs.borrow_mut() = (_io.pos()).try_into()?;
        Ok(self.chunk_ofs.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn id_chars(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_id_chars.get() {
            return Ok(self.id_chars.borrow());
        }
        self.f_id_chars.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.chunk_ofs()?)?)?;
        *self.id_chars.borrow_mut() = _io.read_bytes(4_usize)?;
        _io.seek(_pos)?;
        Ok(self.id_chars.borrow())
    }

    /**
     * Check if chunk_id contains lowercase characters ([a-z], ASCII 97 = a, ASCII 122 = z).
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_unregistered_tag(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_unregistered_tag.get() {
            return Ok(self.is_unregistered_tag.borrow());
        }
        self.f_is_unregistered_tag.set(true);
        *self.is_unregistered_tag.borrow_mut() = ( (( ((((to_i128(*(self.id_chars()?.get(0_usize).ok_or(KError::CastError)?))) >= (to_i128(97)))) && (((to_i128(*(self.id_chars()?.get(0_usize).ok_or(KError::CastError)?))) <= (to_i128(122))))) ) || ( ((((to_i128(*(self.id_chars()?.get(1_usize).ok_or(KError::CastError)?))) >= (to_i128(97)))) && (((to_i128(*(self.id_chars()?.get(1_usize).ok_or(KError::CastError)?))) <= (to_i128(122))))) ) || ( ((((to_i128(*(self.id_chars()?.get(2_usize).ok_or(KError::CastError)?))) >= (to_i128(97)))) && (((to_i128(*(self.id_chars()?.get(2_usize).ok_or(KError::CastError)?))) <= (to_i128(122))))) ) || ( ((((to_i128(*(self.id_chars()?.get(3_usize).ok_or(KError::CastError)?))) >= (to_i128(97)))) && (((to_i128(*(self.id_chars()?.get(3_usize).ok_or(KError::CastError)?))) <= (to_i128(122))))) )) ).try_into()?;
        Ok(self.is_unregistered_tag.borrow())
    }
}
impl Riff_InfoSubchunk {
    pub fn save_chunk_ofs(&self) -> Ref<'_, Vec<u8>> {
        self.save_chunk_ofs.borrow()
    }
}
impl Riff_InfoSubchunk {
    pub fn chunk(&self) -> Ref<'_, OptRc<Riff_Chunk>> {
        self.chunk.borrow()
    }
}
impl Riff_InfoSubchunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Riff_InfoSubchunk {
    pub fn save_chunk_ofs_raw(&self) -> Ref<'_, Vec<u8>> {
        self.save_chunk_ofs_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Riff_ListChunkData {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<Riff_ChunkType>,
    pub(crate) _self_shared: SharedType<Self>,
    save_parent_chunk_data_ofs: RefCell<Vec<u8>>,
    parent_chunk_data: RefCell<OptRc<Riff_ParentChunkData>>,
    _io: RefCell<BytesReader>,
    save_parent_chunk_data_ofs_raw: RefCell<Vec<u8>>,
    f_form_type: Cell<bool>,
    form_type: RefCell<Riff_Fourcc>,
    f_form_type_readable: Cell<bool>,
    form_type_readable: RefCell<String>,
    f_parent_chunk_data_ofs: Cell<bool>,
    parent_chunk_data_ofs: RefCell<i32>,
    f_subchunks: Cell<bool>,
    subchunks: RefCell<Vec<Riff_ListChunkData_Subchunks>>,
}
#[derive(Debug, Clone)]
pub enum Riff_ListChunkData_Subchunks {
    Riff_InfoSubchunk(OptRc<Riff_InfoSubchunk>),
    Riff_ChunkType(OptRc<Riff_ChunkType>),
}
impl TryFrom<&Riff_ListChunkData_Subchunks> for OptRc<Riff_InfoSubchunk> {
    type Error = KError;
    fn try_from(v: &Riff_ListChunkData_Subchunks) -> Result<Self, Self::Error> {
        if let Riff_ListChunkData_Subchunks::Riff_InfoSubchunk(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Riff_InfoSubchunk>> for Riff_ListChunkData_Subchunks {
    fn from(v: OptRc<Riff_InfoSubchunk>) -> Self {
        Self::Riff_InfoSubchunk(v)
    }
}
impl TryFrom<&Riff_ListChunkData_Subchunks> for OptRc<Riff_ChunkType> {
    type Error = KError;
    fn try_from(v: &Riff_ListChunkData_Subchunks) -> Result<Self, Self::Error> {
        if let Riff_ListChunkData_Subchunks::Riff_ChunkType(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Riff_ChunkType>> for Riff_ListChunkData_Subchunks {
    fn from(v: OptRc<Riff_ChunkType>) -> Self {
        Self::Riff_ChunkType(v)
    }
}
impl KStruct for Riff_ListChunkData {
    type Root = Riff;
    type Parent = Riff_ChunkType;

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
        if *self_rc.parent_chunk_data_ofs()? < 0 {
            *self_rc.save_parent_chunk_data_ofs.borrow_mut() = _io.read_bytes(0_usize)?;
        }
        let t = Self::read_into::<_, Riff_ParentChunkData>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.parent_chunk_data.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff_ListChunkData {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn form_type(
        &self
    ) -> KResult<Ref<'_, Riff_Fourcc>> {
        let _io = self._io.borrow();
        if self.f_form_type.get() {
            return Ok(self.form_type.borrow());
        }
        self.f_form_type.set(true);
        *self.form_type.borrow_mut() = i64::from(*self.parent_chunk_data().form_type()).try_into()?;
        Ok(self.form_type.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn form_type_readable(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_form_type_readable.get() {
            return Ok(self.form_type_readable.borrow());
        }
        self.f_form_type_readable.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.parent_chunk_data_ofs()?)?)?;
        *self.form_type_readable.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        _io.seek(_pos)?;
        Ok(self.form_type_readable.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn parent_chunk_data_ofs(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_parent_chunk_data_ofs.get() {
            return Ok(self.parent_chunk_data_ofs.borrow());
        }
        self.f_parent_chunk_data_ofs.set(true);
        *self.parent_chunk_data_ofs.borrow_mut() = (_io.pos()).try_into()?;
        Ok(self.parent_chunk_data_ofs.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn subchunks(
        &self
    ) -> KResult<Ref<'_, Vec<Riff_ListChunkData_Subchunks>>> {
        let _io = self._io.borrow();
        if self.f_subchunks.get() {
            return Ok(self.subchunks.borrow());
        }
        self.f_subchunks.set(true);
        let io = KStream::clone(&*self.parent_chunk_data().subchunks_slot()._io());
        let _pos = io.pos();
        io.seek(0_usize)?;
        *self.subchunks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                match *self.form_type()? {
                    Riff_Fourcc::Info => {
                        let t = Self::read_into::<_, Riff_InfoSubchunk>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                        self.subchunks.borrow_mut().push(t);
                    }
                    _ => {
                        let t = Self::read_into::<_, Riff_ChunkType>(&*_io, Some(self._root.clone()), None)?.into();
                        self.subchunks.borrow_mut().push(t);
                    }
                }
                _i = _i.saturating_add(1);
            }
        }
        io.seek(_pos)?;
        Ok(self.subchunks.borrow())
    }
}
impl Riff_ListChunkData {
    pub fn save_parent_chunk_data_ofs(&self) -> Ref<'_, Vec<u8>> {
        self.save_parent_chunk_data_ofs.borrow()
    }
}
impl Riff_ListChunkData {
    pub fn parent_chunk_data(&self) -> Ref<'_, OptRc<Riff_ParentChunkData>> {
        self.parent_chunk_data.borrow()
    }
}
impl Riff_ListChunkData {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Riff_ListChunkData {
    pub fn save_parent_chunk_data_ofs_raw(&self) -> Ref<'_, Vec<u8>> {
        self.save_parent_chunk_data_ofs_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Riff_ParentChunkData {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    form_type: RefCell<u32>,
    subchunks_slot: RefCell<OptRc<Riff_ParentChunkData_Slot>>,
    _io: RefCell<BytesReader>,
    subchunks_slot_raw: RefCell<Vec<u8>>,
}
impl KStruct for Riff_ParentChunkData {
    type Root = Riff;
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
        *self_rc.form_type.borrow_mut() = _io.read_u4le()?;
        let _raw_subchunks_slot = _io.read_bytes_full()?;
        *self_rc.subchunks_slot_raw.borrow_mut() = _raw_subchunks_slot.clone();
        let _io_subchunks_slot = BytesReader::from(_raw_subchunks_slot);
        let t = Self::read_into::<BytesReader, Riff_ParentChunkData_Slot>(&_io_subchunks_slot, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.subchunks_slot.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Riff_ParentChunkData {
}
impl Riff_ParentChunkData {
    pub fn form_type(&self) -> Ref<'_, u32> {
        self.form_type.borrow()
    }
}
impl Riff_ParentChunkData {
    pub fn subchunks_slot(&self) -> Ref<'_, OptRc<Riff_ParentChunkData_Slot>> {
        self.subchunks_slot.borrow()
    }
}
impl Riff_ParentChunkData {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Riff_ParentChunkData {
    pub fn subchunks_slot_raw(&self) -> Ref<'_, Vec<u8>> {
        self.subchunks_slot_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Riff_ParentChunkData_Slot {
    pub(crate) _root: SharedType<Riff>,
    pub(crate) _parent: SharedType<Riff_ParentChunkData>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Riff_ParentChunkData_Slot {
    type Root = Riff;
    type Parent = Riff_ParentChunkData;

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
impl Riff_ParentChunkData_Slot {
}
impl Riff_ParentChunkData_Slot {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
