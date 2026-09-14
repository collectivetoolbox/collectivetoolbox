// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::dos_datetime::DosDatetime;

/**
 * ZIP is a popular archive file format, introduced in 1989 by Phil Katz
 * and originally implemented in PKZIP utility by PKWARE.
 *
 * Thanks to solid support of it in most desktop environments and
 * operating systems, and algorithms / specs availability in public
 * domain, it quickly became tool of choice for implementing file
 * containers.
 *
 * For example, Java .jar files, OpenDocument, Office Open XML, EPUB files
 * are actually ZIP archives.
 * \sa <https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT> Source
 * \sa <https://users.cs.jmu.edu/buchhofp/forensics/formats/pkzip.html> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Zip {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip>,
    pub(crate) _self_shared: SharedType<Self>,
    sections: RefCell<Vec<OptRc<Zip_PkSection>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip {
    type Root = Zip;
    type Parent = Zip;

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
        *self_rc.sections.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Zip_PkSection>(&*_io, Some(self_rc._root.clone()), None)?.into();
                self_rc.sections.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip {
}
impl Zip {
    pub fn sections(&self) -> Ref<'_, Vec<OptRc<Zip_PkSection>>> {
        self.sections.borrow()
    }
}
impl Zip {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Zip_Compression {
    None,
    Shrunk,
    Reduced1,
    Reduced2,
    Reduced3,
    Reduced4,
    Imploded,
    Deflated,
    EnhancedDeflated,
    PkwareDclImploded,
    Bzip2,
    Lzma,
    IbmTerse,
    IbmLz77Z,
    Zstandard,
    Mp3,
    Xz,
    Jpeg,
    Wavpack,
    Ppmd,
    AexEncryptionMarker,
    Unknown(i64),
}

impl TryFrom<i64> for Zip_Compression {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Zip_Compression> {
        match flag {
            0 => Ok(Zip_Compression::None),
            1 => Ok(Zip_Compression::Shrunk),
            2 => Ok(Zip_Compression::Reduced1),
            3 => Ok(Zip_Compression::Reduced2),
            4 => Ok(Zip_Compression::Reduced3),
            5 => Ok(Zip_Compression::Reduced4),
            6 => Ok(Zip_Compression::Imploded),
            8 => Ok(Zip_Compression::Deflated),
            9 => Ok(Zip_Compression::EnhancedDeflated),
            10 => Ok(Zip_Compression::PkwareDclImploded),
            12 => Ok(Zip_Compression::Bzip2),
            14 => Ok(Zip_Compression::Lzma),
            18 => Ok(Zip_Compression::IbmTerse),
            19 => Ok(Zip_Compression::IbmLz77Z),
            93 => Ok(Zip_Compression::Zstandard),
            94 => Ok(Zip_Compression::Mp3),
            95 => Ok(Zip_Compression::Xz),
            96 => Ok(Zip_Compression::Jpeg),
            97 => Ok(Zip_Compression::Wavpack),
            98 => Ok(Zip_Compression::Ppmd),
            99 => Ok(Zip_Compression::AexEncryptionMarker),
            _ => Ok(Zip_Compression::Unknown(flag)),
        }
    }
}

impl From<&Zip_Compression> for i64 {
    fn from(v: &Zip_Compression) -> Self {
        match *v {
            Zip_Compression::None => 0,
            Zip_Compression::Shrunk => 1,
            Zip_Compression::Reduced1 => 2,
            Zip_Compression::Reduced2 => 3,
            Zip_Compression::Reduced3 => 4,
            Zip_Compression::Reduced4 => 5,
            Zip_Compression::Imploded => 6,
            Zip_Compression::Deflated => 8,
            Zip_Compression::EnhancedDeflated => 9,
            Zip_Compression::PkwareDclImploded => 10,
            Zip_Compression::Bzip2 => 12,
            Zip_Compression::Lzma => 14,
            Zip_Compression::IbmTerse => 18,
            Zip_Compression::IbmLz77Z => 19,
            Zip_Compression::Zstandard => 93,
            Zip_Compression::Mp3 => 94,
            Zip_Compression::Xz => 95,
            Zip_Compression::Jpeg => 96,
            Zip_Compression::Wavpack => 97,
            Zip_Compression::Ppmd => 98,
            Zip_Compression::AexEncryptionMarker => 99,
            Zip_Compression::Unknown(v) => v
        }
    }
}

impl Default for Zip_Compression {
    fn default() -> Self { Zip_Compression::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Zip_ExtraCodes {
    Zip64,
    AvInfo,
    Os2,
    Ntfs,
    Openvms,
    PkwareUnix,
    FileStreamAndForkDescriptors,
    PatchDescriptor,
    Pkcs7,
    X509CertIdAndSignatureForFile,
    X509CertIdForCentralDir,
    StrongEncryptionHeader,
    RecordManagementControls,
    Pkcs7EncRecipCertList,
    IbmS390Uncomp,
    IbmS390Comp,
    Poszip4690,
    ExtendedTimestamp,
    Beos,
    AsiUnix,
    InfozipUnix,
    InfozipUnixVarSize,
    AexEncryption,
    ApacheCommonsCompress,
    MicrosoftOpenPackagingGrowthHint,
    SmsQdos,
    Unknown(i64),
}

impl TryFrom<i64> for Zip_ExtraCodes {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Zip_ExtraCodes> {
        match flag {
            1 => Ok(Zip_ExtraCodes::Zip64),
            7 => Ok(Zip_ExtraCodes::AvInfo),
            9 => Ok(Zip_ExtraCodes::Os2),
            10 => Ok(Zip_ExtraCodes::Ntfs),
            12 => Ok(Zip_ExtraCodes::Openvms),
            13 => Ok(Zip_ExtraCodes::PkwareUnix),
            14 => Ok(Zip_ExtraCodes::FileStreamAndForkDescriptors),
            15 => Ok(Zip_ExtraCodes::PatchDescriptor),
            20 => Ok(Zip_ExtraCodes::Pkcs7),
            21 => Ok(Zip_ExtraCodes::X509CertIdAndSignatureForFile),
            22 => Ok(Zip_ExtraCodes::X509CertIdForCentralDir),
            23 => Ok(Zip_ExtraCodes::StrongEncryptionHeader),
            24 => Ok(Zip_ExtraCodes::RecordManagementControls),
            25 => Ok(Zip_ExtraCodes::Pkcs7EncRecipCertList),
            101 => Ok(Zip_ExtraCodes::IbmS390Uncomp),
            102 => Ok(Zip_ExtraCodes::IbmS390Comp),
            18064 => Ok(Zip_ExtraCodes::Poszip4690),
            21589 => Ok(Zip_ExtraCodes::ExtendedTimestamp),
            25922 => Ok(Zip_ExtraCodes::Beos),
            30062 => Ok(Zip_ExtraCodes::AsiUnix),
            30805 => Ok(Zip_ExtraCodes::InfozipUnix),
            30837 => Ok(Zip_ExtraCodes::InfozipUnixVarSize),
            39169 => Ok(Zip_ExtraCodes::AexEncryption),
            41246 => Ok(Zip_ExtraCodes::ApacheCommonsCompress),
            41504 => Ok(Zip_ExtraCodes::MicrosoftOpenPackagingGrowthHint),
            64842 => Ok(Zip_ExtraCodes::SmsQdos),
            _ => Ok(Zip_ExtraCodes::Unknown(flag)),
        }
    }
}

impl From<&Zip_ExtraCodes> for i64 {
    fn from(v: &Zip_ExtraCodes) -> Self {
        match *v {
            Zip_ExtraCodes::Zip64 => 1,
            Zip_ExtraCodes::AvInfo => 7,
            Zip_ExtraCodes::Os2 => 9,
            Zip_ExtraCodes::Ntfs => 10,
            Zip_ExtraCodes::Openvms => 12,
            Zip_ExtraCodes::PkwareUnix => 13,
            Zip_ExtraCodes::FileStreamAndForkDescriptors => 14,
            Zip_ExtraCodes::PatchDescriptor => 15,
            Zip_ExtraCodes::Pkcs7 => 20,
            Zip_ExtraCodes::X509CertIdAndSignatureForFile => 21,
            Zip_ExtraCodes::X509CertIdForCentralDir => 22,
            Zip_ExtraCodes::StrongEncryptionHeader => 23,
            Zip_ExtraCodes::RecordManagementControls => 24,
            Zip_ExtraCodes::Pkcs7EncRecipCertList => 25,
            Zip_ExtraCodes::IbmS390Uncomp => 101,
            Zip_ExtraCodes::IbmS390Comp => 102,
            Zip_ExtraCodes::Poszip4690 => 18064,
            Zip_ExtraCodes::ExtendedTimestamp => 21589,
            Zip_ExtraCodes::Beos => 25922,
            Zip_ExtraCodes::AsiUnix => 30062,
            Zip_ExtraCodes::InfozipUnix => 30805,
            Zip_ExtraCodes::InfozipUnixVarSize => 30837,
            Zip_ExtraCodes::AexEncryption => 39169,
            Zip_ExtraCodes::ApacheCommonsCompress => 41246,
            Zip_ExtraCodes::MicrosoftOpenPackagingGrowthHint => 41504,
            Zip_ExtraCodes::SmsQdos => 64842,
            Zip_ExtraCodes::Unknown(v) => v
        }
    }
}

impl Default for Zip_ExtraCodes {
    fn default() -> Self { Zip_ExtraCodes::Unknown(0) }
}


/**
 * \sa https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT - 4.3.12
 */

#[derive(Default, Debug, Clone)]
pub struct Zip_CentralDirEntry {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_PkSection>,
    pub(crate) _self_shared: SharedType<Self>,
    version_made_by: RefCell<u16>,
    version_needed_to_extract: RefCell<u16>,
    flags: RefCell<u16>,
    compression_method: RefCell<Zip_Compression>,
    file_mod_time: RefCell<OptRc<DosDatetime>>,
    crc32: RefCell<u32>,
    len_body_compressed: RefCell<u32>,
    len_body_uncompressed: RefCell<u32>,
    len_file_name: RefCell<u16>,
    len_extra: RefCell<u16>,
    len_comment: RefCell<u16>,
    disk_number_start: RefCell<u16>,
    int_file_attr: RefCell<u16>,
    ext_file_attr: RefCell<u32>,
    ofs_local_header: RefCell<i32>,
    file_name: RefCell<String>,
    extra: RefCell<OptRc<Zip_Extras>>,
    comment: RefCell<String>,
    _io: RefCell<BytesReader>,
    file_mod_time_raw: RefCell<Vec<u8>>,
    file_name_raw: RefCell<Vec<u8>>,
    extra_raw: RefCell<Vec<u8>>,
    comment_raw: RefCell<Vec<u8>>,
    f_local_header: Cell<bool>,
    local_header: RefCell<OptRc<Zip_PkSection>>,
}
impl KStruct for Zip_CentralDirEntry {
    type Root = Zip;
    type Parent = Zip_PkSection;

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
        *self_rc.version_made_by.borrow_mut() = _io.read_u2le()?;
        *self_rc.version_needed_to_extract.borrow_mut() = _io.read_u2le()?;
        *self_rc.flags.borrow_mut() = _io.read_u2le()?;
        *self_rc.compression_method.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        let _raw_file_mod_time = _io.read_bytes(4_usize)?;
        *self_rc.file_mod_time_raw.borrow_mut() = _raw_file_mod_time.clone();
        let _io_file_mod_time = BytesReader::from(_raw_file_mod_time);
        let t = Self::read_into::<BytesReader, DosDatetime>(&_io_file_mod_time, None, None)?.into();
        *self_rc.file_mod_time.borrow_mut() = t;
        *self_rc.crc32.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_compressed.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_uncompressed.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_file_name.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_extra.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_comment.borrow_mut() = _io.read_u2le()?;
        *self_rc.disk_number_start.borrow_mut() = _io.read_u2le()?;
        *self_rc.int_file_attr.borrow_mut() = _io.read_u2le()?;
        *self_rc.ext_file_attr.borrow_mut() = _io.read_u4le()?;
        *self_rc.ofs_local_header.borrow_mut() = _io.read_s4le()?;
        *self_rc.file_name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_file_name()))?, "UTF-8")?;
        let _raw_extra = _io.read_bytes(usize::from(*self_rc.len_extra()))?;
        *self_rc.extra_raw.borrow_mut() = _raw_extra.clone();
        let _io_extra = BytesReader::from(_raw_extra);
        let t = Self::read_into::<BytesReader, Zip_Extras>(&_io_extra, Some(self_rc._root.clone()), None)?.into();
        *self_rc.extra.borrow_mut() = t;
        *self_rc.comment.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_comment()))?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_CentralDirEntry {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn local_header(
        &self
    ) -> KResult<Ref<'_, OptRc<Zip_PkSection>>> {
        let _io = self._io.borrow();
        if self.f_local_header.get() {
            return Ok(self.local_header.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs_local_header())?)?;
        let t = Self::read_into::<_, Zip_PkSection>(&*_io, Some(self._root.clone()), None)?.into();
        *self.local_header.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.local_header.borrow())
    }
}
impl Zip_CentralDirEntry {
    pub fn version_made_by(&self) -> Ref<'_, u16> {
        self.version_made_by.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn version_needed_to_extract(&self) -> Ref<'_, u16> {
        self.version_needed_to_extract.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn flags(&self) -> Ref<'_, u16> {
        self.flags.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn compression_method(&self) -> Ref<'_, Zip_Compression> {
        self.compression_method.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn file_mod_time(&self) -> Ref<'_, OptRc<DosDatetime>> {
        self.file_mod_time.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn crc32(&self) -> Ref<'_, u32> {
        self.crc32.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn len_body_compressed(&self) -> Ref<'_, u32> {
        self.len_body_compressed.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn len_body_uncompressed(&self) -> Ref<'_, u32> {
        self.len_body_uncompressed.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn len_file_name(&self) -> Ref<'_, u16> {
        self.len_file_name.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn len_extra(&self) -> Ref<'_, u16> {
        self.len_extra.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn len_comment(&self) -> Ref<'_, u16> {
        self.len_comment.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn disk_number_start(&self) -> Ref<'_, u16> {
        self.disk_number_start.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn int_file_attr(&self) -> Ref<'_, u16> {
        self.int_file_attr.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn ext_file_attr(&self) -> Ref<'_, u32> {
        self.ext_file_attr.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn ofs_local_header(&self) -> Ref<'_, i32> {
        self.ofs_local_header.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn file_name(&self) -> Ref<'_, String> {
        self.file_name.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn extra(&self) -> Ref<'_, OptRc<Zip_Extras>> {
        self.extra.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn comment(&self) -> Ref<'_, String> {
        self.comment.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn file_mod_time_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_mod_time_raw.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn file_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_name_raw.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn extra_raw(&self) -> Ref<'_, Vec<u8>> {
        self.extra_raw.borrow()
    }
}
impl Zip_CentralDirEntry {
    pub fn comment_raw(&self) -> Ref<'_, Vec<u8>> {
        self.comment_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_DataDescriptor {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_PkSection>,
    pub(crate) _self_shared: SharedType<Self>,
    crc32: RefCell<u32>,
    len_body_compressed: RefCell<u32>,
    len_body_uncompressed: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip_DataDescriptor {
    type Root = Zip;
    type Parent = Zip_PkSection;

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
        *self_rc.crc32.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_compressed.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_uncompressed.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_DataDescriptor {
}
impl Zip_DataDescriptor {
    pub fn crc32(&self) -> Ref<'_, u32> {
        self.crc32.borrow()
    }
}
impl Zip_DataDescriptor {
    pub fn len_body_compressed(&self) -> Ref<'_, u32> {
        self.len_body_compressed.borrow()
    }
}
impl Zip_DataDescriptor {
    pub fn len_body_uncompressed(&self) -> Ref<'_, u32> {
        self.len_body_uncompressed.borrow()
    }
}
impl Zip_DataDescriptor {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_EndOfCentralDir {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_PkSection>,
    pub(crate) _self_shared: SharedType<Self>,
    disk_of_end_of_central_dir: RefCell<u16>,
    disk_of_central_dir: RefCell<u16>,
    num_central_dir_entries_on_disk: RefCell<u16>,
    num_central_dir_entries_total: RefCell<u16>,
    len_central_dir: RefCell<u32>,
    ofs_central_dir: RefCell<u32>,
    len_comment: RefCell<u16>,
    comment: RefCell<String>,
    _io: RefCell<BytesReader>,
    comment_raw: RefCell<Vec<u8>>,
}
impl KStruct for Zip_EndOfCentralDir {
    type Root = Zip;
    type Parent = Zip_PkSection;

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
        *self_rc.disk_of_end_of_central_dir.borrow_mut() = _io.read_u2le()?;
        *self_rc.disk_of_central_dir.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_central_dir_entries_on_disk.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_central_dir_entries_total.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_central_dir.borrow_mut() = _io.read_u4le()?;
        *self_rc.ofs_central_dir.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_comment.borrow_mut() = _io.read_u2le()?;
        *self_rc.comment.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_comment()))?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_EndOfCentralDir {
}
impl Zip_EndOfCentralDir {
    pub fn disk_of_end_of_central_dir(&self) -> Ref<'_, u16> {
        self.disk_of_end_of_central_dir.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn disk_of_central_dir(&self) -> Ref<'_, u16> {
        self.disk_of_central_dir.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn num_central_dir_entries_on_disk(&self) -> Ref<'_, u16> {
        self.num_central_dir_entries_on_disk.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn num_central_dir_entries_total(&self) -> Ref<'_, u16> {
        self.num_central_dir_entries_total.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn len_central_dir(&self) -> Ref<'_, u32> {
        self.len_central_dir.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn ofs_central_dir(&self) -> Ref<'_, u32> {
        self.ofs_central_dir.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn len_comment(&self) -> Ref<'_, u16> {
        self.len_comment.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn comment(&self) -> Ref<'_, String> {
        self.comment.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_EndOfCentralDir {
    pub fn comment_raw(&self) -> Ref<'_, Vec<u8>> {
        self.comment_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_Extras>,
    pub(crate) _self_shared: SharedType<Self>,
    code: RefCell<Zip_ExtraCodes>,
    len_body: RefCell<u16>,
    body: RefCell<Option<Zip_ExtraField_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Zip_ExtraField_Body {
    Zip_ExtraField_ExtendedTimestamp(OptRc<Zip_ExtraField_ExtendedTimestamp>),
    Zip_ExtraField_InfozipUnixVarSize(OptRc<Zip_ExtraField_InfozipUnixVarSize>),
    Zip_ExtraField_Ntfs(OptRc<Zip_ExtraField_Ntfs>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Zip_ExtraField_Body> for OptRc<Zip_ExtraField_ExtendedTimestamp> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Body::Zip_ExtraField_ExtendedTimestamp(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_ExtraField_ExtendedTimestamp>> for Zip_ExtraField_Body {
    fn from(v: OptRc<Zip_ExtraField_ExtendedTimestamp>) -> Self {
        Self::Zip_ExtraField_ExtendedTimestamp(v)
    }
}
impl TryFrom<&Zip_ExtraField_Body> for OptRc<Zip_ExtraField_InfozipUnixVarSize> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Body::Zip_ExtraField_InfozipUnixVarSize(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_ExtraField_InfozipUnixVarSize>> for Zip_ExtraField_Body {
    fn from(v: OptRc<Zip_ExtraField_InfozipUnixVarSize>) -> Self {
        Self::Zip_ExtraField_InfozipUnixVarSize(v)
    }
}
impl TryFrom<&Zip_ExtraField_Body> for OptRc<Zip_ExtraField_Ntfs> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Body::Zip_ExtraField_Ntfs(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_ExtraField_Ntfs>> for Zip_ExtraField_Body {
    fn from(v: OptRc<Zip_ExtraField_Ntfs>) -> Self {
        Self::Zip_ExtraField_Ntfs(v)
    }
}
impl TryFrom<&Zip_ExtraField_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Zip_ExtraField_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Zip_ExtraField {
    type Root = Zip;
    type Parent = Zip_Extras;

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
        *self_rc.code.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        *self_rc.len_body.borrow_mut() = _io.read_u2le()?;
        match *self_rc.code() {
            Zip_ExtraCodes::ExtendedTimestamp => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_body()))?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, Zip_ExtraField_ExtendedTimestamp>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            Zip_ExtraCodes::InfozipUnixVarSize => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_body()))?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, Zip_ExtraField_InfozipUnixVarSize>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            Zip_ExtraCodes::Ntfs => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_body()))?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, Zip_ExtraField_Ntfs>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField {
}
impl Zip_ExtraField {
    pub fn code(&self) -> Ref<'_, Zip_ExtraCodes> {
        self.code.borrow()
    }
}
impl Zip_ExtraField {
    pub fn len_body(&self) -> Ref<'_, u16> {
        self.len_body.borrow()
    }
}
impl Zip_ExtraField {
    pub fn body(&self) -> Ref<'_, Option<Zip_ExtraField_Body>> {
        self.body.borrow()
    }
}
impl Zip_ExtraField {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_ExtraField {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

/**
 * \sa <https://github.com/LuaDist/zip/blob/b710806/proginfo/extrafld.txt#L817> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_ExtendedTimestamp {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField>,
    pub(crate) _self_shared: SharedType<Self>,
    flags: RefCell<OptRc<Zip_ExtraField_ExtendedTimestamp_InfoFlags>>,
    mod_time: RefCell<u32>,
    access_time: RefCell<u32>,
    create_time: RefCell<u32>,
    _io: RefCell<BytesReader>,
    flags_raw: RefCell<Vec<u8>>,
}
impl KStruct for Zip_ExtraField_ExtendedTimestamp {
    type Root = Zip;
    type Parent = Zip_ExtraField;

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
        let _raw_flags = _io.read_bytes(1_usize)?;
        *self_rc.flags_raw.borrow_mut() = _raw_flags.clone();
        let _io_flags = BytesReader::from(_raw_flags);
        let t = Self::read_into::<BytesReader, Zip_ExtraField_ExtendedTimestamp_InfoFlags>(&_io_flags, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        if *self_rc.flags().has_mod_time() {
            *self_rc.mod_time.borrow_mut() = _io.read_u4le()?;
        }
        if *self_rc.flags().has_access_time() {
            *self_rc.access_time.borrow_mut() = _io.read_u4le()?;
        }
        if *self_rc.flags().has_create_time() {
            *self_rc.create_time.borrow_mut() = _io.read_u4le()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_ExtendedTimestamp {
}
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn flags(&self) -> Ref<'_, OptRc<Zip_ExtraField_ExtendedTimestamp_InfoFlags>> {
        self.flags.borrow()
    }
}

/**
 * Unix timestamp
 */
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn mod_time(&self) -> Ref<'_, u32> {
        self.mod_time.borrow()
    }
}

/**
 * Unix timestamp
 */
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn access_time(&self) -> Ref<'_, u32> {
        self.access_time.borrow()
    }
}

/**
 * Unix timestamp
 */
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn create_time(&self) -> Ref<'_, u32> {
        self.create_time.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp {
    pub fn flags_raw(&self) -> Ref<'_, Vec<u8>> {
        self.flags_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField_ExtendedTimestamp>,
    pub(crate) _self_shared: SharedType<Self>,
    has_mod_time: RefCell<bool>,
    has_access_time: RefCell<bool>,
    has_create_time: RefCell<bool>,
    reserved: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    type Root = Zip;
    type Parent = Zip_ExtraField_ExtendedTimestamp;

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
        *self_rc.has_mod_time.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.has_access_time.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.has_create_time.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_le(5)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub fn has_mod_time(&self) -> Ref<'_, bool> {
        self.has_mod_time.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub fn has_access_time(&self) -> Ref<'_, bool> {
        self.has_access_time.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub fn has_create_time(&self) -> Ref<'_, bool> {
        self.has_create_time.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Zip_ExtraField_ExtendedTimestamp_InfoFlags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://github.com/LuaDist/zip/blob/b710806/proginfo/extrafld.txt#L1339> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_InfozipUnixVarSize {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u8>,
    len_uid: RefCell<u8>,
    uid: RefCell<Vec<u8>>,
    len_gid: RefCell<u8>,
    gid: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    uid_raw: RefCell<Vec<u8>>,
    gid_raw: RefCell<Vec<u8>>,
}
impl KStruct for Zip_ExtraField_InfozipUnixVarSize {
    type Root = Zip;
    type Parent = Zip_ExtraField;

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
        *self_rc.version.borrow_mut() = _io.read_u1()?;
        *self_rc.len_uid.borrow_mut() = _io.read_u1()?;
        *self_rc.uid.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_uid()))?;
        *self_rc.len_gid.borrow_mut() = _io.read_u1()?;
        *self_rc.gid.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_gid()))?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_InfozipUnixVarSize {
}

/**
 * Version of this extra field, currently 1
 */
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn version(&self) -> Ref<'_, u8> {
        self.version.borrow()
    }
}

/**
 * Size of UID field
 */
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn len_uid(&self) -> Ref<'_, u8> {
        self.len_uid.borrow()
    }
}

/**
 * UID (User ID) for a file
 */
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn uid(&self) -> Ref<'_, Vec<u8>> {
        self.uid.borrow()
    }
}

/**
 * Size of GID field
 */
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn len_gid(&self) -> Ref<'_, u8> {
        self.len_gid.borrow()
    }
}

/**
 * GID (Group ID) for a file
 */
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn gid(&self) -> Ref<'_, Vec<u8>> {
        self.gid.borrow()
    }
}
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn uid_raw(&self) -> Ref<'_, Vec<u8>> {
        self.uid_raw.borrow()
    }
}
impl Zip_ExtraField_InfozipUnixVarSize {
    pub fn gid_raw(&self) -> Ref<'_, Vec<u8>> {
        self.gid_raw.borrow()
    }
}

/**
 * \sa <https://github.com/LuaDist/zip/blob/b710806/proginfo/extrafld.txt#L191> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_Ntfs {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u32>,
    attributes: RefCell<Vec<OptRc<Zip_ExtraField_Ntfs_Attribute>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip_ExtraField_Ntfs {
    type Root = Zip;
    type Parent = Zip_ExtraField;

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
        *self_rc.reserved.borrow_mut() = _io.read_u4le()?;
        *self_rc.attributes.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Zip_ExtraField_Ntfs_Attribute>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.attributes.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_Ntfs {
}
impl Zip_ExtraField_Ntfs {
    pub fn reserved(&self) -> Ref<'_, u32> {
        self.reserved.borrow()
    }
}
impl Zip_ExtraField_Ntfs {
    pub fn attributes(&self) -> Ref<'_, Vec<OptRc<Zip_ExtraField_Ntfs_Attribute>>> {
        self.attributes.borrow()
    }
}
impl Zip_ExtraField_Ntfs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_Ntfs_Attribute {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField_Ntfs>,
    pub(crate) _self_shared: SharedType<Self>,
    tag: RefCell<u16>,
    len_body: RefCell<u16>,
    body: RefCell<Option<Zip_ExtraField_Ntfs_Attribute_Body>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Zip_ExtraField_Ntfs_Attribute_Body {
    Zip_ExtraField_Ntfs_Attribute1(OptRc<Zip_ExtraField_Ntfs_Attribute1>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Zip_ExtraField_Ntfs_Attribute_Body> for OptRc<Zip_ExtraField_Ntfs_Attribute1> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Ntfs_Attribute_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Ntfs_Attribute_Body::Zip_ExtraField_Ntfs_Attribute1(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_ExtraField_Ntfs_Attribute1>> for Zip_ExtraField_Ntfs_Attribute_Body {
    fn from(v: OptRc<Zip_ExtraField_Ntfs_Attribute1>) -> Self {
        Self::Zip_ExtraField_Ntfs_Attribute1(v)
    }
}
impl TryFrom<&Zip_ExtraField_Ntfs_Attribute_Body> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Zip_ExtraField_Ntfs_Attribute_Body) -> Result<Self, Self::Error> {
        if let Zip_ExtraField_Ntfs_Attribute_Body::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Zip_ExtraField_Ntfs_Attribute_Body {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Zip_ExtraField_Ntfs_Attribute {
    type Root = Zip;
    type Parent = Zip_ExtraField_Ntfs;

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
        *self_rc.tag.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_body.borrow_mut() = _io.read_u2le()?;
        match *self_rc.tag() {
            1 => {
                *self_rc.body_raw.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_body()))?.into();
                let body_raw = self_rc.body_raw.borrow();
                let _t_body_raw_io = BytesReader::from(body_raw.clone());
                let t = Self::read_into::<BytesReader, Zip_ExtraField_Ntfs_Attribute1>(&_t_body_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.body.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_Ntfs_Attribute {
}
impl Zip_ExtraField_Ntfs_Attribute {
    pub fn tag(&self) -> Ref<'_, u16> {
        self.tag.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute {
    pub fn len_body(&self) -> Ref<'_, u16> {
        self.len_body.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute {
    pub fn body(&self) -> Ref<'_, Option<Zip_ExtraField_Ntfs_Attribute_Body>> {
        self.body.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_ExtraField_Ntfs_Attribute1 {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_ExtraField_Ntfs_Attribute>,
    pub(crate) _self_shared: SharedType<Self>,
    last_mod_time: RefCell<u64>,
    last_access_time: RefCell<u64>,
    creation_time: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip_ExtraField_Ntfs_Attribute1 {
    type Root = Zip;
    type Parent = Zip_ExtraField_Ntfs_Attribute;

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
        *self_rc.last_mod_time.borrow_mut() = _io.read_u8le()?;
        *self_rc.last_access_time.borrow_mut() = _io.read_u8le()?;
        *self_rc.creation_time.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_ExtraField_Ntfs_Attribute1 {
}
impl Zip_ExtraField_Ntfs_Attribute1 {
    pub fn last_mod_time(&self) -> Ref<'_, u64> {
        self.last_mod_time.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute1 {
    pub fn last_access_time(&self) -> Ref<'_, u64> {
        self.last_access_time.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute1 {
    pub fn creation_time(&self) -> Ref<'_, u64> {
        self.creation_time.borrow()
    }
}
impl Zip_ExtraField_Ntfs_Attribute1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_Extras {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Zip_ExtraField>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Zip_Extras {
    type Root = Zip;
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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Zip_ExtraField>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_Extras {
}
impl Zip_Extras {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Zip_ExtraField>>> {
        self.entries.borrow()
    }
}
impl Zip_Extras {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_LocalFile {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_PkSection>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Zip_LocalFileHeader>>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
impl KStruct for Zip_LocalFile {
    type Root = Zip;
    type Parent = Zip_PkSection;

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
        let t = Self::read_into::<_, Zip_LocalFileHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.header().len_body_compressed())?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_LocalFile {
}
impl Zip_LocalFile {
    pub fn header(&self) -> Ref<'_, OptRc<Zip_LocalFileHeader>> {
        self.header.borrow()
    }
}
impl Zip_LocalFile {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl Zip_LocalFile {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_LocalFile {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Zip_LocalFileHeader {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_LocalFile>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u16>,
    flags: RefCell<OptRc<Zip_LocalFileHeader_GpFlags>>,
    compression_method: RefCell<Zip_Compression>,
    file_mod_time: RefCell<OptRc<DosDatetime>>,
    crc32: RefCell<u32>,
    len_body_compressed: RefCell<u32>,
    len_body_uncompressed: RefCell<u32>,
    len_file_name: RefCell<u16>,
    len_extra: RefCell<u16>,
    file_name: RefCell<String>,
    extra: RefCell<OptRc<Zip_Extras>>,
    _io: RefCell<BytesReader>,
    flags_raw: RefCell<Vec<u8>>,
    file_mod_time_raw: RefCell<Vec<u8>>,
    file_name_raw: RefCell<Vec<u8>>,
    extra_raw: RefCell<Vec<u8>>,
}
impl KStruct for Zip_LocalFileHeader {
    type Root = Zip;
    type Parent = Zip_LocalFile;

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
        *self_rc.version.borrow_mut() = _io.read_u2le()?;
        let _raw_flags = _io.read_bytes(2_usize)?;
        *self_rc.flags_raw.borrow_mut() = _raw_flags.clone();
        let _io_flags = BytesReader::from(_raw_flags);
        let t = Self::read_into::<BytesReader, Zip_LocalFileHeader_GpFlags>(&_io_flags, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        *self_rc.compression_method.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        let _raw_file_mod_time = _io.read_bytes(4_usize)?;
        *self_rc.file_mod_time_raw.borrow_mut() = _raw_file_mod_time.clone();
        let _io_file_mod_time = BytesReader::from(_raw_file_mod_time);
        let t = Self::read_into::<BytesReader, DosDatetime>(&_io_file_mod_time, None, None)?.into();
        *self_rc.file_mod_time.borrow_mut() = t;
        *self_rc.crc32.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_compressed.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body_uncompressed.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_file_name.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_extra.borrow_mut() = _io.read_u2le()?;
        *self_rc.file_name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_file_name()))?, "UTF-8")?;
        let _raw_extra = _io.read_bytes(usize::from(*self_rc.len_extra()))?;
        *self_rc.extra_raw.borrow_mut() = _raw_extra.clone();
        let _io_extra = BytesReader::from(_raw_extra);
        let t = Self::read_into::<BytesReader, Zip_Extras>(&_io_extra, Some(self_rc._root.clone()), None)?.into();
        *self_rc.extra.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_LocalFileHeader {
}
impl Zip_LocalFileHeader {
    pub fn version(&self) -> Ref<'_, u16> {
        self.version.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn flags(&self) -> Ref<'_, OptRc<Zip_LocalFileHeader_GpFlags>> {
        self.flags.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn compression_method(&self) -> Ref<'_, Zip_Compression> {
        self.compression_method.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn file_mod_time(&self) -> Ref<'_, OptRc<DosDatetime>> {
        self.file_mod_time.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn crc32(&self) -> Ref<'_, u32> {
        self.crc32.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn len_body_compressed(&self) -> Ref<'_, u32> {
        self.len_body_compressed.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn len_body_uncompressed(&self) -> Ref<'_, u32> {
        self.len_body_uncompressed.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn len_file_name(&self) -> Ref<'_, u16> {
        self.len_file_name.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn len_extra(&self) -> Ref<'_, u16> {
        self.len_extra.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn file_name(&self) -> Ref<'_, String> {
        self.file_name.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn extra(&self) -> Ref<'_, OptRc<Zip_Extras>> {
        self.extra.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn flags_raw(&self) -> Ref<'_, Vec<u8>> {
        self.flags_raw.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn file_mod_time_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_mod_time_raw.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn file_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.file_name_raw.borrow()
    }
}
impl Zip_LocalFileHeader {
    pub fn extra_raw(&self) -> Ref<'_, Vec<u8>> {
        self.extra_raw.borrow()
    }
}

/**
 * \sa https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT - 4.4.4
 * \sa https://users.cs.jmu.edu/buchhofp/forensics/formats/pkzip.html Local file headers
 */

#[derive(Default, Debug, Clone)]
pub struct Zip_LocalFileHeader_GpFlags {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<Zip_LocalFileHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    file_encrypted: RefCell<bool>,
    comp_options_raw: RefCell<u64>,
    has_data_descriptor: RefCell<bool>,
    reserved_1: RefCell<bool>,
    comp_patched_data: RefCell<bool>,
    strong_encrypt: RefCell<bool>,
    reserved_2: RefCell<u64>,
    lang_encoding: RefCell<bool>,
    reserved_3: RefCell<bool>,
    mask_header_values: RefCell<bool>,
    reserved_4: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_deflated_mode: Cell<bool>,
    deflated_mode: RefCell<Zip_LocalFileHeader_GpFlags_DeflateMode>,
    f_imploded_dict_byte_size: Cell<bool>,
    imploded_dict_byte_size: RefCell<i32>,
    f_imploded_num_sf_trees: Cell<bool>,
    imploded_num_sf_trees: RefCell<i32>,
    f_lzma_has_eos_marker: Cell<bool>,
    lzma_has_eos_marker: RefCell<bool>,
}
impl KStruct for Zip_LocalFileHeader_GpFlags {
    type Root = Zip;
    type Parent = Zip_LocalFileHeader;

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
        *self_rc.file_encrypted.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.comp_options_raw.borrow_mut() = _io.read_bits_int_le(2)?;
        *self_rc.has_data_descriptor.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.reserved_1.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.comp_patched_data.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.strong_encrypt.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.reserved_2.borrow_mut() = _io.read_bits_int_le(4)?;
        *self_rc.lang_encoding.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.reserved_3.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.mask_header_values.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.reserved_4.borrow_mut() = _io.read_bits_int_le(2)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_LocalFileHeader_GpFlags {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn deflated_mode(
        &self
    ) -> KResult<Ref<'_, Zip_LocalFileHeader_GpFlags_DeflateMode>> {
        let _io = self._io.borrow();
        if self.f_deflated_mode.get() {
            return Ok(self.deflated_mode.borrow());
        }
        self.f_deflated_mode.set(true);
        if  ((*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.compression_method() == Zip_Compression::Deflated) || (*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.compression_method() == Zip_Compression::EnhancedDeflated))  {
            *self.deflated_mode.borrow_mut() = i64::try_from(*self.comp_options_raw())?.try_into()?;
        }
        Ok(self.deflated_mode.borrow())
    }

    /**
     * 8KiB or 4KiB in bytes
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn imploded_dict_byte_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_imploded_dict_byte_size.get() {
            return Ok(self.imploded_dict_byte_size.borrow());
        }
        self.f_imploded_dict_byte_size.set(true);
        if *self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.compression_method() == Zip_Compression::Imploded {
            *self.imploded_dict_byte_size.borrow_mut() = ((if ((to_i128(((*self.comp_options_raw()) & (1_u64)))) != (to_i128(0))) { 8_i32 } else { 4_i32 }).saturating_mul(1024_i32)).try_into()?;
        }
        Ok(self.imploded_dict_byte_size.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn imploded_num_sf_trees(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_imploded_num_sf_trees.get() {
            return Ok(self.imploded_num_sf_trees.borrow());
        }
        self.f_imploded_num_sf_trees.set(true);
        if *self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.compression_method() == Zip_Compression::Imploded {
            *self.imploded_num_sf_trees.borrow_mut() = (if ((to_i128(((*self.comp_options_raw()) & (2_u64)))) != (to_i128(0))) { 3_i32 } else { 2_i32 }).try_into()?;
        }
        Ok(self.imploded_num_sf_trees.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn lzma_has_eos_marker(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_lzma_has_eos_marker.get() {
            return Ok(self.lzma_has_eos_marker.borrow());
        }
        self.f_lzma_has_eos_marker.set(true);
        if *self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.compression_method() == Zip_Compression::Lzma {
            *self.lzma_has_eos_marker.borrow_mut() = (((to_i128(((*self.comp_options_raw()) & (1_u64)))) != (to_i128(0)))).try_into()?;
        }
        Ok(self.lzma_has_eos_marker.borrow())
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn file_encrypted(&self) -> Ref<'_, bool> {
        self.file_encrypted.borrow()
    }
}

/**
 * internal; access derived value instances instead
 */
impl Zip_LocalFileHeader_GpFlags {
    pub fn comp_options_raw(&self) -> Ref<'_, u64> {
        self.comp_options_raw.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn has_data_descriptor(&self) -> Ref<'_, bool> {
        self.has_data_descriptor.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn reserved_1(&self) -> Ref<'_, bool> {
        self.reserved_1.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn comp_patched_data(&self) -> Ref<'_, bool> {
        self.comp_patched_data.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn strong_encrypt(&self) -> Ref<'_, bool> {
        self.strong_encrypt.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn reserved_2(&self) -> Ref<'_, u64> {
        self.reserved_2.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn lang_encoding(&self) -> Ref<'_, bool> {
        self.lang_encoding.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn reserved_3(&self) -> Ref<'_, bool> {
        self.reserved_3.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn mask_header_values(&self) -> Ref<'_, bool> {
        self.mask_header_values.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn reserved_4(&self) -> Ref<'_, u64> {
        self.reserved_4.borrow()
    }
}
impl Zip_LocalFileHeader_GpFlags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Zip_LocalFileHeader_GpFlags_DeflateMode {
    Normal,
    Maximum,
    Fast,
    SuperFast,
    Unknown(i64),
}

impl TryFrom<i64> for Zip_LocalFileHeader_GpFlags_DeflateMode {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Zip_LocalFileHeader_GpFlags_DeflateMode> {
        match flag {
            0 => Ok(Zip_LocalFileHeader_GpFlags_DeflateMode::Normal),
            1 => Ok(Zip_LocalFileHeader_GpFlags_DeflateMode::Maximum),
            2 => Ok(Zip_LocalFileHeader_GpFlags_DeflateMode::Fast),
            3 => Ok(Zip_LocalFileHeader_GpFlags_DeflateMode::SuperFast),
            _ => Ok(Zip_LocalFileHeader_GpFlags_DeflateMode::Unknown(flag)),
        }
    }
}

impl From<&Zip_LocalFileHeader_GpFlags_DeflateMode> for i64 {
    fn from(v: &Zip_LocalFileHeader_GpFlags_DeflateMode) -> Self {
        match *v {
            Zip_LocalFileHeader_GpFlags_DeflateMode::Normal => 0,
            Zip_LocalFileHeader_GpFlags_DeflateMode::Maximum => 1,
            Zip_LocalFileHeader_GpFlags_DeflateMode::Fast => 2,
            Zip_LocalFileHeader_GpFlags_DeflateMode::SuperFast => 3,
            Zip_LocalFileHeader_GpFlags_DeflateMode::Unknown(v) => v
        }
    }
}

impl Default for Zip_LocalFileHeader_GpFlags_DeflateMode {
    fn default() -> Self { Zip_LocalFileHeader_GpFlags_DeflateMode::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Zip_PkSection {
    pub(crate) _root: SharedType<Zip>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    section_type: RefCell<u16>,
    body: RefCell<Option<Zip_PkSection_Body>>,
    _io: RefCell<BytesReader>,
}
#[derive(Debug, Clone)]
pub enum Zip_PkSection_Body {
    Zip_LocalFile(OptRc<Zip_LocalFile>),
    Zip_EndOfCentralDir(OptRc<Zip_EndOfCentralDir>),
    Zip_DataDescriptor(OptRc<Zip_DataDescriptor>),
    Zip_CentralDirEntry(OptRc<Zip_CentralDirEntry>),
}
impl TryFrom<&Zip_PkSection_Body> for OptRc<Zip_LocalFile> {
    type Error = KError;
    fn try_from(v: &Zip_PkSection_Body) -> Result<Self, Self::Error> {
        if let Zip_PkSection_Body::Zip_LocalFile(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_LocalFile>> for Zip_PkSection_Body {
    fn from(v: OptRc<Zip_LocalFile>) -> Self {
        Self::Zip_LocalFile(v)
    }
}
impl TryFrom<&Zip_PkSection_Body> for OptRc<Zip_EndOfCentralDir> {
    type Error = KError;
    fn try_from(v: &Zip_PkSection_Body) -> Result<Self, Self::Error> {
        if let Zip_PkSection_Body::Zip_EndOfCentralDir(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_EndOfCentralDir>> for Zip_PkSection_Body {
    fn from(v: OptRc<Zip_EndOfCentralDir>) -> Self {
        Self::Zip_EndOfCentralDir(v)
    }
}
impl TryFrom<&Zip_PkSection_Body> for OptRc<Zip_DataDescriptor> {
    type Error = KError;
    fn try_from(v: &Zip_PkSection_Body) -> Result<Self, Self::Error> {
        if let Zip_PkSection_Body::Zip_DataDescriptor(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_DataDescriptor>> for Zip_PkSection_Body {
    fn from(v: OptRc<Zip_DataDescriptor>) -> Self {
        Self::Zip_DataDescriptor(v)
    }
}
impl TryFrom<&Zip_PkSection_Body> for OptRc<Zip_CentralDirEntry> {
    type Error = KError;
    fn try_from(v: &Zip_PkSection_Body) -> Result<Self, Self::Error> {
        if let Zip_PkSection_Body::Zip_CentralDirEntry(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Zip_CentralDirEntry>> for Zip_PkSection_Body {
    fn from(v: OptRc<Zip_CentralDirEntry>) -> Self {
        Self::Zip_CentralDirEntry(v)
    }
}
impl KStruct for Zip_PkSection {
    type Root = Zip;
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
        *self_rc.magic.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.magic() == vec![0x50u8, 0x4bu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/pk_section/seq/0".to_string() }));
        }
        *self_rc.section_type.borrow_mut() = _io.read_u2le()?;
        match *self_rc.section_type() {
            1027 => {
                let t = Self::read_into::<_, Zip_LocalFile>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            1541 => {
                let t = Self::read_into::<_, Zip_EndOfCentralDir>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            2055 => {
                let t = Self::read_into::<_, Zip_DataDescriptor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            513 => {
                let t = Self::read_into::<_, Zip_CentralDirEntry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.body.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Zip_PkSection {
}
impl Zip_PkSection {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Zip_PkSection {
    pub fn section_type(&self) -> Ref<'_, u16> {
        self.section_type.borrow()
    }
}
impl Zip_PkSection {
    pub fn body(&self) -> Ref<'_, Option<Zip_PkSection_Body>> {
        self.body.borrow()
    }
}
impl Zip_PkSection {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
