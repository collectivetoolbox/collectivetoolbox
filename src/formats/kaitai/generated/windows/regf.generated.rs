// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * This spec allows to parse files used by Microsoft Windows family of
 * operating systems to store parts of its "registry". "Registry" is a
 * hierarchical database that is used to store system settings (global
 * configuration, per-user, per-application configuration, etc).
 *
 * Typically, registry files are stored in:
 *
 * * System-wide: several files in `%SystemRoot%\System32\Config\`
 * * User-wide:
 *   * `%USERPROFILE%\Ntuser.dat`
 *   * `%USERPROFILE%\Local Settings\Application Data\Microsoft\Windows\Usrclass.dat` (localized, Windows 2000, Server 2003 and Windows XP)
 *   * `%USERPROFILE%\AppData\Local\Microsoft\Windows\Usrclass.dat` (non-localized, Windows Vista and later)
 *
 * Note that one typically can't access files directly on a mounted
 * filesystem with a running Windows OS.
 * \sa <https://github.com/libyal/libregf/blob/main/documentation/Windows%20NT%20Registry%20File%20(REGF)%20format.asciidoc> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Regf {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Regf_FileHeader>>,
    hive_bins: RefCell<Vec<OptRc<Regf_HiveBin>>>,
    _io: RefCell<BytesReader>,
    hive_bins_raw: RefCell<Vec<u8>>,
}
impl KStruct for Regf {
    type Root = Regf;
    type Parent = Regf;

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
        let t = Self::read_into::<_, Regf_FileHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.hive_bins.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let _raw_hive_bins = _io.read_bytes(4096_usize)?;
                let _io_hive_bins = BytesReader::from(_raw_hive_bins);
                let t = Self::read_into::<BytesReader, Regf_HiveBin>(&_io_hive_bins, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.hive_bins.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf {
}
impl Regf {
    pub fn header(&self) -> Ref<'_, OptRc<Regf_FileHeader>> {
        self.header.borrow()
    }
}
impl Regf {
    pub fn hive_bins(&self) -> Ref<'_, Vec<OptRc<Regf_HiveBin>>> {
        self.hive_bins.borrow()
    }
}
impl Regf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Regf {
    pub fn hive_bins_raw(&self) -> Ref<'_, Vec<u8>> {
        self.hive_bins_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_FileHeader {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf>,
    pub(crate) _self_shared: SharedType<Self>,
    signature: RefCell<Vec<u8>>,
    primary_sequence_number: RefCell<u32>,
    secondary_sequence_number: RefCell<u32>,
    last_modification_date_and_time: RefCell<OptRc<Regf_Filetime>>,
    major_version: RefCell<u32>,
    minor_version: RefCell<u32>,
    r#type: RefCell<Regf_FileHeader_FileType>,
    format: RefCell<Regf_FileHeader_FileFormat>,
    root_key_offset: RefCell<u32>,
    hive_bins_data_size: RefCell<u32>,
    clustering_factor: RefCell<u32>,
    unknown1: RefCell<Vec<u8>>,
    unknown2: RefCell<Vec<u8>>,
    checksum: RefCell<u32>,
    reserved: RefCell<Vec<u8>>,
    boot_type: RefCell<u32>,
    boot_recover: RefCell<u32>,
    _io: RefCell<BytesReader>,
    unknown1_raw: RefCell<Vec<u8>>,
    unknown2_raw: RefCell<Vec<u8>>,
    reserved_raw: RefCell<Vec<u8>>,
}
impl KStruct for Regf_FileHeader {
    type Root = Regf;
    type Parent = Regf;

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
        *self_rc.signature.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.signature() == vec![0x72u8, 0x65u8, 0x67u8, 0x66u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/file_header/seq/0".to_string() }));
        }
        *self_rc.primary_sequence_number.borrow_mut() = _io.read_u4le()?;
        *self_rc.secondary_sequence_number.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, Regf_Filetime>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.last_modification_date_and_time.borrow_mut() = t;
        *self_rc.major_version.borrow_mut() = _io.read_u4le()?;
        *self_rc.minor_version.borrow_mut() = _io.read_u4le()?;
        *self_rc.r#type.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.format.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.root_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.hive_bins_data_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.clustering_factor.borrow_mut() = _io.read_u4le()?;
        *self_rc.unknown1.borrow_mut() = _io.read_bytes(64_usize)?;
        *self_rc.unknown2.borrow_mut() = _io.read_bytes(396_usize)?;
        *self_rc.checksum.borrow_mut() = _io.read_u4le()?;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(3576_usize)?;
        *self_rc.boot_type.borrow_mut() = _io.read_u4le()?;
        *self_rc.boot_recover.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_FileHeader {
}
impl Regf_FileHeader {
    pub fn signature(&self) -> Ref<'_, Vec<u8>> {
        self.signature.borrow()
    }
}
impl Regf_FileHeader {
    pub fn primary_sequence_number(&self) -> Ref<'_, u32> {
        self.primary_sequence_number.borrow()
    }
}
impl Regf_FileHeader {
    pub fn secondary_sequence_number(&self) -> Ref<'_, u32> {
        self.secondary_sequence_number.borrow()
    }
}
impl Regf_FileHeader {
    pub fn last_modification_date_and_time(&self) -> Ref<'_, OptRc<Regf_Filetime>> {
        self.last_modification_date_and_time.borrow()
    }
}
impl Regf_FileHeader {
    pub fn major_version(&self) -> Ref<'_, u32> {
        self.major_version.borrow()
    }
}
impl Regf_FileHeader {
    pub fn minor_version(&self) -> Ref<'_, u32> {
        self.minor_version.borrow()
    }
}
impl Regf_FileHeader {
    pub fn r#type(&self) -> Ref<'_, Regf_FileHeader_FileType> {
        self.r#type.borrow()
    }
}
impl Regf_FileHeader {
    pub fn format(&self) -> Ref<'_, Regf_FileHeader_FileFormat> {
        self.format.borrow()
    }
}
impl Regf_FileHeader {
    pub fn root_key_offset(&self) -> Ref<'_, u32> {
        self.root_key_offset.borrow()
    }
}
impl Regf_FileHeader {
    pub fn hive_bins_data_size(&self) -> Ref<'_, u32> {
        self.hive_bins_data_size.borrow()
    }
}
impl Regf_FileHeader {
    pub fn clustering_factor(&self) -> Ref<'_, u32> {
        self.clustering_factor.borrow()
    }
}
impl Regf_FileHeader {
    pub fn unknown1(&self) -> Ref<'_, Vec<u8>> {
        self.unknown1.borrow()
    }
}
impl Regf_FileHeader {
    pub fn unknown2(&self) -> Ref<'_, Vec<u8>> {
        self.unknown2.borrow()
    }
}
impl Regf_FileHeader {
    pub fn checksum(&self) -> Ref<'_, u32> {
        self.checksum.borrow()
    }
}
impl Regf_FileHeader {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl Regf_FileHeader {
    pub fn boot_type(&self) -> Ref<'_, u32> {
        self.boot_type.borrow()
    }
}
impl Regf_FileHeader {
    pub fn boot_recover(&self) -> Ref<'_, u32> {
        self.boot_recover.borrow()
    }
}
impl Regf_FileHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Regf_FileHeader {
    pub fn unknown1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.unknown1_raw.borrow()
    }
}
impl Regf_FileHeader {
    pub fn unknown2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.unknown2_raw.borrow()
    }
}
impl Regf_FileHeader {
    pub fn reserved_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Regf_FileHeader_FileFormat {
    DirectMemoryLoad,
    Unknown(i64),
}

impl TryFrom<i64> for Regf_FileHeader_FileFormat {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Regf_FileHeader_FileFormat> {
        match flag {
            1 => Ok(Regf_FileHeader_FileFormat::DirectMemoryLoad),
            _ => Ok(Regf_FileHeader_FileFormat::Unknown(flag)),
        }
    }
}

impl From<&Regf_FileHeader_FileFormat> for i64 {
    fn from(v: &Regf_FileHeader_FileFormat) -> Self {
        match *v {
            Regf_FileHeader_FileFormat::DirectMemoryLoad => 1,
            Regf_FileHeader_FileFormat::Unknown(v) => v
        }
    }
}

impl Default for Regf_FileHeader_FileFormat {
    fn default() -> Self { Regf_FileHeader_FileFormat::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Regf_FileHeader_FileType {
    Normal,
    TransactionLog,
    Unknown(i64),
}

impl TryFrom<i64> for Regf_FileHeader_FileType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Regf_FileHeader_FileType> {
        match flag {
            0 => Ok(Regf_FileHeader_FileType::Normal),
            1 => Ok(Regf_FileHeader_FileType::TransactionLog),
            _ => Ok(Regf_FileHeader_FileType::Unknown(flag)),
        }
    }
}

impl From<&Regf_FileHeader_FileType> for i64 {
    fn from(v: &Regf_FileHeader_FileType) -> Self {
        match *v {
            Regf_FileHeader_FileType::Normal => 0,
            Regf_FileHeader_FileType::TransactionLog => 1,
            Regf_FileHeader_FileType::Unknown(v) => v
        }
    }
}

impl Default for Regf_FileHeader_FileType {
    fn default() -> Self { Regf_FileHeader_FileType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Regf_Filetime {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<KStructUnit>,
    pub(crate) _self_shared: SharedType<Self>,
    value: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_Filetime {
    type Root = Regf;
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
        *self_rc.value.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_Filetime {
}
impl Regf_Filetime {
    pub fn value(&self) -> Ref<'_, u64> {
        self.value.borrow()
    }
}
impl Regf_Filetime {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBin {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Regf_HiveBinHeader>>,
    cells: RefCell<Vec<OptRc<Regf_HiveBinCell>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBin {
    type Root = Regf;
    type Parent = Regf;

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
        let t = Self::read_into::<_, Regf_HiveBinHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.cells.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Regf_HiveBinCell>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.cells.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBin {
}
impl Regf_HiveBin {
    pub fn header(&self) -> Ref<'_, OptRc<Regf_HiveBinHeader>> {
        self.header.borrow()
    }
}
impl Regf_HiveBin {
    pub fn cells(&self) -> Ref<'_, Vec<OptRc<Regf_HiveBinCell>>> {
        self.cells.borrow()
    }
}
impl Regf_HiveBin {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBin>,
    pub(crate) _self_shared: SharedType<Self>,
    cell_size_raw: RefCell<i32>,
    identifier: RefCell<String>,
    data: RefCell<Option<Regf_HiveBinCell_Data>>,
    _io: RefCell<BytesReader>,
    identifier_raw: RefCell<Vec<u8>>,
    data_raw: RefCell<Vec<u8>>,
    f_cell_size: Cell<bool>,
    cell_size: RefCell<i32>,
    f_is_allocated: Cell<bool>,
    is_allocated: RefCell<bool>,
}
#[derive(Debug, Clone)]
pub enum Regf_HiveBinCell_Data {
    Regf_HiveBinCell_SubKeyListLhLf(OptRc<Regf_HiveBinCell_SubKeyListLhLf>),
    Regf_HiveBinCell_SubKeyListLi(OptRc<Regf_HiveBinCell_SubKeyListLi>),
    Regf_HiveBinCell_NamedKey(OptRc<Regf_HiveBinCell_NamedKey>),
    Regf_HiveBinCell_SubKeyListRi(OptRc<Regf_HiveBinCell_SubKeyListRi>),
    Regf_HiveBinCell_SubKeyListSk(OptRc<Regf_HiveBinCell_SubKeyListSk>),
    Regf_HiveBinCell_SubKeyListVk(OptRc<Regf_HiveBinCell_SubKeyListVk>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_SubKeyListLhLf> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_SubKeyListLhLf(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_SubKeyListLhLf>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_SubKeyListLhLf>) -> Self {
        Self::Regf_HiveBinCell_SubKeyListLhLf(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_SubKeyListLi> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_SubKeyListLi(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_SubKeyListLi>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_SubKeyListLi>) -> Self {
        Self::Regf_HiveBinCell_SubKeyListLi(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_NamedKey> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_NamedKey(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_NamedKey>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_NamedKey>) -> Self {
        Self::Regf_HiveBinCell_NamedKey(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_SubKeyListRi> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_SubKeyListRi(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_SubKeyListRi>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_SubKeyListRi>) -> Self {
        Self::Regf_HiveBinCell_SubKeyListRi(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_SubKeyListSk> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_SubKeyListSk(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_SubKeyListSk>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_SubKeyListSk>) -> Self {
        Self::Regf_HiveBinCell_SubKeyListSk(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for OptRc<Regf_HiveBinCell_SubKeyListVk> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Regf_HiveBinCell_SubKeyListVk(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Regf_HiveBinCell_SubKeyListVk>> for Regf_HiveBinCell_Data {
    fn from(v: OptRc<Regf_HiveBinCell_SubKeyListVk>) -> Self {
        Self::Regf_HiveBinCell_SubKeyListVk(v)
    }
}
impl TryFrom<&Regf_HiveBinCell_Data> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Regf_HiveBinCell_Data) -> Result<Self, Self::Error> {
        if let Regf_HiveBinCell_Data::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Regf_HiveBinCell_Data {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Regf_HiveBinCell {
    type Root = Regf;
    type Parent = Regf_HiveBin;

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
        *self_rc.cell_size_raw.borrow_mut() = _io.read_s4le()?;
        *self_rc.identifier.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ascii")?;
        match self_rc.identifier().as_str() {
            "lf" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListLhLf>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "lh" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListLhLf>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "li" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListLi>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "nk" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_NamedKey>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "ri" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListRi>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "sk" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListSk>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            "vk" => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(((*self_rc.cell_size()?).saturating_sub(2_i32)).saturating_sub(4_i32))?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Regf_HiveBinCell_SubKeyListVk>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.data.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn cell_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_cell_size.get() {
            return Ok(self.cell_size.borrow());
        }
        self.f_cell_size.set(true);
        *self.cell_size.borrow_mut() = ((if ((to_i128(*self.cell_size_raw())) < (to_i128(0))) { (0_i32).saturating_sub(to_i32(1)) } else { 1_i32 }).saturating_mul(*self.cell_size_raw())).try_into()?;
        Ok(self.cell_size.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_allocated(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_allocated.get() {
            return Ok(self.is_allocated.borrow());
        }
        self.f_is_allocated.set(true);
        *self.is_allocated.borrow_mut() = (((to_i128(*self.cell_size_raw())) < (to_i128(0)))).try_into()?;
        Ok(self.is_allocated.borrow())
    }
}
impl Regf_HiveBinCell {
    pub fn cell_size_raw(&self) -> Ref<'_, i32> {
        self.cell_size_raw.borrow()
    }
}
impl Regf_HiveBinCell {
    pub fn identifier(&self) -> Ref<'_, String> {
        self.identifier.borrow()
    }
}
impl Regf_HiveBinCell {
    pub fn data(&self) -> Ref<'_, Option<Regf_HiveBinCell_Data>> {
        self.data.borrow()
    }
}
impl Regf_HiveBinCell {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Regf_HiveBinCell {
    pub fn identifier_raw(&self) -> Ref<'_, Vec<u8>> {
        self.identifier_raw.borrow()
    }
}
impl Regf_HiveBinCell {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_NamedKey {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    flags: RefCell<Regf_HiveBinCell_NamedKey_NkFlags>,
    last_key_written_date_and_time: RefCell<OptRc<Regf_Filetime>>,
    unknown1: RefCell<u32>,
    parent_key_offset: RefCell<u32>,
    number_of_sub_keys: RefCell<u32>,
    number_of_volatile_sub_keys: RefCell<u32>,
    sub_keys_list_offset: RefCell<u32>,
    number_of_values: RefCell<u32>,
    values_list_offset: RefCell<u32>,
    security_key_offset: RefCell<u32>,
    class_name_offset: RefCell<u32>,
    largest_sub_key_name_size: RefCell<u32>,
    largest_sub_key_class_name_size: RefCell<u32>,
    largest_value_name_size: RefCell<u32>,
    largest_value_data_size: RefCell<u32>,
    unknown2: RefCell<u32>,
    key_name_size: RefCell<u16>,
    class_name_size: RefCell<u16>,
    unknown_string_size: RefCell<u32>,
    unknown_string: RefCell<String>,
    _io: RefCell<BytesReader>,
    unknown_string_raw: RefCell<Vec<u8>>,
}
impl KStruct for Regf_HiveBinCell_NamedKey {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.flags.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        let t = Self::read_into::<_, Regf_Filetime>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.last_key_written_date_and_time.borrow_mut() = t;
        *self_rc.unknown1.borrow_mut() = _io.read_u4le()?;
        *self_rc.parent_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.number_of_sub_keys.borrow_mut() = _io.read_u4le()?;
        *self_rc.number_of_volatile_sub_keys.borrow_mut() = _io.read_u4le()?;
        *self_rc.sub_keys_list_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.number_of_values.borrow_mut() = _io.read_u4le()?;
        *self_rc.values_list_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.security_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.class_name_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.largest_sub_key_name_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.largest_sub_key_class_name_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.largest_value_name_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.largest_value_data_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.unknown2.borrow_mut() = _io.read_u4le()?;
        *self_rc.key_name_size.borrow_mut() = _io.read_u2le()?;
        *self_rc.class_name_size.borrow_mut() = _io.read_u2le()?;
        *self_rc.unknown_string_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.unknown_string.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.unknown_string_size())?)?, "ascii")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_NamedKey {
}
impl Regf_HiveBinCell_NamedKey {
    pub fn flags(&self) -> Ref<'_, Regf_HiveBinCell_NamedKey_NkFlags> {
        self.flags.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn last_key_written_date_and_time(&self) -> Ref<'_, OptRc<Regf_Filetime>> {
        self.last_key_written_date_and_time.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn unknown1(&self) -> Ref<'_, u32> {
        self.unknown1.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn parent_key_offset(&self) -> Ref<'_, u32> {
        self.parent_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn number_of_sub_keys(&self) -> Ref<'_, u32> {
        self.number_of_sub_keys.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn number_of_volatile_sub_keys(&self) -> Ref<'_, u32> {
        self.number_of_volatile_sub_keys.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn sub_keys_list_offset(&self) -> Ref<'_, u32> {
        self.sub_keys_list_offset.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn number_of_values(&self) -> Ref<'_, u32> {
        self.number_of_values.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn values_list_offset(&self) -> Ref<'_, u32> {
        self.values_list_offset.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn security_key_offset(&self) -> Ref<'_, u32> {
        self.security_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn class_name_offset(&self) -> Ref<'_, u32> {
        self.class_name_offset.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn largest_sub_key_name_size(&self) -> Ref<'_, u32> {
        self.largest_sub_key_name_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn largest_sub_key_class_name_size(&self) -> Ref<'_, u32> {
        self.largest_sub_key_class_name_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn largest_value_name_size(&self) -> Ref<'_, u32> {
        self.largest_value_name_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn largest_value_data_size(&self) -> Ref<'_, u32> {
        self.largest_value_data_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn unknown2(&self) -> Ref<'_, u32> {
        self.unknown2.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn key_name_size(&self) -> Ref<'_, u16> {
        self.key_name_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn class_name_size(&self) -> Ref<'_, u16> {
        self.class_name_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn unknown_string_size(&self) -> Ref<'_, u32> {
        self.unknown_string_size.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn unknown_string(&self) -> Ref<'_, String> {
        self.unknown_string.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Regf_HiveBinCell_NamedKey {
    pub fn unknown_string_raw(&self) -> Ref<'_, Vec<u8>> {
        self.unknown_string_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Regf_HiveBinCell_NamedKey_NkFlags {
    KeyIsVolatile,
    KeyHiveExit,
    KeyHiveEntry,
    KeyNoDelete,
    KeySymLink,
    KeyCompName,
    KeyPrefefHandle,
    KeyVirtMirrored,
    KeyVirtTarget,
    KeyVirtualStore,
    Unknown1,
    Unknown2,
    Unknown(i64),
}

impl TryFrom<i64> for Regf_HiveBinCell_NamedKey_NkFlags {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Regf_HiveBinCell_NamedKey_NkFlags> {
        match flag {
            1 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyIsVolatile),
            2 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyHiveExit),
            4 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyHiveEntry),
            8 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyNoDelete),
            16 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeySymLink),
            32 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyCompName),
            64 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyPrefefHandle),
            128 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtMirrored),
            256 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtTarget),
            512 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtualStore),
            4096 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::Unknown1),
            16384 => Ok(Regf_HiveBinCell_NamedKey_NkFlags::Unknown2),
            _ => Ok(Regf_HiveBinCell_NamedKey_NkFlags::Unknown(flag)),
        }
    }
}

impl From<&Regf_HiveBinCell_NamedKey_NkFlags> for i64 {
    fn from(v: &Regf_HiveBinCell_NamedKey_NkFlags) -> Self {
        match *v {
            Regf_HiveBinCell_NamedKey_NkFlags::KeyIsVolatile => 1,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyHiveExit => 2,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyHiveEntry => 4,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyNoDelete => 8,
            Regf_HiveBinCell_NamedKey_NkFlags::KeySymLink => 16,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyCompName => 32,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyPrefefHandle => 64,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtMirrored => 128,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtTarget => 256,
            Regf_HiveBinCell_NamedKey_NkFlags::KeyVirtualStore => 512,
            Regf_HiveBinCell_NamedKey_NkFlags::Unknown1 => 4096,
            Regf_HiveBinCell_NamedKey_NkFlags::Unknown2 => 16384,
            Regf_HiveBinCell_NamedKey_NkFlags::Unknown(v) => v
        }
    }
}

impl Default for Regf_HiveBinCell_NamedKey_NkFlags {
    fn default() -> Self { Regf_HiveBinCell_NamedKey_NkFlags::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListLhLf {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    count: RefCell<u16>,
    items: RefCell<Vec<OptRc<Regf_HiveBinCell_SubKeyListLhLf_Item>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListLhLf {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.count.borrow_mut() = _io.read_u2le()?;
        *self_rc.items.borrow_mut() = Vec::new();
        let l_items = usize::from(*self_rc.count());
        for _i in 0_usize..l_items {
            let t = Self::read_into::<_, Regf_HiveBinCell_SubKeyListLhLf_Item>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.items.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf {
}
impl Regf_HiveBinCell_SubKeyListLhLf {
    pub fn count(&self) -> Ref<'_, u16> {
        self.count.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf {
    pub fn items(&self) -> Ref<'_, Vec<OptRc<Regf_HiveBinCell_SubKeyListLhLf_Item>>> {
        self.items.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListLhLf_Item {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell_SubKeyListLhLf>,
    pub(crate) _self_shared: SharedType<Self>,
    named_key_offset: RefCell<u32>,
    hash_value: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListLhLf_Item {
    type Root = Regf;
    type Parent = Regf_HiveBinCell_SubKeyListLhLf;

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
        *self_rc.named_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.hash_value.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf_Item {
}
impl Regf_HiveBinCell_SubKeyListLhLf_Item {
    pub fn named_key_offset(&self) -> Ref<'_, u32> {
        self.named_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf_Item {
    pub fn hash_value(&self) -> Ref<'_, u32> {
        self.hash_value.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLhLf_Item {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListLi {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    count: RefCell<u16>,
    items: RefCell<Vec<OptRc<Regf_HiveBinCell_SubKeyListLi_Item>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListLi {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.count.borrow_mut() = _io.read_u2le()?;
        *self_rc.items.borrow_mut() = Vec::new();
        let l_items = usize::from(*self_rc.count());
        for _i in 0_usize..l_items {
            let t = Self::read_into::<_, Regf_HiveBinCell_SubKeyListLi_Item>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.items.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListLi {
}
impl Regf_HiveBinCell_SubKeyListLi {
    pub fn count(&self) -> Ref<'_, u16> {
        self.count.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLi {
    pub fn items(&self) -> Ref<'_, Vec<OptRc<Regf_HiveBinCell_SubKeyListLi_Item>>> {
        self.items.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLi {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListLi_Item {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell_SubKeyListLi>,
    pub(crate) _self_shared: SharedType<Self>,
    named_key_offset: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListLi_Item {
    type Root = Regf;
    type Parent = Regf_HiveBinCell_SubKeyListLi;

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
        *self_rc.named_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListLi_Item {
}
impl Regf_HiveBinCell_SubKeyListLi_Item {
    pub fn named_key_offset(&self) -> Ref<'_, u32> {
        self.named_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListLi_Item {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListRi {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    count: RefCell<u16>,
    items: RefCell<Vec<OptRc<Regf_HiveBinCell_SubKeyListRi_Item>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListRi {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.count.borrow_mut() = _io.read_u2le()?;
        *self_rc.items.borrow_mut() = Vec::new();
        let l_items = usize::from(*self_rc.count());
        for _i in 0_usize..l_items {
            let t = Self::read_into::<_, Regf_HiveBinCell_SubKeyListRi_Item>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.items.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListRi {
}
impl Regf_HiveBinCell_SubKeyListRi {
    pub fn count(&self) -> Ref<'_, u16> {
        self.count.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListRi {
    pub fn items(&self) -> Ref<'_, Vec<OptRc<Regf_HiveBinCell_SubKeyListRi_Item>>> {
        self.items.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListRi {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListRi_Item {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell_SubKeyListRi>,
    pub(crate) _self_shared: SharedType<Self>,
    sub_key_list_offset: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListRi_Item {
    type Root = Regf;
    type Parent = Regf_HiveBinCell_SubKeyListRi;

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
        *self_rc.sub_key_list_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListRi_Item {
}
impl Regf_HiveBinCell_SubKeyListRi_Item {
    pub fn sub_key_list_offset(&self) -> Ref<'_, u32> {
        self.sub_key_list_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListRi_Item {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListSk {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    unknown1: RefCell<u16>,
    previous_security_key_offset: RefCell<u32>,
    next_security_key_offset: RefCell<u32>,
    reference_count: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListSk {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.unknown1.borrow_mut() = _io.read_u2le()?;
        *self_rc.previous_security_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.next_security_key_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.reference_count.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListSk {
}
impl Regf_HiveBinCell_SubKeyListSk {
    pub fn unknown1(&self) -> Ref<'_, u16> {
        self.unknown1.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListSk {
    pub fn previous_security_key_offset(&self) -> Ref<'_, u32> {
        self.previous_security_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListSk {
    pub fn next_security_key_offset(&self) -> Ref<'_, u32> {
        self.next_security_key_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListSk {
    pub fn reference_count(&self) -> Ref<'_, u32> {
        self.reference_count.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListSk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinCell_SubKeyListVk {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBinCell>,
    pub(crate) _self_shared: SharedType<Self>,
    value_name_size: RefCell<u16>,
    data_size: RefCell<u32>,
    data_offset: RefCell<u32>,
    data_type: RefCell<Regf_HiveBinCell_SubKeyListVk_DataTypeEnum>,
    flags: RefCell<Regf_HiveBinCell_SubKeyListVk_VkFlags>,
    padding: RefCell<u16>,
    value_name: RefCell<String>,
    _io: RefCell<BytesReader>,
    value_name_raw: RefCell<Vec<u8>>,
}
impl KStruct for Regf_HiveBinCell_SubKeyListVk {
    type Root = Regf;
    type Parent = Regf_HiveBinCell;

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
        *self_rc.value_name_size.borrow_mut() = _io.read_u2le()?;
        *self_rc.data_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.data_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.data_type.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.flags.borrow_mut() = i64::from(_io.read_u2le()?).try_into()?;
        *self_rc.padding.borrow_mut() = _io.read_u2le()?;
        if *self_rc.flags() == Regf_HiveBinCell_SubKeyListVk_VkFlags::ValueCompName {
            *self_rc.value_name.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.value_name_size()))?, "ascii")?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn value_name_size(&self) -> Ref<'_, u16> {
        self.value_name_size.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn data_size(&self) -> Ref<'_, u32> {
        self.data_size.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn data_offset(&self) -> Ref<'_, u32> {
        self.data_offset.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn data_type(&self) -> Ref<'_, Regf_HiveBinCell_SubKeyListVk_DataTypeEnum> {
        self.data_type.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn flags(&self) -> Ref<'_, Regf_HiveBinCell_SubKeyListVk_VkFlags> {
        self.flags.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn padding(&self) -> Ref<'_, u16> {
        self.padding.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn value_name(&self) -> Ref<'_, String> {
        self.value_name.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Regf_HiveBinCell_SubKeyListVk {
    pub fn value_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.value_name_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Regf_HiveBinCell_SubKeyListVk_DataTypeEnum {
    RegNone,
    RegSz,
    RegExpandSz,
    RegBinary,
    RegDword,
    RegDwordBigEndian,
    RegLink,
    RegMultiSz,
    RegResourceList,
    RegFullResourceDescriptor,
    RegResourceRequirementsList,
    RegQword,
    Unknown(i64),
}

impl TryFrom<i64> for Regf_HiveBinCell_SubKeyListVk_DataTypeEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Regf_HiveBinCell_SubKeyListVk_DataTypeEnum> {
        match flag {
            0 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegNone),
            1 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegSz),
            2 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegExpandSz),
            3 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegBinary),
            4 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegDword),
            5 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegDwordBigEndian),
            6 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegLink),
            7 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegMultiSz),
            8 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegResourceList),
            9 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegFullResourceDescriptor),
            10 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegResourceRequirementsList),
            11 => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegQword),
            _ => Ok(Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::Unknown(flag)),
        }
    }
}

impl From<&Regf_HiveBinCell_SubKeyListVk_DataTypeEnum> for i64 {
    fn from(v: &Regf_HiveBinCell_SubKeyListVk_DataTypeEnum) -> Self {
        match *v {
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegNone => 0,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegSz => 1,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegExpandSz => 2,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegBinary => 3,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegDword => 4,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegDwordBigEndian => 5,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegLink => 6,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegMultiSz => 7,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegResourceList => 8,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegFullResourceDescriptor => 9,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegResourceRequirementsList => 10,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::RegQword => 11,
            Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::Unknown(v) => v
        }
    }
}

impl Default for Regf_HiveBinCell_SubKeyListVk_DataTypeEnum {
    fn default() -> Self { Regf_HiveBinCell_SubKeyListVk_DataTypeEnum::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Regf_HiveBinCell_SubKeyListVk_VkFlags {
    ValueCompName,
    Unknown(i64),
}

impl TryFrom<i64> for Regf_HiveBinCell_SubKeyListVk_VkFlags {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Regf_HiveBinCell_SubKeyListVk_VkFlags> {
        match flag {
            1 => Ok(Regf_HiveBinCell_SubKeyListVk_VkFlags::ValueCompName),
            _ => Ok(Regf_HiveBinCell_SubKeyListVk_VkFlags::Unknown(flag)),
        }
    }
}

impl From<&Regf_HiveBinCell_SubKeyListVk_VkFlags> for i64 {
    fn from(v: &Regf_HiveBinCell_SubKeyListVk_VkFlags) -> Self {
        match *v {
            Regf_HiveBinCell_SubKeyListVk_VkFlags::ValueCompName => 1,
            Regf_HiveBinCell_SubKeyListVk_VkFlags::Unknown(v) => v
        }
    }
}

impl Default for Regf_HiveBinCell_SubKeyListVk_VkFlags {
    fn default() -> Self { Regf_HiveBinCell_SubKeyListVk_VkFlags::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Regf_HiveBinHeader {
    pub(crate) _root: SharedType<Regf>,
    pub(crate) _parent: SharedType<Regf_HiveBin>,
    pub(crate) _self_shared: SharedType<Self>,
    signature: RefCell<Vec<u8>>,
    offset: RefCell<u32>,
    size: RefCell<u32>,
    unknown1: RefCell<u32>,
    unknown2: RefCell<u32>,
    timestamp: RefCell<OptRc<Regf_Filetime>>,
    unknown4: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Regf_HiveBinHeader {
    type Root = Regf;
    type Parent = Regf_HiveBin;

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
        *self_rc.signature.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.signature() == vec![0x68u8, 0x62u8, 0x69u8, 0x6eu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/hive_bin_header/seq/0".to_string() }));
        }
        *self_rc.offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.unknown1.borrow_mut() = _io.read_u4le()?;
        *self_rc.unknown2.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, Regf_Filetime>(&*_io, Some(self_rc._root.clone()), None)?.into();
        *self_rc.timestamp.borrow_mut() = t;
        *self_rc.unknown4.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Regf_HiveBinHeader {
}
impl Regf_HiveBinHeader {
    pub fn signature(&self) -> Ref<'_, Vec<u8>> {
        self.signature.borrow()
    }
}

/**
 * The offset of the hive bin, Value in bytes and relative from
 * the start of the hive bin data
 */
impl Regf_HiveBinHeader {
    pub fn offset(&self) -> Ref<'_, u32> {
        self.offset.borrow()
    }
}

/**
 * Size of the hive bin
 */
impl Regf_HiveBinHeader {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}

/**
 * 0 most of the time, can contain remnant data
 */
impl Regf_HiveBinHeader {
    pub fn unknown1(&self) -> Ref<'_, u32> {
        self.unknown1.borrow()
    }
}

/**
 * 0 most of the time, can contain remnant data
 */
impl Regf_HiveBinHeader {
    pub fn unknown2(&self) -> Ref<'_, u32> {
        self.unknown2.borrow()
    }
}

/**
 * Only the root (first) hive bin seems to contain a valid FILETIME
 */
impl Regf_HiveBinHeader {
    pub fn timestamp(&self) -> Ref<'_, OptRc<Regf_Filetime>> {
        self.timestamp.borrow()
    }
}

/**
 * Contains number of bytes
 */
impl Regf_HiveBinHeader {
    pub fn unknown4(&self) -> Ref<'_, u32> {
        self.unknown4.borrow()
    }
}
impl Regf_HiveBinHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
