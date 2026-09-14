// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * The metadata stored by Android at the beginning of a "super" partition, which
 * is what it calls a disk partition that holds one or more Dynamic Partitions.
 * Dynamic Partitions do more or less the same thing that LVM does on Linux,
 * allowing Android to map ranges of non-contiguous extents to a single logical
 * device. This metadata holds that mapping.
 * \sa <https://source.android.com/docs/core/ota/dynamic_partitions> Source
 * \sa <https://android.googlesource.com/platform/system/core/+/refs/tags/android-11.0.0_r8/fs_mgr/liblp/include/liblp/metadata_format.h> Source
 */

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_root: Cell<bool>,
    root: RefCell<OptRc<AndroidSuper_Root>>,
}
impl KStruct for AndroidSuper {
    type Root = AndroidSuper;
    type Parent = AndroidSuper;

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
impl AndroidSuper {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn root(
        &self
    ) -> KResult<Ref<'_, OptRc<AndroidSuper_Root>>> {
        let _io = self._io.borrow();
        if self.f_root.get() {
            return Ok(self.root.borrow());
        }
        let _pos = _io.pos();
        _io.seek(4096_usize)?;
        let t = Self::read_into::<_, AndroidSuper_Root>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.root.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.root.borrow())
    }
}
impl AndroidSuper {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Geometry {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Root>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    struct_size: RefCell<u32>,
    checksum: RefCell<Vec<u8>>,
    metadata_max_size: RefCell<u32>,
    metadata_slot_count: RefCell<u32>,
    logical_block_size: RefCell<u32>,
    _io: RefCell<BytesReader>,
    checksum_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Geometry {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Root;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic() == vec![0x67u8, 0x44u8, 0x6cu8, 0x61u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/geometry/seq/0".to_string() }));
        }
        *self_rc.struct_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.checksum.borrow_mut() = _io.read_bytes(32_usize)?;
        *self_rc.metadata_max_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.metadata_slot_count.borrow_mut() = _io.read_u4le()?;
        *self_rc.logical_block_size.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Geometry {
}
impl AndroidSuper_Geometry {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn struct_size(&self) -> Ref<'_, u32> {
        self.struct_size.borrow()
    }
}

/**
 * SHA-256 hash of struct_size bytes from beginning of geometry,
 * calculated as if checksum were zeroed out
 */
impl AndroidSuper_Geometry {
    pub fn checksum(&self) -> Ref<'_, Vec<u8>> {
        self.checksum.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn metadata_max_size(&self) -> Ref<'_, u32> {
        self.metadata_max_size.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn metadata_slot_count(&self) -> Ref<'_, u32> {
        self.metadata_slot_count.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn logical_block_size(&self) -> Ref<'_, u32> {
        self.logical_block_size.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Geometry {
    pub fn checksum_raw(&self) -> Ref<'_, Vec<u8>> {
        self.checksum_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Root>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    major_version: RefCell<u16>,
    minor_version: RefCell<u16>,
    header_size: RefCell<u32>,
    header_checksum: RefCell<Vec<u8>>,
    tables_size: RefCell<u32>,
    tables_checksum: RefCell<Vec<u8>>,
    partitions: RefCell<OptRc<AndroidSuper_Metadata_TableDescriptor>>,
    extents: RefCell<OptRc<AndroidSuper_Metadata_TableDescriptor>>,
    groups: RefCell<OptRc<AndroidSuper_Metadata_TableDescriptor>>,
    block_devices: RefCell<OptRc<AndroidSuper_Metadata_TableDescriptor>>,
    _io: RefCell<BytesReader>,
    header_checksum_raw: RefCell<Vec<u8>>,
    tables_checksum_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Metadata {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Root;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic() == vec![0x30u8, 0x50u8, 0x4cu8, 0x41u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/metadata/seq/0".to_string() }));
        }
        *self_rc.major_version.borrow_mut() = _io.read_u2le()?;
        *self_rc.minor_version.borrow_mut() = _io.read_u2le()?;
        *self_rc.header_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.header_checksum.borrow_mut() = _io.read_bytes(32_usize)?;
        *self_rc.tables_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.tables_checksum.borrow_mut() = _io.read_bytes(32_usize)?;
        let f = |t : &mut AndroidSuper_Metadata_TableDescriptor| Ok(t.set_params(AndroidSuper_Metadata_TableKind::Partitions));
        let t = Self::read_into_with_init::<_, AndroidSuper_Metadata_TableDescriptor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.partitions.borrow_mut() = t;
        let f = |t : &mut AndroidSuper_Metadata_TableDescriptor| Ok(t.set_params(AndroidSuper_Metadata_TableKind::Extents));
        let t = Self::read_into_with_init::<_, AndroidSuper_Metadata_TableDescriptor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.extents.borrow_mut() = t;
        let f = |t : &mut AndroidSuper_Metadata_TableDescriptor| Ok(t.set_params(AndroidSuper_Metadata_TableKind::Groups));
        let t = Self::read_into_with_init::<_, AndroidSuper_Metadata_TableDescriptor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.groups.borrow_mut() = t;
        let f = |t : &mut AndroidSuper_Metadata_TableDescriptor| Ok(t.set_params(AndroidSuper_Metadata_TableKind::BlockDevices));
        let t = Self::read_into_with_init::<_, AndroidSuper_Metadata_TableDescriptor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.block_devices.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata {
}
impl AndroidSuper_Metadata {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn major_version(&self) -> Ref<'_, u16> {
        self.major_version.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn minor_version(&self) -> Ref<'_, u16> {
        self.minor_version.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn header_size(&self) -> Ref<'_, u32> {
        self.header_size.borrow()
    }
}

/**
 * SHA-256 hash of header_size bytes from beginning of metadata,
 * calculated as if header_checksum were zeroed out
 */
impl AndroidSuper_Metadata {
    pub fn header_checksum(&self) -> Ref<'_, Vec<u8>> {
        self.header_checksum.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn tables_size(&self) -> Ref<'_, u32> {
        self.tables_size.borrow()
    }
}

/**
 * SHA-256 hash of tables_size bytes from end of header
 */
impl AndroidSuper_Metadata {
    pub fn tables_checksum(&self) -> Ref<'_, Vec<u8>> {
        self.tables_checksum.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn partitions(&self) -> Ref<'_, OptRc<AndroidSuper_Metadata_TableDescriptor>> {
        self.partitions.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn extents(&self) -> Ref<'_, OptRc<AndroidSuper_Metadata_TableDescriptor>> {
        self.extents.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn groups(&self) -> Ref<'_, OptRc<AndroidSuper_Metadata_TableDescriptor>> {
        self.groups.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn block_devices(&self) -> Ref<'_, OptRc<AndroidSuper_Metadata_TableDescriptor>> {
        self.block_devices.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn header_checksum_raw(&self) -> Ref<'_, Vec<u8>> {
        self.header_checksum_raw.borrow()
    }
}
impl AndroidSuper_Metadata {
    pub fn tables_checksum_raw(&self) -> Ref<'_, Vec<u8>> {
        self.tables_checksum_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AndroidSuper_Metadata_TableKind {
    Partitions,
    Extents,
    Groups,
    BlockDevices,
    Unknown(i64),
}

impl TryFrom<i64> for AndroidSuper_Metadata_TableKind {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<AndroidSuper_Metadata_TableKind> {
        match flag {
            0 => Ok(AndroidSuper_Metadata_TableKind::Partitions),
            1 => Ok(AndroidSuper_Metadata_TableKind::Extents),
            2 => Ok(AndroidSuper_Metadata_TableKind::Groups),
            3 => Ok(AndroidSuper_Metadata_TableKind::BlockDevices),
            _ => Ok(AndroidSuper_Metadata_TableKind::Unknown(flag)),
        }
    }
}

impl From<&AndroidSuper_Metadata_TableKind> for i64 {
    fn from(v: &AndroidSuper_Metadata_TableKind) -> Self {
        match *v {
            AndroidSuper_Metadata_TableKind::Partitions => 0,
            AndroidSuper_Metadata_TableKind::Extents => 1,
            AndroidSuper_Metadata_TableKind::Groups => 2,
            AndroidSuper_Metadata_TableKind::BlockDevices => 3,
            AndroidSuper_Metadata_TableKind::Unknown(v) => v
        }
    }
}

impl Default for AndroidSuper_Metadata_TableKind {
    fn default() -> Self { AndroidSuper_Metadata_TableKind::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata_BlockDevice {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Metadata_TableDescriptor>,
    pub(crate) _self_shared: SharedType<Self>,
    first_logical_sector: RefCell<u64>,
    alignment: RefCell<u32>,
    alignment_offset: RefCell<u32>,
    size: RefCell<u64>,
    partition_name: RefCell<String>,
    flag_slot_suffixed: RefCell<bool>,
    flags_reserved: RefCell<u64>,
    _io: RefCell<BytesReader>,
    partition_name_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Metadata_BlockDevice {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Metadata_TableDescriptor;

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
        *self_rc.first_logical_sector.borrow_mut() = _io.read_u8le()?;
        *self_rc.alignment.borrow_mut() = _io.read_u4le()?;
        *self_rc.alignment_offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.size.borrow_mut() = _io.read_u8le()?;
        *self_rc.partition_name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(36_usize)?, Some(0), false, None), "UTF-8")?;
        *self_rc.flag_slot_suffixed.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.flags_reserved.borrow_mut() = _io.read_bits_int_le(31)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata_BlockDevice {
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn first_logical_sector(&self) -> Ref<'_, u64> {
        self.first_logical_sector.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn alignment(&self) -> Ref<'_, u32> {
        self.alignment.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn alignment_offset(&self) -> Ref<'_, u32> {
        self.alignment_offset.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn size(&self) -> Ref<'_, u64> {
        self.size.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn partition_name(&self) -> Ref<'_, String> {
        self.partition_name.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn flag_slot_suffixed(&self) -> Ref<'_, bool> {
        self.flag_slot_suffixed.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn flags_reserved(&self) -> Ref<'_, u64> {
        self.flags_reserved.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Metadata_BlockDevice {
    pub fn partition_name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.partition_name_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata_Extent {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Metadata_TableDescriptor>,
    pub(crate) _self_shared: SharedType<Self>,
    num_sectors: RefCell<u64>,
    target_type: RefCell<AndroidSuper_Metadata_Extent_TargetType>,
    target_data: RefCell<u64>,
    target_source: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidSuper_Metadata_Extent {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Metadata_TableDescriptor;

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
        *self_rc.num_sectors.borrow_mut() = _io.read_u8le()?;
        *self_rc.target_type.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        *self_rc.target_data.borrow_mut() = _io.read_u8le()?;
        *self_rc.target_source.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata_Extent {
}
impl AndroidSuper_Metadata_Extent {
    pub fn num_sectors(&self) -> Ref<'_, u64> {
        self.num_sectors.borrow()
    }
}
impl AndroidSuper_Metadata_Extent {
    pub fn target_type(&self) -> Ref<'_, AndroidSuper_Metadata_Extent_TargetType> {
        self.target_type.borrow()
    }
}
impl AndroidSuper_Metadata_Extent {
    pub fn target_data(&self) -> Ref<'_, u64> {
        self.target_data.borrow()
    }
}
impl AndroidSuper_Metadata_Extent {
    pub fn target_source(&self) -> Ref<'_, u32> {
        self.target_source.borrow()
    }
}
impl AndroidSuper_Metadata_Extent {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AndroidSuper_Metadata_Extent_TargetType {
    Linear,
    Zero,
    Unknown(i64),
}

impl TryFrom<i64> for AndroidSuper_Metadata_Extent_TargetType {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<AndroidSuper_Metadata_Extent_TargetType> {
        match flag {
            0 => Ok(AndroidSuper_Metadata_Extent_TargetType::Linear),
            1 => Ok(AndroidSuper_Metadata_Extent_TargetType::Zero),
            _ => Ok(AndroidSuper_Metadata_Extent_TargetType::Unknown(flag)),
        }
    }
}

impl From<&AndroidSuper_Metadata_Extent_TargetType> for i64 {
    fn from(v: &AndroidSuper_Metadata_Extent_TargetType) -> Self {
        match *v {
            AndroidSuper_Metadata_Extent_TargetType::Linear => 0,
            AndroidSuper_Metadata_Extent_TargetType::Zero => 1,
            AndroidSuper_Metadata_Extent_TargetType::Unknown(v) => v
        }
    }
}

impl Default for AndroidSuper_Metadata_Extent_TargetType {
    fn default() -> Self { AndroidSuper_Metadata_Extent_TargetType::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata_Group {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Metadata_TableDescriptor>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    flag_slot_suffixed: RefCell<bool>,
    flags_reserved: RefCell<u64>,
    maximum_size: RefCell<u64>,
    _io: RefCell<BytesReader>,
    name_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Metadata_Group {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Metadata_TableDescriptor;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(36_usize)?, Some(0), false, None), "UTF-8")?;
        *self_rc.flag_slot_suffixed.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.flags_reserved.borrow_mut() = _io.read_bits_int_le(31)?;
        io.align_to_byte()?;
        *self_rc.maximum_size.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata_Group {
}
impl AndroidSuper_Metadata_Group {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl AndroidSuper_Metadata_Group {
    pub fn flag_slot_suffixed(&self) -> Ref<'_, bool> {
        self.flag_slot_suffixed.borrow()
    }
}
impl AndroidSuper_Metadata_Group {
    pub fn flags_reserved(&self) -> Ref<'_, u64> {
        self.flags_reserved.borrow()
    }
}
impl AndroidSuper_Metadata_Group {
    pub fn maximum_size(&self) -> Ref<'_, u64> {
        self.maximum_size.borrow()
    }
}
impl AndroidSuper_Metadata_Group {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Metadata_Group {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata_Partition {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Metadata_TableDescriptor>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    attr_readonly: RefCell<bool>,
    attr_slot_suffixed: RefCell<bool>,
    attr_updated: RefCell<bool>,
    attr_disabled: RefCell<bool>,
    attrs_reserved: RefCell<u64>,
    first_extent_index: RefCell<u32>,
    num_extents: RefCell<u32>,
    group_index: RefCell<u32>,
    _io: RefCell<BytesReader>,
    name_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Metadata_Partition {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Metadata_TableDescriptor;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(36_usize)?, Some(0), false, None), "UTF-8")?;
        *self_rc.attr_readonly.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.attr_slot_suffixed.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.attr_updated.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.attr_disabled.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.attrs_reserved.borrow_mut() = _io.read_bits_int_le(28)?;
        io.align_to_byte()?;
        *self_rc.first_extent_index.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_extents.borrow_mut() = _io.read_u4le()?;
        *self_rc.group_index.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata_Partition {
}
impl AndroidSuper_Metadata_Partition {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn attr_readonly(&self) -> Ref<'_, bool> {
        self.attr_readonly.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn attr_slot_suffixed(&self) -> Ref<'_, bool> {
        self.attr_slot_suffixed.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn attr_updated(&self) -> Ref<'_, bool> {
        self.attr_updated.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn attr_disabled(&self) -> Ref<'_, bool> {
        self.attr_disabled.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn attrs_reserved(&self) -> Ref<'_, u64> {
        self.attrs_reserved.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn first_extent_index(&self) -> Ref<'_, u32> {
        self.first_extent_index.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn num_extents(&self) -> Ref<'_, u32> {
        self.num_extents.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn group_index(&self) -> Ref<'_, u32> {
        self.group_index.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Metadata_Partition {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Metadata_TableDescriptor {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper_Metadata>,
    pub(crate) _self_shared: SharedType<Self>,
    kind: RefCell<AndroidSuper_Metadata_TableKind>,
    offset: RefCell<u32>,
    num_entries: RefCell<u32>,
    entry_size: RefCell<u32>,
    _io: RefCell<BytesReader>,
    table_raw: RefCell<Vec<Vec<u8>>>,
    f_table: Cell<bool>,
    table: RefCell<Vec<AndroidSuper_Metadata_TableDescriptor_Table>>,
}
#[derive(Debug, Clone)]
pub enum AndroidSuper_Metadata_TableDescriptor_Table {
    AndroidSuper_Metadata_BlockDevice(OptRc<AndroidSuper_Metadata_BlockDevice>),
    AndroidSuper_Metadata_Extent(OptRc<AndroidSuper_Metadata_Extent>),
    AndroidSuper_Metadata_Group(OptRc<AndroidSuper_Metadata_Group>),
    AndroidSuper_Metadata_Partition(OptRc<AndroidSuper_Metadata_Partition>),
    Bytes(Vec<u8>),
}
impl TryFrom<&AndroidSuper_Metadata_TableDescriptor_Table> for OptRc<AndroidSuper_Metadata_BlockDevice> {
    type Error = KError;
    fn try_from(v: &AndroidSuper_Metadata_TableDescriptor_Table) -> Result<Self, Self::Error> {
        if let AndroidSuper_Metadata_TableDescriptor_Table::AndroidSuper_Metadata_BlockDevice(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<AndroidSuper_Metadata_BlockDevice>> for AndroidSuper_Metadata_TableDescriptor_Table {
    fn from(v: OptRc<AndroidSuper_Metadata_BlockDevice>) -> Self {
        Self::AndroidSuper_Metadata_BlockDevice(v)
    }
}
impl TryFrom<&AndroidSuper_Metadata_TableDescriptor_Table> for OptRc<AndroidSuper_Metadata_Extent> {
    type Error = KError;
    fn try_from(v: &AndroidSuper_Metadata_TableDescriptor_Table) -> Result<Self, Self::Error> {
        if let AndroidSuper_Metadata_TableDescriptor_Table::AndroidSuper_Metadata_Extent(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<AndroidSuper_Metadata_Extent>> for AndroidSuper_Metadata_TableDescriptor_Table {
    fn from(v: OptRc<AndroidSuper_Metadata_Extent>) -> Self {
        Self::AndroidSuper_Metadata_Extent(v)
    }
}
impl TryFrom<&AndroidSuper_Metadata_TableDescriptor_Table> for OptRc<AndroidSuper_Metadata_Group> {
    type Error = KError;
    fn try_from(v: &AndroidSuper_Metadata_TableDescriptor_Table) -> Result<Self, Self::Error> {
        if let AndroidSuper_Metadata_TableDescriptor_Table::AndroidSuper_Metadata_Group(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<AndroidSuper_Metadata_Group>> for AndroidSuper_Metadata_TableDescriptor_Table {
    fn from(v: OptRc<AndroidSuper_Metadata_Group>) -> Self {
        Self::AndroidSuper_Metadata_Group(v)
    }
}
impl TryFrom<&AndroidSuper_Metadata_TableDescriptor_Table> for OptRc<AndroidSuper_Metadata_Partition> {
    type Error = KError;
    fn try_from(v: &AndroidSuper_Metadata_TableDescriptor_Table) -> Result<Self, Self::Error> {
        if let AndroidSuper_Metadata_TableDescriptor_Table::AndroidSuper_Metadata_Partition(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<AndroidSuper_Metadata_Partition>> for AndroidSuper_Metadata_TableDescriptor_Table {
    fn from(v: OptRc<AndroidSuper_Metadata_Partition>) -> Self {
        Self::AndroidSuper_Metadata_Partition(v)
    }
}
impl TryFrom<&AndroidSuper_Metadata_TableDescriptor_Table> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &AndroidSuper_Metadata_TableDescriptor_Table) -> Result<Self, Self::Error> {
        if let AndroidSuper_Metadata_TableDescriptor_Table::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for AndroidSuper_Metadata_TableDescriptor_Table {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for AndroidSuper_Metadata_TableDescriptor {
    type Root = AndroidSuper;
    type Parent = AndroidSuper_Metadata;

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
        *self_rc.offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_entries.borrow_mut() = _io.read_u4le()?;
        *self_rc.entry_size.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn kind(&self) -> Ref<'_, AndroidSuper_Metadata_TableKind> {
        self.kind.borrow()
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn set_params(&mut self, kind: AndroidSuper_Metadata_TableKind) {
        *self.kind.borrow_mut() = kind;
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn table(
        &self
    ) -> KResult<Ref<'_, Vec<AndroidSuper_Metadata_TableDescriptor_Table>>> {
        let _io = self._io.borrow();
        if self.f_table.get() {
            return Ok(self.table.borrow());
        }
        self.f_table.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((*self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.header_size()).saturating_add(*self.offset()))?)?;
        *self.table_raw.borrow_mut() = Vec::new();
        *self.table.borrow_mut() = Vec::new();
        let l_table = usize::try_from(*self.num_entries())?;
        for _i in 0_usize..l_table {
            self.table_raw.borrow_mut().push(_io.read_bytes(usize::try_from(*self.entry_size())?)?.into());
            let table_raw = self.table_raw.borrow();
            let _io_table_raw = BytesReader::from(table_raw.last().ok_or(KError::EmptyIterator)?.clone());
            match *self.kind() {
                AndroidSuper_Metadata_TableKind::BlockDevices => {
                    let t = Self::read_into::<BytesReader, AndroidSuper_Metadata_BlockDevice>(&_io_table_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                    self.table.borrow_mut().push(t);
                }
                AndroidSuper_Metadata_TableKind::Extents => {
                    let t = Self::read_into::<BytesReader, AndroidSuper_Metadata_Extent>(&_io_table_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                    self.table.borrow_mut().push(t);
                }
                AndroidSuper_Metadata_TableKind::Groups => {
                    let t = Self::read_into::<BytesReader, AndroidSuper_Metadata_Group>(&_io_table_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                    self.table.borrow_mut().push(t);
                }
                AndroidSuper_Metadata_TableKind::Partitions => {
                    let t = Self::read_into::<BytesReader, AndroidSuper_Metadata_Partition>(&_io_table_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                    self.table.borrow_mut().push(t);
                }
                _ => {
                    self.table.borrow_mut().push(_io_table_raw.read_bytes_full()?.into());
                }
            }
        }
        _io.seek(_pos)?;
        Ok(self.table.borrow())
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn offset(&self) -> Ref<'_, u32> {
        self.offset.borrow()
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn num_entries(&self) -> Ref<'_, u32> {
        self.num_entries.borrow()
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn entry_size(&self) -> Ref<'_, u32> {
        self.entry_size.borrow()
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Metadata_TableDescriptor {
    pub fn table_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.table_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidSuper_Root {
    pub(crate) _root: SharedType<AndroidSuper>,
    pub(crate) _parent: SharedType<AndroidSuper>,
    pub(crate) _self_shared: SharedType<Self>,
    primary_geometry: RefCell<OptRc<AndroidSuper_Geometry>>,
    backup_geometry: RefCell<OptRc<AndroidSuper_Geometry>>,
    primary_metadata: RefCell<Vec<OptRc<AndroidSuper_Metadata>>>,
    backup_metadata: RefCell<Vec<OptRc<AndroidSuper_Metadata>>>,
    _io: RefCell<BytesReader>,
    primary_geometry_raw: RefCell<Vec<u8>>,
    backup_geometry_raw: RefCell<Vec<u8>>,
    primary_metadata_raw: RefCell<Vec<u8>>,
    backup_metadata_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidSuper_Root {
    type Root = AndroidSuper;
    type Parent = AndroidSuper;

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
        let _raw_primary_geometry = _io.read_bytes(4096_usize)?;
        *self_rc.primary_geometry_raw.borrow_mut() = _raw_primary_geometry.clone();
        let _io_primary_geometry = BytesReader::from(_raw_primary_geometry);
        let t = Self::read_into::<BytesReader, AndroidSuper_Geometry>(&_io_primary_geometry, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.primary_geometry.borrow_mut() = t;
        let _raw_backup_geometry = _io.read_bytes(4096_usize)?;
        *self_rc.backup_geometry_raw.borrow_mut() = _raw_backup_geometry.clone();
        let _io_backup_geometry = BytesReader::from(_raw_backup_geometry);
        let t = Self::read_into::<BytesReader, AndroidSuper_Geometry>(&_io_backup_geometry, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.backup_geometry.borrow_mut() = t;
        *self_rc.primary_metadata.borrow_mut() = Vec::new();
        let l_primary_metadata = usize::try_from(*self_rc.primary_geometry().metadata_slot_count())?;
        for _i in 0_usize..l_primary_metadata {
            let _raw_primary_metadata = _io.read_bytes(usize::try_from(*self_rc.primary_geometry().metadata_max_size())?)?;
            let _io_primary_metadata = BytesReader::from(_raw_primary_metadata);
            let t = Self::read_into::<BytesReader, AndroidSuper_Metadata>(&_io_primary_metadata, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.primary_metadata.borrow_mut().push(t);
        }
        *self_rc.backup_metadata.borrow_mut() = Vec::new();
        let l_backup_metadata = usize::try_from(*self_rc.primary_geometry().metadata_slot_count())?;
        for _i in 0_usize..l_backup_metadata {
            let _raw_backup_metadata = _io.read_bytes(usize::try_from(*self_rc.primary_geometry().metadata_max_size())?)?;
            let _io_backup_metadata = BytesReader::from(_raw_backup_metadata);
            let t = Self::read_into::<BytesReader, AndroidSuper_Metadata>(&_io_backup_metadata, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.backup_metadata.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidSuper_Root {
}
impl AndroidSuper_Root {
    pub fn primary_geometry(&self) -> Ref<'_, OptRc<AndroidSuper_Geometry>> {
        self.primary_geometry.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn backup_geometry(&self) -> Ref<'_, OptRc<AndroidSuper_Geometry>> {
        self.backup_geometry.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn primary_metadata(&self) -> Ref<'_, Vec<OptRc<AndroidSuper_Metadata>>> {
        self.primary_metadata.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn backup_metadata(&self) -> Ref<'_, Vec<OptRc<AndroidSuper_Metadata>>> {
        self.backup_metadata.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn primary_geometry_raw(&self) -> Ref<'_, Vec<u8>> {
        self.primary_geometry_raw.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn backup_geometry_raw(&self) -> Ref<'_, Vec<u8>> {
        self.backup_geometry_raw.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn primary_metadata_raw(&self) -> Ref<'_, Vec<u8>> {
        self.primary_metadata_raw.borrow()
    }
}
impl AndroidSuper_Root {
    pub fn backup_metadata_raw(&self) -> Ref<'_, Vec<u8>> {
        self.backup_metadata_raw.borrow()
    }
}
