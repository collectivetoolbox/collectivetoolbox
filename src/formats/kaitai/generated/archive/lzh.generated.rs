// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::dos_datetime::DosDatetime;

/**
 * LHA (LHarc, LZH) is a file format used by a popular freeware
 * eponymous archiver, created in 1988 by Haruyasu Yoshizaki. Over the
 * years, many ports and implementations were developed, sporting many
 * extensions to original 1988 LZH.
 *
 * File format is pretty simple and essentially consists of a stream of
 * records.
 */

#[derive(Default, Debug, Clone)]
pub struct Lzh {
    pub(crate) _root: SharedType<Lzh>,
    pub(crate) _parent: SharedType<Lzh>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Lzh_Record>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Lzh {
    type Root = Lzh;
    type Parent = Lzh;

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
                let t = Self::read_into::<_, Lzh_Record>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Lzh {
}
impl Lzh {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Lzh_Record>>> {
        self.entries.borrow()
    }
}
impl Lzh {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lzh_FileRecord {
    pub(crate) _root: SharedType<Lzh>,
    pub(crate) _parent: SharedType<Lzh_Record>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Lzh_Header>>,
    file_uncompr_crc16: RefCell<u16>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    header_raw: RefCell<Vec<u8>>,
    body_raw: RefCell<Vec<u8>>,
}
impl KStruct for Lzh_FileRecord {
    type Root = Lzh;
    type Parent = Lzh_Record;

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
        let _raw_header = _io.read_bytes(usize::try_from((i32::from(*self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.header_len())).saturating_sub(1_i32))?)?;
        *self_rc.header_raw.borrow_mut() = _raw_header.clone();
        let _io_header = BytesReader::from(_raw_header);
        let t = Self::read_into::<BytesReader, Lzh_Header>(&_io_header, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        if ((to_i128(*self_rc.header().header1().lha_level())) == (to_i128(0))) {
            *self_rc.file_uncompr_crc16.borrow_mut() = _io.read_u2le()?;
        }
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.header().header1().file_size_compr())?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Lzh_FileRecord {
}
impl Lzh_FileRecord {
    pub fn header(&self) -> Ref<'_, OptRc<Lzh_Header>> {
        self.header.borrow()
    }
}
impl Lzh_FileRecord {
    pub fn file_uncompr_crc16(&self) -> Ref<'_, u16> {
        self.file_uncompr_crc16.borrow()
    }
}
impl Lzh_FileRecord {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl Lzh_FileRecord {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Lzh_FileRecord {
    pub fn header_raw(&self) -> Ref<'_, Vec<u8>> {
        self.header_raw.borrow()
    }
}
impl Lzh_FileRecord {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lzh_Header {
    pub(crate) _root: SharedType<Lzh>,
    pub(crate) _parent: SharedType<Lzh_FileRecord>,
    pub(crate) _self_shared: SharedType<Self>,
    header1: RefCell<OptRc<Lzh_Header1>>,
    filename_len: RefCell<u8>,
    filename: RefCell<String>,
    file_uncompr_crc16: RefCell<u16>,
    os: RefCell<u8>,
    ext_header_size: RefCell<u16>,
    _io: RefCell<BytesReader>,
    filename_raw: RefCell<Vec<u8>>,
}
impl KStruct for Lzh_Header {
    type Root = Lzh;
    type Parent = Lzh_FileRecord;

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
        let t = Self::read_into::<_, Lzh_Header1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header1.borrow_mut() = t;
        if ((to_i128(*self_rc.header1().lha_level())) == (to_i128(0))) {
            *self_rc.filename_len.borrow_mut() = _io.read_u1()?;
        }
        if ((to_i128(*self_rc.header1().lha_level())) == (to_i128(0))) {
            *self_rc.filename.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.filename_len()))?, "ASCII")?;
        }
        if ((to_i128(*self_rc.header1().lha_level())) == (to_i128(2))) {
            *self_rc.file_uncompr_crc16.borrow_mut() = _io.read_u2le()?;
        }
        if ((to_i128(*self_rc.header1().lha_level())) == (to_i128(2))) {
            *self_rc.os.borrow_mut() = _io.read_u1()?;
        }
        if ((to_i128(*self_rc.header1().lha_level())) == (to_i128(2))) {
            *self_rc.ext_header_size.borrow_mut() = _io.read_u2le()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Lzh_Header {
}

/**
 * Level-neutral header, same for all LHA levels. Subsequent fields order and meaning varies, based on LHA level specified in this header.
 */
impl Lzh_Header {
    pub fn header1(&self) -> Ref<'_, OptRc<Lzh_Header1>> {
        self.header1.borrow()
    }
}
impl Lzh_Header {
    pub fn filename_len(&self) -> Ref<'_, u8> {
        self.filename_len.borrow()
    }
}
impl Lzh_Header {
    pub fn filename(&self) -> Ref<'_, String> {
        self.filename.borrow()
    }
}
impl Lzh_Header {
    pub fn file_uncompr_crc16(&self) -> Ref<'_, u16> {
        self.file_uncompr_crc16.borrow()
    }
}
impl Lzh_Header {
    pub fn os(&self) -> Ref<'_, u8> {
        self.os.borrow()
    }
}
impl Lzh_Header {
    pub fn ext_header_size(&self) -> Ref<'_, u16> {
        self.ext_header_size.borrow()
    }
}
impl Lzh_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Lzh_Header {
    pub fn filename_raw(&self) -> Ref<'_, Vec<u8>> {
        self.filename_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lzh_Header1 {
    pub(crate) _root: SharedType<Lzh>,
    pub(crate) _parent: SharedType<Lzh_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    header_checksum: RefCell<u8>,
    method_id: RefCell<String>,
    file_size_compr: RefCell<u32>,
    file_size_uncompr: RefCell<u32>,
    file_timestamp: RefCell<OptRc<DosDatetime>>,
    attr: RefCell<u8>,
    lha_level: RefCell<u8>,
    _io: RefCell<BytesReader>,
    method_id_raw: RefCell<Vec<u8>>,
    file_timestamp_raw: RefCell<Vec<u8>>,
}
impl KStruct for Lzh_Header1 {
    type Root = Lzh;
    type Parent = Lzh_Header;

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
        *self_rc.header_checksum.borrow_mut() = _io.read_u1()?;
        *self_rc.method_id.borrow_mut() = bytes_to_str(&_io.read_bytes(5_usize)?, "ASCII")?;
        *self_rc.file_size_compr.borrow_mut() = _io.read_u4le()?;
        *self_rc.file_size_uncompr.borrow_mut() = _io.read_u4le()?;
        let _raw_file_timestamp = _io.read_bytes(4_usize)?;
        *self_rc.file_timestamp_raw.borrow_mut() = _raw_file_timestamp.clone();
        let _io_file_timestamp = BytesReader::from(_raw_file_timestamp);
        let t = Self::read_into::<BytesReader, DosDatetime>(&_io_file_timestamp, None, None)?.into();
        *self_rc.file_timestamp.borrow_mut() = t;
        *self_rc.attr.borrow_mut() = _io.read_u1()?;
        *self_rc.lha_level.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Lzh_Header1 {
}
impl Lzh_Header1 {
    pub fn header_checksum(&self) -> Ref<'_, u8> {
        self.header_checksum.borrow()
    }
}
impl Lzh_Header1 {
    pub fn method_id(&self) -> Ref<'_, String> {
        self.method_id.borrow()
    }
}

/**
 * Compressed file size
 */
impl Lzh_Header1 {
    pub fn file_size_compr(&self) -> Ref<'_, u32> {
        self.file_size_compr.borrow()
    }
}

/**
 * Uncompressed file size
 */
impl Lzh_Header1 {
    pub fn file_size_uncompr(&self) -> Ref<'_, u32> {
        self.file_size_uncompr.borrow()
    }
}

/**
 * Original file date/time
 */
impl Lzh_Header1 {
    pub fn file_timestamp(&self) -> Ref<'_, OptRc<DosDatetime>> {
        self.file_timestamp.borrow()
    }
}

/**
 * File or directory attribute
 */
impl Lzh_Header1 {
    pub fn attr(&self) -> Ref<'_, u8> {
        self.attr.borrow()
    }
}
impl Lzh_Header1 {
    pub fn lha_level(&self) -> Ref<'_, u8> {
        self.lha_level.borrow()
    }
}
impl Lzh_Header1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Lzh_Header1 {
    pub fn method_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.method_id_raw.borrow()
    }
}
impl Lzh_Header1 {
    pub fn file_timestamp_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_timestamp_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lzh_Record {
    pub(crate) _root: SharedType<Lzh>,
    pub(crate) _parent: SharedType<Lzh>,
    pub(crate) _self_shared: SharedType<Self>,
    header_len: RefCell<u8>,
    file_record: RefCell<OptRc<Lzh_FileRecord>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Lzh_Record {
    type Root = Lzh;
    type Parent = Lzh;

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
        *self_rc.header_len.borrow_mut() = _io.read_u1()?;
        if ((to_i128(*self_rc.header_len())) > (to_i128(0))) {
            let t = Self::read_into::<_, Lzh_FileRecord>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.file_record.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Lzh_Record {
}
impl Lzh_Record {
    pub fn header_len(&self) -> Ref<'_, u8> {
        self.header_len.borrow()
    }
}
impl Lzh_Record {
    pub fn file_record(&self) -> Ref<'_, OptRc<Lzh_FileRecord>> {
        self.file_record.borrow()
    }
}
impl Lzh_Record {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
