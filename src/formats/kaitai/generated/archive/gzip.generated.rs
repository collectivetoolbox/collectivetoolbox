// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Gzip is a popular and standard single-file archiving format. It
 * essentially provides a container that stores original file name,
 * timestamp and a few other things (like optional comment), basic
 * CRCs, etc, and a file compressed by a chosen compression algorithm.
 *
 * As of 2019, there is actually only one working solution for
 * compression algorithms, so it's typically raw DEFLATE stream
 * (without zlib header) in all gzipped files.
 * \sa <https://www.rfc-editor.org/rfc/rfc1952> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Gzip {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    compression_method: RefCell<Gzip_CompressionMethods>,
    flags: RefCell<OptRc<Gzip_Flags>>,
    mod_time: RefCell<u32>,
    extra_flags: RefCell<Option<Gzip_ExtraFlags>>,
    os: RefCell<Gzip_Oses>,
    extras: RefCell<OptRc<Gzip_Extras>>,
    name: RefCell<Vec<u8>>,
    comment: RefCell<Vec<u8>>,
    header_crc16: RefCell<u16>,
    body: RefCell<Vec<u8>>,
    body_crc32: RefCell<u32>,
    len_uncompressed: RefCell<u32>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Gzip_ExtraFlags {
    Gzip_ExtraFlagsDeflate(OptRc<Gzip_ExtraFlagsDeflate>),
}
impl TryFrom<&Gzip_ExtraFlags> for OptRc<Gzip_ExtraFlagsDeflate> {
    type Error = KError;
    fn try_from(v: &Gzip_ExtraFlags) -> Result<Self, Self::Error> {
        if let Gzip_ExtraFlags::Gzip_ExtraFlagsDeflate(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Gzip_ExtraFlagsDeflate>> for Gzip_ExtraFlags {
    fn from(v: OptRc<Gzip_ExtraFlagsDeflate>) -> Self {
        Self::Gzip_ExtraFlagsDeflate(v)
    }
}
impl KStruct for Gzip {
    type Root = Gzip;
    type Parent = Gzip;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.magic() == vec![0x1fu8, 0x8bu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.compression_method.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        let t = Self::read_into::<_, Gzip_Flags>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.flags.borrow_mut() = t;
        *self_rc.mod_time.borrow_mut() = _io.read_u4le()?;
        match *self_rc.compression_method() {
            Gzip_CompressionMethods::Deflate => {
                let t = Self::read_into::<_, Gzip_ExtraFlagsDeflate>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.extra_flags.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc.os.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        if *self_rc.flags().has_extra() {
            let t = Self::read_into::<_, Gzip_Extras>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.extras.borrow_mut() = t;
        }
        if *self_rc.flags().has_name() {
            *self_rc.name.borrow_mut() = _io.read_bytes_term(0, false, true, true)?;
        }
        if *self_rc.flags().has_comment() {
            *self_rc.comment.borrow_mut() = _io.read_bytes_term(0, false, true, true)?;
        }
        if *self_rc.flags().has_header_crc() {
            *self_rc.header_crc16.borrow_mut() = _io.read_u2le()?;
        }
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(((usize::try_from((i64::try_from(_io.size())?))?).saturating_sub(_io.pos())).saturating_sub(8_usize))?)?;
        *self_rc.body_crc32.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_uncompressed.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip {
}
impl Gzip {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * Compression method used to compress file body. In practice, only
 * one method is widely used: 8 = deflate.
 */
impl Gzip {
    pub fn compression_method(&self) -> Ref<'_, Gzip_CompressionMethods> {
        self.compression_method.borrow()
    }
}
impl Gzip {
    pub fn flags(&self) -> Ref<'_, OptRc<Gzip_Flags>> {
        self.flags.borrow()
    }
}

/**
 * Last modification time of a file archived in UNIX timestamp format.
 */
impl Gzip {
    pub fn mod_time(&self) -> Ref<'_, u32> {
        self.mod_time.borrow()
    }
}

/**
 * Extra flags, specific to compression method chosen.
 */
impl Gzip {
    pub fn extra_flags(&self) -> Ref<'_, Option<Gzip_ExtraFlags>> {
        self.extra_flags.borrow()
    }
}

/**
 * OS used to compress this file.
 */
impl Gzip {
    pub fn os(&self) -> Ref<'_, Gzip_Oses> {
        self.os.borrow()
    }
}
impl Gzip {
    pub fn extras(&self) -> Ref<'_, OptRc<Gzip_Extras>> {
        self.extras.borrow()
    }
}
impl Gzip {
    pub fn name(&self) -> Ref<'_, Vec<u8>> {
        self.name.borrow()
    }
}
impl Gzip {
    pub fn comment(&self) -> Ref<'_, Vec<u8>> {
        self.comment.borrow()
    }
}
impl Gzip {
    pub fn header_crc16(&self) -> Ref<'_, u16> {
        self.header_crc16.borrow()
    }
}

/**
 * Compressed body of a file archived. Note that we don't make an
 * attempt to decompress it here.
 */
impl Gzip {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}

/**
 * CRC32 checksum of an uncompressed file body
 */
impl Gzip {
    pub fn body_crc32(&self) -> Ref<'_, u32> {
        self.body_crc32.borrow()
    }
}

/**
 * Size of original uncompressed data in bytes (truncated to 32
 * bits).
 */
impl Gzip {
    pub fn len_uncompressed(&self) -> Ref<'_, u32> {
        self.len_uncompressed.borrow()
    }
}
impl Gzip {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Gzip {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Gzip_CompressionMethods {
    Deflate,
    Unknown(i64),
}

impl TryFrom<i64> for Gzip_CompressionMethods {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Gzip_CompressionMethods> {
        match flag {
            8 => Ok(Gzip_CompressionMethods::Deflate),
            _ => Ok(Gzip_CompressionMethods::Unknown(flag)),
        }
    }
}

impl From<&Gzip_CompressionMethods> for i64 {
    fn from(v: &Gzip_CompressionMethods) -> Self {
        match *v {
            Gzip_CompressionMethods::Deflate => 8,
            Gzip_CompressionMethods::Unknown(v) => v
        }
    }
}

impl Default for Gzip_CompressionMethods {
    fn default() -> Self { Gzip_CompressionMethods::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Gzip_Oses {

    /**
     * FAT filesystem (MS-DOS, OS/2, NT/Win32)
     */
    Fat,

    /**
     * Amiga
     */
    Amiga,

    /**
     * VMS (or OpenVMS)
     */
    Vms,

    /**
     * Unix
     */
    Unix,

    /**
     * VM/CMS
     */
    VmCms,

    /**
     * Atari TOS
     */
    AtariTos,

    /**
     * HPFS filesystem (OS/2, NT)
     */
    Hpfs,

    /**
     * Macintosh
     */
    Macintosh,

    /**
     * Z-System
     */
    ZSystem,

    /**
     * CP/M
     */
    CpM,

    /**
     * TOPS-20
     */
    Tops20,

    /**
     * NTFS filesystem (NT)
     */
    Ntfs,

    /**
     * QDOS
     */
    Qdos,

    /**
     * Acorn RISCOS
     */
    AcornRiscos,
    Unknown,
    UnknownVariant(i64),
}

impl TryFrom<i64> for Gzip_Oses {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Gzip_Oses> {
        match flag {
            0 => Ok(Gzip_Oses::Fat),
            1 => Ok(Gzip_Oses::Amiga),
            2 => Ok(Gzip_Oses::Vms),
            3 => Ok(Gzip_Oses::Unix),
            4 => Ok(Gzip_Oses::VmCms),
            5 => Ok(Gzip_Oses::AtariTos),
            6 => Ok(Gzip_Oses::Hpfs),
            7 => Ok(Gzip_Oses::Macintosh),
            8 => Ok(Gzip_Oses::ZSystem),
            9 => Ok(Gzip_Oses::CpM),
            10 => Ok(Gzip_Oses::Tops20),
            11 => Ok(Gzip_Oses::Ntfs),
            12 => Ok(Gzip_Oses::Qdos),
            13 => Ok(Gzip_Oses::AcornRiscos),
            255 => Ok(Gzip_Oses::Unknown),
            _ => Ok(Gzip_Oses::UnknownVariant(flag)),
        }
    }
}

impl From<&Gzip_Oses> for i64 {
    fn from(v: &Gzip_Oses) -> Self {
        match *v {
            Gzip_Oses::Fat => 0,
            Gzip_Oses::Amiga => 1,
            Gzip_Oses::Vms => 2,
            Gzip_Oses::Unix => 3,
            Gzip_Oses::VmCms => 4,
            Gzip_Oses::AtariTos => 5,
            Gzip_Oses::Hpfs => 6,
            Gzip_Oses::Macintosh => 7,
            Gzip_Oses::ZSystem => 8,
            Gzip_Oses::CpM => 9,
            Gzip_Oses::Tops20 => 10,
            Gzip_Oses::Ntfs => 11,
            Gzip_Oses::Qdos => 12,
            Gzip_Oses::AcornRiscos => 13,
            Gzip_Oses::Unknown => 255,
            Gzip_Oses::UnknownVariant(v) => v
        }
    }
}

impl Default for Gzip_Oses {
    fn default() -> Self { Gzip_Oses::UnknownVariant(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Gzip_ExtraFlagsDeflate {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip>,
    pub(crate) _self_shared: SharedType<Self>,
    compression_strength: RefCell<Gzip_ExtraFlagsDeflate_CompressionStrengths>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Gzip_ExtraFlagsDeflate {
    type Root = Gzip;
    type Parent = Gzip;

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
        *self_rc.compression_strength.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip_ExtraFlagsDeflate {
}
impl Gzip_ExtraFlagsDeflate {
    pub fn compression_strength(&self) -> Ref<'_, Gzip_ExtraFlagsDeflate_CompressionStrengths> {
        self.compression_strength.borrow()
    }
}
impl Gzip_ExtraFlagsDeflate {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Gzip_ExtraFlagsDeflate_CompressionStrengths {
    Best,
    Fast,
    Unknown(i64),
}

impl TryFrom<i64> for Gzip_ExtraFlagsDeflate_CompressionStrengths {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Gzip_ExtraFlagsDeflate_CompressionStrengths> {
        match flag {
            2 => Ok(Gzip_ExtraFlagsDeflate_CompressionStrengths::Best),
            4 => Ok(Gzip_ExtraFlagsDeflate_CompressionStrengths::Fast),
            _ => Ok(Gzip_ExtraFlagsDeflate_CompressionStrengths::Unknown(flag)),
        }
    }
}

impl From<&Gzip_ExtraFlagsDeflate_CompressionStrengths> for i64 {
    fn from(v: &Gzip_ExtraFlagsDeflate_CompressionStrengths) -> Self {
        match *v {
            Gzip_ExtraFlagsDeflate_CompressionStrengths::Best => 2,
            Gzip_ExtraFlagsDeflate_CompressionStrengths::Fast => 4,
            Gzip_ExtraFlagsDeflate_CompressionStrengths::Unknown(v) => v
        }
    }
}

impl Default for Gzip_ExtraFlagsDeflate_CompressionStrengths {
    fn default() -> Self { Gzip_ExtraFlagsDeflate_CompressionStrengths::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Gzip_Extras {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip>,
    pub(crate) _self_shared: SharedType<Self>,
    len_subfields: RefCell<u16>,
    subfields: RefCell<OptRc<Gzip_Subfields>>,
    _io: RefCell<BytesReader>,
    subfields_raw: RefCell<Vec<u8>>,
}
impl KStruct for Gzip_Extras {
    type Root = Gzip;
    type Parent = Gzip;

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
        *self_rc.len_subfields.borrow_mut() = _io.read_u2le()?;
        let _raw_subfields = _io.read_bytes(usize::from(*self_rc.len_subfields()))?;
        *self_rc.subfields_raw.borrow_mut() = _raw_subfields.clone();
        let _io_subfields = BytesReader::from(_raw_subfields);
        let t = Self::read_into::<BytesReader, Gzip_Subfields>(&_io_subfields, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.subfields.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip_Extras {
}
impl Gzip_Extras {
    pub fn len_subfields(&self) -> Ref<'_, u16> {
        self.len_subfields.borrow()
    }
}
impl Gzip_Extras {
    pub fn subfields(&self) -> Ref<'_, OptRc<Gzip_Subfields>> {
        self.subfields.borrow()
    }
}
impl Gzip_Extras {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Gzip_Extras {
    pub fn subfields_raw(&self) -> Ref<'_, Vec<u8>> {
        self.subfields_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Gzip_Flags {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<u64>,
    has_comment: RefCell<bool>,
    has_name: RefCell<bool>,
    has_extra: RefCell<bool>,
    has_header_crc: RefCell<bool>,
    is_text: RefCell<bool>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Gzip_Flags {
    type Root = Gzip;
    type Parent = Gzip;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(3)?;
        *self_rc.has_comment.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.has_name.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.has_extra.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.has_header_crc.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.is_text.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip_Flags {
}
impl Gzip_Flags {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}
impl Gzip_Flags {
    pub fn has_comment(&self) -> Ref<'_, bool> {
        self.has_comment.borrow()
    }
}
impl Gzip_Flags {
    pub fn has_name(&self) -> Ref<'_, bool> {
        self.has_name.borrow()
    }
}

/**
 * If true, optional extra fields are present in the archive.
 */
impl Gzip_Flags {
    pub fn has_extra(&self) -> Ref<'_, bool> {
        self.has_extra.borrow()
    }
}

/**
 * If true, this archive includes a CRC16 checksum for the header.
 */
impl Gzip_Flags {
    pub fn has_header_crc(&self) -> Ref<'_, bool> {
        self.has_header_crc.borrow()
    }
}

/**
 * If true, file inside this archive is a text file from
 * compressor's point of view.
 */
impl Gzip_Flags {
    pub fn is_text(&self) -> Ref<'_, bool> {
        self.is_text.borrow()
    }
}
impl Gzip_Flags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Every subfield follows typical [TLV scheme](https://en.wikipedia.org/wiki/Type-length-value):
 *
 * * `id` serves role of "T"ype
 * * `len_data` serves role of "L"ength
 * * `data` serves role of "V"alue
 *
 * This way it's possible to for arbitrary parser to skip over
 * subfields it does not support.
 */

#[derive(Default, Debug, Clone)]
pub struct Gzip_Subfield {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip_Subfields>,
    pub(crate) _self_shared: SharedType<Self>,
    id: RefCell<u16>,
    len_data: RefCell<u16>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
}
impl KStruct for Gzip_Subfield {
    type Root = Gzip;
    type Parent = Gzip_Subfields;

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
        *self_rc.id.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_data.borrow_mut() = _io.read_u2le()?;
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::from(*self_rc.len_data()))?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip_Subfield {
}

/**
 * Subfield ID, typically two ASCII letters.
 */
impl Gzip_Subfield {
    pub fn id(&self) -> Ref<'_, u16> {
        self.id.borrow()
    }
}
impl Gzip_Subfield {
    pub fn len_data(&self) -> Ref<'_, u16> {
        self.len_data.borrow()
    }
}
impl Gzip_Subfield {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Gzip_Subfield {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Gzip_Subfield {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

/**
 * Container for many subfields, constrained by size of stream.
 */

#[derive(Default, Debug, Clone)]
pub struct Gzip_Subfields {
    pub(crate) _root: SharedType<Gzip>,
    pub(crate) _parent: SharedType<Gzip_Extras>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<Gzip_Subfield>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Gzip_Subfields {
    type Root = Gzip;
    type Parent = Gzip_Extras;

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
                let t = Self::read_into::<_, Gzip_Subfield>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Gzip_Subfields {
}
impl Gzip_Subfields {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<Gzip_Subfield>>> {
        self.entries.borrow()
    }
}
impl Gzip_Subfields {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
