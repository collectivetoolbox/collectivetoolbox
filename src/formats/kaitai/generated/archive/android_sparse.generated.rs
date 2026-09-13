// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * The Android sparse format is a format to more efficiently store files
 * for for example firmware updates to save on bandwidth. Files in sparse
 * format first have to be converted back to their original format.
 *
 * A tool to create images for testing can be found in the Android source code tree:
 *
 * <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse> - `img2simg.c`
 *
 * Note: this is not the same as the Android sparse data image format.
 * \sa <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse/sparse_format.h> Source
 * \sa <https://web.archive.org/web/20220322054458/https://source.android.com/devices/bootloader/images#sparse-image-format> Source
 */

#[derive(Default, Debug, Clone)]
pub struct AndroidSparse {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse>,
    pub(crate) _self_shared: SharedType<Self>,
    header_prefix: RefCell<OptRc<AndroidSparse_FileHeaderPrefix>>,
    header: RefCell<OptRc<AndroidSparse_FileHeader>>,
    chunks: RefCell<Vec<OptRc<AndroidSparse_Chunk>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidSparse {
    type Root = AndroidSparse;
    type Parent = AndroidSparse;

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
        let t = Self::read_into::<_, AndroidSparse_FileHeaderPrefix>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header_prefix.borrow_mut() = t;
        let t = Self::read_into::<_, AndroidSparse_FileHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.chunks.borrow_mut() = Vec::new();
        let l_chunks = *self_rc.header().num_chunks();
        for _i in 0..l_chunks {
            let t = Self::read_into::<_, AndroidSparse_Chunk>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.chunks.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl AndroidSparse {
}

/**
 * internal; access `_root.header` instead
 */
impl AndroidSparse {
    pub fn header_prefix(&self) -> Ref<'_, OptRc<AndroidSparse_FileHeaderPrefix>> {
        self.header_prefix.borrow()
    }
}
impl AndroidSparse {
    pub fn header(&self) -> Ref<'_, OptRc<AndroidSparse_FileHeader>> {
        self.header.borrow()
    }
}
impl AndroidSparse {
    pub fn chunks(&self) -> Ref<'_, Vec<OptRc<AndroidSparse_Chunk>>> {
        self.chunks.borrow()
    }
}
impl AndroidSparse {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum AndroidSparse_ChunkTypes {
    Raw,
    Fill,
    DontCare,
    Crc32,
    Unknown(i64),
}

impl TryFrom<i64> for AndroidSparse_ChunkTypes {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<AndroidSparse_ChunkTypes> {
        match flag {
            51905 => Ok(AndroidSparse_ChunkTypes::Raw),
            51906 => Ok(AndroidSparse_ChunkTypes::Fill),
            51907 => Ok(AndroidSparse_ChunkTypes::DontCare),
            51908 => Ok(AndroidSparse_ChunkTypes::Crc32),
            _ => Ok(AndroidSparse_ChunkTypes::Unknown(flag)),
        }
    }
}

impl From<&AndroidSparse_ChunkTypes> for i64 {
    fn from(v: &AndroidSparse_ChunkTypes) -> Self {
        match *v {
            AndroidSparse_ChunkTypes::Raw => 51905,
            AndroidSparse_ChunkTypes::Fill => 51906,
            AndroidSparse_ChunkTypes::DontCare => 51907,
            AndroidSparse_ChunkTypes::Crc32 => 51908,
            AndroidSparse_ChunkTypes::Unknown(v) => v
        }
    }
}

impl Default for AndroidSparse_ChunkTypes {
    fn default() -> Self { AndroidSparse_ChunkTypes::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct AndroidSparse_Chunk {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<AndroidSparse_Chunk_ChunkHeader>>,
    body: RefCell<Option<AndroidSparse_Chunk_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum AndroidSparse_Chunk_Body {
    U4(u32),
    Bytes(Vec<u8>),
}
impl From<&AndroidSparse_Chunk_Body> for u32 {
    fn from(v: &AndroidSparse_Chunk_Body) -> Self {
        if let AndroidSparse_Chunk_Body::U4(x) = v {
            return x.clone();
        }
        panic!("expected AndroidSparse_Chunk_Body::U4, got {:?}", v)
    }
}
impl From<u32> for AndroidSparse_Chunk_Body {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl From<&AndroidSparse_Chunk_Body> for Vec<u8> {
    fn from(v: &AndroidSparse_Chunk_Body) -> Self {
        if let AndroidSparse_Chunk_Body::Bytes(x) = v {
            return x.clone();
        }
        panic!("expected AndroidSparse_Chunk_Body::Bytes, got {:?}", v)
    }
}
impl From<Vec<u8>> for AndroidSparse_Chunk_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for AndroidSparse_Chunk {
    type Root = AndroidSparse;
    type Parent = AndroidSparse;

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
        let t = Self::read_into::<_, AndroidSparse_Chunk_ChunkHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        match *self_rc.header().chunk_type() {
            AndroidSparse_ChunkTypes::Crc32 => {
                *self_rc.body.borrow_mut() = Some(_io.read_u4le()?.into());
            }
            _ => {
                *self_rc.body.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        Ok(())
    }
}
impl AndroidSparse_Chunk {
}
impl AndroidSparse_Chunk {
    pub fn header(&self) -> Ref<'_, OptRc<AndroidSparse_Chunk_ChunkHeader>> {
        self.header.borrow()
    }
}
impl AndroidSparse_Chunk {
    pub fn body(&self) -> Ref<'_, Option<AndroidSparse_Chunk_Body>> {
        self.body.borrow()
    }
}
impl AndroidSparse_Chunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSparse_Chunk {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSparse_Chunk_ChunkHeader {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    chunk_type: RefCell<AndroidSparse_ChunkTypes>,
    reserved1: RefCell<u16>,
    num_body_blocks: RefCell<u32>,
    len_chunk: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_len_body: Cell<bool>,
    len_body: RefCell<u32>,
    f_len_body_expected: Cell<bool>,
    len_body_expected: RefCell<i32>,
}
impl KStruct for AndroidSparse_Chunk_ChunkHeader {
    type Root = AndroidSparse;
    type Parent = AndroidSparse_Chunk;

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
        *self_rc.chunk_type.borrow_mut() = i64::try_from(_io.read_u2le()?)?.try_into()?;
        *self_rc.reserved1.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_body_blocks.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_chunk.borrow_mut() = _io.read_u4le()?;
        let expected: u32 = (if *self_rc.len_body_expected()? != -(1) { ((((*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header().len_chunk_header()) as i32) + ((*self_rc.len_body_expected()?) as i32))) as u32 } else { (*self_rc.len_chunk()) as u32 }).try_into()?;
        if !(*self_rc.len_chunk() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/chunk/types/chunk_header/seq/3".to_string() }));
        }
        Ok(())
    }
}
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn len_body(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_len_body.get() {
            return Ok(self.len_body.borrow());
        }
        self.f_len_body.set(true);
        *self.len_body.borrow_mut() = ((((*self.len_chunk()) as u32) - ((*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header().len_chunk_header()) as u32))).try_into()?;
        Ok(self.len_body.borrow())
    }

    /**
     * \sa <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse/sparse_read.cpp#184> Source
     * \sa <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse/sparse_read.cpp#215> Source
     * \sa <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse/sparse_read.cpp#249> Source
     * \sa <https://android.googlesource.com/platform/system/core/+/e8d02c50d7/libsparse/sparse_read.cpp#270> Source
     */
    pub fn len_body_expected(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_body_expected.get() {
            return Ok(self.len_body_expected.borrow());
        }
        self.f_len_body_expected.set(true);
        *self.len_body_expected.borrow_mut() = (if *self.chunk_type() == AndroidSparse_ChunkTypes::Raw { ((((*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header().block_size()) as u32) * ((*self.num_body_blocks()) as u32))) as u32 } else { (if *self.chunk_type() == AndroidSparse_ChunkTypes::Fill { (4) as i32 } else { (if *self.chunk_type() == AndroidSparse_ChunkTypes::DontCare { (0) as i32 } else { (if *self.chunk_type() == AndroidSparse_ChunkTypes::Crc32 { (4) as i32 } else { (-(1)) as i32 }) as i32 }) as i32 }) as u32 }).try_into()?;
        Ok(self.len_body_expected.borrow())
    }
}
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn chunk_type(&self) -> Ref<'_, AndroidSparse_ChunkTypes> {
        self.chunk_type.borrow()
    }
}
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn reserved1(&self) -> Ref<'_, u16> {
        self.reserved1.borrow()
    }
}

/**
 * size of the chunk body in blocks in output image
 */
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn num_body_blocks(&self) -> Ref<'_, u32> {
        self.num_body_blocks.borrow()
    }
}

/**
 * in bytes of chunk input file including chunk header and data
 */
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn len_chunk(&self) -> Ref<'_, u32> {
        self.len_chunk.borrow()
    }
}
impl AndroidSparse_Chunk_ChunkHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSparse_FileHeader {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse>,
    pub(crate) _self_shared: SharedType<Self>,
    len_chunk_header: RefCell<u16>,
    block_size: RefCell<u32>,
    num_blocks: RefCell<u32>,
    num_chunks: RefCell<u32>,
    checksum: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_len_header: Cell<bool>,
    len_header: RefCell<u16>,
    f_version: Cell<bool>,
    version: RefCell<OptRc<AndroidSparse_Version>>,
}
impl KStruct for AndroidSparse_FileHeader {
    type Root = AndroidSparse;
    type Parent = AndroidSparse;

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
        *self_rc.len_chunk_header.borrow_mut() = _io.read_u2le()?;
        *self_rc.block_size.borrow_mut() = _io.read_u4le()?;
        let _tmpa = *self_rc.block_size();
        if !((((((_tmpa) as u32) % ((4) as u32)) as u32) == (0 as u32))) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::Expr, src_path: "/types/file_header/seq/1".to_string() }));
        }
        *self_rc.num_blocks.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_chunks.borrow_mut() = _io.read_u4le()?;
        *self_rc.checksum.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl AndroidSparse_FileHeader {

    /**
     * size of file header, should be 28
     */
    pub fn len_header(
        &self
    ) -> KResult<Ref<'_, u16>> {
        let _io = self._io.borrow();
        if self.f_len_header.get() {
            return Ok(self.len_header.borrow());
        }
        self.f_len_header.set(true);
        *self.len_header.borrow_mut() = (*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header_prefix().len_header()).try_into()?;
        Ok(self.len_header.borrow())
    }
    pub fn version(
        &self
    ) -> KResult<Ref<'_, OptRc<AndroidSparse_Version>>> {
        let _io = self._io.borrow();
        if self.f_version.get() {
            return Ok(self.version.borrow());
        }
        *self.version.borrow_mut() = self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.header_prefix().version().clone();
        Ok(self.version.borrow())
    }
}

/**
 * size of chunk header, should be 12
 */
impl AndroidSparse_FileHeader {
    pub fn len_chunk_header(&self) -> Ref<'_, u16> {
        self.len_chunk_header.borrow()
    }
}

/**
 * block size in bytes, must be a multiple of 4
 */
impl AndroidSparse_FileHeader {
    pub fn block_size(&self) -> Ref<'_, u32> {
        self.block_size.borrow()
    }
}

/**
 * blocks in the original data
 */
impl AndroidSparse_FileHeader {
    pub fn num_blocks(&self) -> Ref<'_, u32> {
        self.num_blocks.borrow()
    }
}
impl AndroidSparse_FileHeader {
    pub fn num_chunks(&self) -> Ref<'_, u32> {
        self.num_chunks.borrow()
    }
}

/**
 * CRC32 checksum of the original data
 *
 * In practice always 0; if checksum writing is requested, a CRC32 chunk is written
 * at the end of the file instead. The canonical `libsparse` implementation does this
 * and other implementations tend to follow it, see
 * <https://gitlab.com/teskje/android-sparse-rs/-/blob/57c2577/src/write.rs#L112-114>
 */
impl AndroidSparse_FileHeader {
    pub fn checksum(&self) -> Ref<'_, u32> {
        self.checksum.borrow()
    }
}
impl AndroidSparse_FileHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSparse_FileHeaderPrefix {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    version: RefCell<OptRc<AndroidSparse_Version>>,
    len_header: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidSparse_FileHeaderPrefix {
    type Root = AndroidSparse;
    type Parent = AndroidSparse;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(usize::try_from(4)?)?;
        if !(*self_rc.magic() == vec![0x3au8, 0xffu8, 0x26u8, 0xedu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header_prefix/seq/0".to_string() }));
        }
        let t = Self::read_into::<_, AndroidSparse_Version>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.version.borrow_mut() = t;
        *self_rc.len_header.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl AndroidSparse_FileHeaderPrefix {
}
impl AndroidSparse_FileHeaderPrefix {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * internal; access `_root.header.version` instead
 */
impl AndroidSparse_FileHeaderPrefix {
    pub fn version(&self) -> Ref<'_, OptRc<AndroidSparse_Version>> {
        self.version.borrow()
    }
}

/**
 * internal; access `_root.header.len_header` instead
 */
impl AndroidSparse_FileHeaderPrefix {
    pub fn len_header(&self) -> Ref<'_, u16> {
        self.len_header.borrow()
    }
}
impl AndroidSparse_FileHeaderPrefix {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSparse_Version {
    pub(crate) _root: SharedType<AndroidSparse>,
    pub(crate) _parent: SharedType<AndroidSparse_FileHeaderPrefix>,
    pub(crate) _self_shared: SharedType<Self>,
    major: RefCell<u16>,
    minor: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidSparse_Version {
    type Root = AndroidSparse;
    type Parent = AndroidSparse_FileHeaderPrefix;

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
        *self_rc.major.borrow_mut() = _io.read_u2le()?;
        let expected: u16 = (1).try_into()?;
        if !(*self_rc.major() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/version/seq/0".to_string() }));
        }
        *self_rc.minor.borrow_mut() = _io.read_u2le()?;
        Ok(())
    }
}
impl AndroidSparse_Version {
}
impl AndroidSparse_Version {
    pub fn major(&self) -> Ref<'_, u16> {
        self.major.borrow()
    }
}
impl AndroidSparse_Version {
    pub fn minor(&self) -> Ref<'_, u16> {
        self.minor.borrow()
    }
}
impl AndroidSparse_Version {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
