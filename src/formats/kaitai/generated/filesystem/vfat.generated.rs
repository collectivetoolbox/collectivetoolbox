// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::dos_datetime::DosDatetime;

/**
 * \sa <https://download.microsoft.com/download/0/8/4/084c452b-b772-4fe5-89bb-a0cbf082286a/fatgen103.doc> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Vfat {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat>,
    pub(crate) _self_shared: SharedType<Self>,
    boot_sector: RefCell<OptRc<Vfat_BootSector>>,
    _io: RefCell<BytesReader>,
    fats_raw: RefCell<Vec<Vec<u8>>>,
    root_dir_raw: RefCell<Vec<u8>>,
    f_fats: Cell<bool>,
    fats: RefCell<Vec<Vec<u8>>>,
    f_root_dir: Cell<bool>,
    root_dir: RefCell<OptRc<Vfat_RootDirectory>>,
}
impl KStruct for Vfat {
    type Root = Vfat;
    type Parent = Vfat;

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
        let t = Self::read_into::<_, Vfat_BootSector>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.boot_sector.borrow_mut() = t;
        Ok(())
    }
}
impl Vfat {
    pub fn fats(
        &self
    ) -> KResult<Ref<'_, Vec<Vec<u8>>>> {
        let _io = self._io.borrow();
        if self.f_fats.get() {
            return Ok(self.fats.borrow());
        }
        self.f_fats.set(true);
        let _pos = _io.pos();
        _io.seek(usize::from(*self.boot_sector().pos_fats()?))?;
        *self.fats_raw.borrow_mut() = Vec::new();
        *self.fats.borrow_mut() = Vec::new();
        let l_fats = usize::from(*self.boot_sector().bpb().num_fats());
        for _i in 0_usize..l_fats {
            self.fats_raw.borrow_mut().push(_io.read_bytes(usize::try_from(*self.boot_sector().size_fat()?)?)?.into());
            let fats_raw = self.fats_raw.borrow();
            let _io_fats_raw = BytesReader::from(fats_raw.last().ok_or(KError::EmptyIterator)?.clone());
        }
        _io.seek(_pos)?;
        Ok(self.fats.borrow())
    }
    pub fn root_dir(
        &self
    ) -> KResult<Ref<'_, OptRc<Vfat_RootDirectory>>> {
        let _io = self._io.borrow();
        if self.f_root_dir.get() {
            return Ok(self.root_dir.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.boot_sector().pos_root_dir()?)?)?;
        *self.root_dir_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self.boot_sector().size_root_dir()?)?)?.into();
        let root_dir_raw = self.root_dir_raw.borrow();
        let _t_root_dir_raw_io = BytesReader::from(root_dir_raw.clone());
        let t = Self::read_into::<BytesReader, Vfat_RootDirectory>(&_t_root_dir_raw_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.root_dir.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.root_dir.borrow())
    }
}
impl Vfat {
    pub fn boot_sector(&self) -> Ref<'_, OptRc<Vfat_BootSector>> {
        self.boot_sector.borrow()
    }
}
impl Vfat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Vfat {
    pub fn fats_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.fats_raw.borrow()
    }
}
impl Vfat {
    pub fn root_dir_raw(&self) -> Ref<'_, Vec<u8>> {
        self.root_dir_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vfat_BiosParamBlock {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat_BootSector>,
    pub(crate) _self_shared: SharedType<Self>,
    bytes_per_ls: RefCell<u16>,
    ls_per_clus: RefCell<u8>,
    num_reserved_ls: RefCell<u16>,
    num_fats: RefCell<u8>,
    max_root_dir_rec: RefCell<u16>,
    total_ls_2: RefCell<u16>,
    media_code: RefCell<u8>,
    ls_per_fat: RefCell<u16>,
    ps_per_track: RefCell<u16>,
    num_heads: RefCell<u16>,
    num_hidden_sectors: RefCell<u32>,
    total_ls_4: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vfat_BiosParamBlock {
    type Root = Vfat;
    type Parent = Vfat_BootSector;

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
        *self_rc.bytes_per_ls.borrow_mut() = _io.read_u2le()?;
        *self_rc.ls_per_clus.borrow_mut() = _io.read_u1()?;
        *self_rc.num_reserved_ls.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_fats.borrow_mut() = _io.read_u1()?;
        *self_rc.max_root_dir_rec.borrow_mut() = _io.read_u2le()?;
        *self_rc.total_ls_2.borrow_mut() = _io.read_u2le()?;
        *self_rc.media_code.borrow_mut() = _io.read_u1()?;
        *self_rc.ls_per_fat.borrow_mut() = _io.read_u2le()?;
        *self_rc.ps_per_track.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_heads.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_hidden_sectors.borrow_mut() = _io.read_u4le()?;
        *self_rc.total_ls_4.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl Vfat_BiosParamBlock {
}

/**
 * Bytes per logical sector
 */
impl Vfat_BiosParamBlock {
    pub fn bytes_per_ls(&self) -> Ref<'_, u16> {
        self.bytes_per_ls.borrow()
    }
}

/**
 * Logical sectors per cluster
 */
impl Vfat_BiosParamBlock {
    pub fn ls_per_clus(&self) -> Ref<'_, u8> {
        self.ls_per_clus.borrow()
    }
}

/**
 * Count of reserved logical sectors. The number of logical
 * sectors before the first FAT in the file system image.
 */
impl Vfat_BiosParamBlock {
    pub fn num_reserved_ls(&self) -> Ref<'_, u16> {
        self.num_reserved_ls.borrow()
    }
}

/**
 * Number of File Allocation Tables
 */
impl Vfat_BiosParamBlock {
    pub fn num_fats(&self) -> Ref<'_, u8> {
        self.num_fats.borrow()
    }
}

/**
 * Maximum number of FAT12 or FAT16 root directory entries. 0
 * for FAT32, where the root directory is stored in ordinary
 * data clusters.
 */
impl Vfat_BiosParamBlock {
    pub fn max_root_dir_rec(&self) -> Ref<'_, u16> {
        self.max_root_dir_rec.borrow()
    }
}

/**
 * Total logical sectors (if zero, use total_ls_4)
 */
impl Vfat_BiosParamBlock {
    pub fn total_ls_2(&self) -> Ref<'_, u16> {
        self.total_ls_2.borrow()
    }
}

/**
 * Media descriptor
 */
impl Vfat_BiosParamBlock {
    pub fn media_code(&self) -> Ref<'_, u8> {
        self.media_code.borrow()
    }
}

/**
 * Logical sectors per File Allocation Table for
 * FAT12/FAT16. FAT32 sets this to 0 and uses the 32-bit value
 * at offset 0x024 instead.
 */
impl Vfat_BiosParamBlock {
    pub fn ls_per_fat(&self) -> Ref<'_, u16> {
        self.ls_per_fat.borrow()
    }
}

/**
 * Physical sectors per track for disks with INT 13h CHS
 * geometry, e.g., 15 for a "1.20 MB" (1200 KB) floppy. A zero
 * entry indicates that this entry is reserved, but not used.
 */
impl Vfat_BiosParamBlock {
    pub fn ps_per_track(&self) -> Ref<'_, u16> {
        self.ps_per_track.borrow()
    }
}

/**
 * Number of heads for disks with INT 13h CHS geometry,[9]
 * e.g., 2 for a double sided floppy.
 */
impl Vfat_BiosParamBlock {
    pub fn num_heads(&self) -> Ref<'_, u16> {
        self.num_heads.borrow()
    }
}

/**
 * Number of hidden sectors preceding the partition that
 * contains this FAT volume. This field should always be zero
 * on media that are not partitioned. This DOS 3.0 entry is
 * incompatible with a similar entry at offset 0x01C in BPBs
 * since DOS 3.31.  It must not be used if the logical sectors
 * entry at offset 0x013 is zero.
 */
impl Vfat_BiosParamBlock {
    pub fn num_hidden_sectors(&self) -> Ref<'_, u32> {
        self.num_hidden_sectors.borrow()
    }
}

/**
 * Total logical sectors including hidden sectors. This DOS 3.2
 * entry is incompatible with a similar entry at offset 0x020
 * in BPBs since DOS 3.31. It must not be used if the logical
 * sectors entry at offset 0x013 is zero.
 */
impl Vfat_BiosParamBlock {
    pub fn total_ls_4(&self) -> Ref<'_, u32> {
        self.total_ls_4.borrow()
    }
}
impl Vfat_BiosParamBlock {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vfat_BootSector {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat>,
    pub(crate) _self_shared: SharedType<Self>,
    jmp_instruction: RefCell<Vec<u8>>,
    oem_name: RefCell<String>,
    bpb: RefCell<OptRc<Vfat_BiosParamBlock>>,
    ebpb_fat16: RefCell<OptRc<Vfat_ExtBiosParamBlockFat16>>,
    ebpb_fat32: RefCell<OptRc<Vfat_ExtBiosParamBlockFat32>>,
    _io: RefCell<BytesReader>,
    f_is_fat32: Cell<bool>,
    is_fat32: RefCell<bool>,
    f_ls_per_fat: Cell<bool>,
    ls_per_fat: RefCell<u32>,
    f_ls_per_root_dir: Cell<bool>,
    ls_per_root_dir: RefCell<i32>,
    f_pos_fats: Cell<bool>,
    pos_fats: RefCell<u16>,
    f_pos_root_dir: Cell<bool>,
    pos_root_dir: RefCell<u32>,
    f_size_fat: Cell<bool>,
    size_fat: RefCell<u32>,
    f_size_root_dir: Cell<bool>,
    size_root_dir: RefCell<i32>,
}
impl KStruct for Vfat_BootSector {
    type Root = Vfat;
    type Parent = Vfat;

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
        *self_rc.jmp_instruction.borrow_mut() = _io.read_bytes(3_usize)?;
        *self_rc.oem_name.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 32), "ASCII")?;
        let t = Self::read_into::<_, Vfat_BiosParamBlock>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.bpb.borrow_mut() = t;
        if !(*self_rc.is_fat32()?) {
            let t = Self::read_into::<_, Vfat_ExtBiosParamBlockFat16>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.ebpb_fat16.borrow_mut() = t;
        }
        if *self_rc.is_fat32()? {
            let t = Self::read_into::<_, Vfat_ExtBiosParamBlockFat32>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.ebpb_fat32.borrow_mut() = t;
        }
        Ok(())
    }
}
impl Vfat_BootSector {

    /**
     * Determines if filesystem is FAT32 (true) or FAT12/16 (false)
     * by analyzing some preliminary conditions in BPB. Used to
     * determine whether we should parse post-BPB data as
     * `ext_bios_param_block_fat16` or `ext_bios_param_block_fat32`.
     */
    pub fn is_fat32(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_fat32.get() {
            return Ok(self.is_fat32.borrow());
        }
        self.f_is_fat32.set(true);
        *self.is_fat32.borrow_mut() = (*self.bpb().max_root_dir_rec() == 0).try_into()?;
        Ok(self.is_fat32.borrow())
    }
    pub fn ls_per_fat(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_ls_per_fat.get() {
            return Ok(self.ls_per_fat.borrow());
        }
        self.f_ls_per_fat.set(true);
        *self.ls_per_fat.borrow_mut() = (if *self.is_fat32()? { *self.ebpb_fat32().ls_per_fat() } else { u32::from(*self.bpb().ls_per_fat()) }).try_into()?;
        Ok(self.ls_per_fat.borrow())
    }

    /**
     * Size of root directory in logical sectors
     * \sa FAT: General Overview of On-Disk Format, section "FAT Data Structure"
     */
    pub fn ls_per_root_dir(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ls_per_root_dir.get() {
            return Ok(self.ls_per_root_dir.borrow());
        }
        self.f_ls_per_root_dir.set(true);
        *self.ls_per_root_dir.borrow_mut() = (((((i32::from(*self.bpb().max_root_dir_rec())).saturating_mul(32_i32)).saturating_add(i32::from(*self.bpb().bytes_per_ls()))).saturating_sub(1_i32)).checked_div(i32::from(*self.bpb().bytes_per_ls())).ok_or(KError::CastError)?).try_into()?;
        Ok(self.ls_per_root_dir.borrow())
    }

    /**
     * Offset of FATs in bytes from start of filesystem
     */
    pub fn pos_fats(
        &self
    ) -> KResult<Ref<'_, u16>> {
        let _io = self._io.borrow();
        if self.f_pos_fats.get() {
            return Ok(self.pos_fats.borrow());
        }
        self.f_pos_fats.set(true);
        *self.pos_fats.borrow_mut() = ((*self.bpb().bytes_per_ls()).saturating_mul(*self.bpb().num_reserved_ls())).try_into()?;
        Ok(self.pos_fats.borrow())
    }

    /**
     * Offset of root directory in bytes from start of filesystem
     */
    pub fn pos_root_dir(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_pos_root_dir.get() {
            return Ok(self.pos_root_dir.borrow());
        }
        self.f_pos_root_dir.set(true);
        *self.pos_root_dir.borrow_mut() = ((u32::from(*self.bpb().bytes_per_ls())).saturating_mul((u32::from(*self.bpb().num_reserved_ls())).saturating_add((*self.ls_per_fat()?).saturating_mul(u32::from(*self.bpb().num_fats()))))).try_into()?;
        Ok(self.pos_root_dir.borrow())
    }

    /**
     * Size of one FAT in bytes
     */
    pub fn size_fat(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_size_fat.get() {
            return Ok(self.size_fat.borrow());
        }
        self.f_size_fat.set(true);
        *self.size_fat.borrow_mut() = ((u32::from(*self.bpb().bytes_per_ls())).saturating_mul(*self.ls_per_fat()?)).try_into()?;
        Ok(self.size_fat.borrow())
    }

    /**
     * Size of root directory in bytes
     */
    pub fn size_root_dir(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_size_root_dir.get() {
            return Ok(self.size_root_dir.borrow());
        }
        self.f_size_root_dir.set(true);
        *self.size_root_dir.borrow_mut() = ((*self.ls_per_root_dir()?).saturating_mul(i32::from(*self.bpb().bytes_per_ls()))).try_into()?;
        Ok(self.size_root_dir.borrow())
    }
}
impl Vfat_BootSector {
    pub fn jmp_instruction(&self) -> Ref<'_, Vec<u8>> {
        self.jmp_instruction.borrow()
    }
}
impl Vfat_BootSector {
    pub fn oem_name(&self) -> Ref<'_, String> {
        self.oem_name.borrow()
    }
}

/**
 * Basic BIOS parameter block, present in all versions of FAT
 */
impl Vfat_BootSector {
    pub fn bpb(&self) -> Ref<'_, OptRc<Vfat_BiosParamBlock>> {
        self.bpb.borrow()
    }
}

/**
 * FAT12/16-specific extended BIOS parameter block
 */
impl Vfat_BootSector {
    pub fn ebpb_fat16(&self) -> Ref<'_, OptRc<Vfat_ExtBiosParamBlockFat16>> {
        self.ebpb_fat16.borrow()
    }
}

/**
 * FAT32-specific extended BIOS parameter block
 */
impl Vfat_BootSector {
    pub fn ebpb_fat32(&self) -> Ref<'_, OptRc<Vfat_ExtBiosParamBlockFat32>> {
        self.ebpb_fat32.borrow()
    }
}
impl Vfat_BootSector {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Extended BIOS Parameter Block (DOS 4.0+, OS/2 1.0+). Used only
 * for FAT12 and FAT16.
 */

#[derive(Default, Debug, Clone)]
pub struct Vfat_ExtBiosParamBlockFat16 {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat_BootSector>,
    pub(crate) _self_shared: SharedType<Self>,
    phys_drive_num: RefCell<u8>,
    reserved1: RefCell<u8>,
    ext_boot_sign: RefCell<u8>,
    volume_id: RefCell<Vec<u8>>,
    partition_volume_label: RefCell<String>,
    fs_type_str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vfat_ExtBiosParamBlockFat16 {
    type Root = Vfat;
    type Parent = Vfat_BootSector;

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
        *self_rc.phys_drive_num.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved1.borrow_mut() = _io.read_u1()?;
        *self_rc.ext_boot_sign.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_id.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.partition_volume_label.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(11_usize)?, 32), "ASCII")?;
        *self_rc.fs_type_str.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 32), "ASCII")?;
        Ok(())
    }
}
impl Vfat_ExtBiosParamBlockFat16 {
}

/**
 * Physical drive number (0x00 for (first) removable media,
 * 0x80 for (first) fixed disk as per INT 13h).
 */
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn phys_drive_num(&self) -> Ref<'_, u8> {
        self.phys_drive_num.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn reserved1(&self) -> Ref<'_, u8> {
        self.reserved1.borrow()
    }
}

/**
 * Should be 0x29 to indicate that an EBPB with the following 3
 * entries exists.
 */
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn ext_boot_sign(&self) -> Ref<'_, u8> {
        self.ext_boot_sign.borrow()
    }
}

/**
 * Volume ID (serial number).
 *
 * Typically the serial number "xxxx-xxxx" is created by a
 * 16-bit addition of both DX values returned by INT 21h/AH=2Ah
 * (get system date) and INT 21h/AH=2Ch (get system time) for
 * the high word and another 16-bit addition of both CX values
 * for the low word of the serial number. Alternatively, some
 * DR-DOS disk utilities provide a /# option to generate a
 * human-readable time stamp "mmdd-hhmm" build from BCD-encoded
 * 8-bit values for the month, day, hour and minute instead of
 * a serial number.
 */
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn volume_id(&self) -> Ref<'_, Vec<u8>> {
        self.volume_id.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn partition_volume_label(&self) -> Ref<'_, String> {
        self.partition_volume_label.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn fs_type_str(&self) -> Ref<'_, String> {
        self.fs_type_str.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat16 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * Extended BIOS Parameter Block for FAT32
 */

#[derive(Default, Debug, Clone)]
pub struct Vfat_ExtBiosParamBlockFat32 {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat_BootSector>,
    pub(crate) _self_shared: SharedType<Self>,
    ls_per_fat: RefCell<u32>,
    has_active_fat: RefCell<bool>,
    reserved1: RefCell<u64>,
    active_fat_id: RefCell<u64>,
    reserved2: RefCell<Vec<u8>>,
    fat_version: RefCell<u16>,
    root_dir_start_clus: RefCell<u32>,
    ls_fs_info: RefCell<u16>,
    boot_sectors_copy_start_ls: RefCell<u16>,
    reserved3: RefCell<Vec<u8>>,
    phys_drive_num: RefCell<u8>,
    reserved4: RefCell<u8>,
    ext_boot_sign: RefCell<u8>,
    volume_id: RefCell<Vec<u8>>,
    partition_volume_label: RefCell<String>,
    fs_type_str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vfat_ExtBiosParamBlockFat32 {
    type Root = Vfat;
    type Parent = Vfat_BootSector;

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
        *self_rc.ls_per_fat.borrow_mut() = _io.read_u4le()?;
        *self_rc.has_active_fat.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(3)?;
        *self_rc.active_fat_id.borrow_mut() = _io.read_bits_int_be(4)?;
        io.align_to_byte()?;
        *self_rc.reserved2.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.reserved2() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/ext_bios_param_block_fat32/seq/4".to_string() }));
        }
        *self_rc.fat_version.borrow_mut() = _io.read_u2le()?;
        *self_rc.root_dir_start_clus.borrow_mut() = _io.read_u4le()?;
        *self_rc.ls_fs_info.borrow_mut() = _io.read_u2le()?;
        *self_rc.boot_sectors_copy_start_ls.borrow_mut() = _io.read_u2le()?;
        *self_rc.reserved3.borrow_mut() = _io.read_bytes(12_usize)?;
        *self_rc.phys_drive_num.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved4.borrow_mut() = _io.read_u1()?;
        *self_rc.ext_boot_sign.borrow_mut() = _io.read_u1()?;
        *self_rc.volume_id.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.partition_volume_label.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(11_usize)?, 32), "ASCII")?;
        *self_rc.fs_type_str.borrow_mut() = bytes_to_str(&bytes_strip_right(&_io.read_bytes(8_usize)?, 32), "ASCII")?;
        Ok(())
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
}

/**
 * Logical sectors per file allocation table (corresponds with
 * the old entry `ls_per_fat` in the DOS 2.0 BPB).
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn ls_per_fat(&self) -> Ref<'_, u32> {
        self.ls_per_fat.borrow()
    }
}

/**
 * If true, then there is "active" FAT, which is designated in
 * `active_fat` attribute. If false, all FATs are mirrored as
 * usual.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn has_active_fat(&self) -> Ref<'_, bool> {
        self.has_active_fat.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}

/**
 * Zero-based number of active FAT, if `has_active_fat`
 * attribute is true.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn active_fat_id(&self) -> Ref<'_, u64> {
        self.active_fat_id.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn reserved2(&self) -> Ref<'_, Vec<u8>> {
        self.reserved2.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn fat_version(&self) -> Ref<'_, u16> {
        self.fat_version.borrow()
    }
}

/**
 * Cluster number of root directory start, typically 2 if it
 * contains no bad sector. (Microsoft's FAT32 implementation
 * imposes an artificial limit of 65,535 entries per directory,
 * whilst many third-party implementations do not.)
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn root_dir_start_clus(&self) -> Ref<'_, u32> {
        self.root_dir_start_clus.borrow()
    }
}

/**
 * Logical sector number of FS Information Sector, typically 1,
 * i.e., the second of the three FAT32 boot sectors. Values
 * like 0 and 0xFFFF are used by some FAT32 implementations to
 * designate absence of FS Information Sector.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn ls_fs_info(&self) -> Ref<'_, u16> {
        self.ls_fs_info.borrow()
    }
}

/**
 * First logical sector number of a copy of the three FAT32
 * boot sectors, typically 6.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn boot_sectors_copy_start_ls(&self) -> Ref<'_, u16> {
        self.boot_sectors_copy_start_ls.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn reserved3(&self) -> Ref<'_, Vec<u8>> {
        self.reserved3.borrow()
    }
}

/**
 * Physical drive number (0x00 for (first) removable media,
 * 0x80 for (first) fixed disk as per INT 13h).
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn phys_drive_num(&self) -> Ref<'_, u8> {
        self.phys_drive_num.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn reserved4(&self) -> Ref<'_, u8> {
        self.reserved4.borrow()
    }
}

/**
 * Should be 0x29 to indicate that an EBPB with the following 3
 * entries exists.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn ext_boot_sign(&self) -> Ref<'_, u8> {
        self.ext_boot_sign.borrow()
    }
}

/**
 * Volume ID (serial number).
 *
 * Typically the serial number "xxxx-xxxx" is created by a
 * 16-bit addition of both DX values returned by INT 21h/AH=2Ah
 * (get system date) and INT 21h/AH=2Ch (get system time) for
 * the high word and another 16-bit addition of both CX values
 * for the low word of the serial number. Alternatively, some
 * DR-DOS disk utilities provide a /# option to generate a
 * human-readable time stamp "mmdd-hhmm" build from BCD-encoded
 * 8-bit values for the month, day, hour and minute instead of
 * a serial number.
 */
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn volume_id(&self) -> Ref<'_, Vec<u8>> {
        self.volume_id.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn partition_volume_label(&self) -> Ref<'_, String> {
        self.partition_volume_label.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn fs_type_str(&self) -> Ref<'_, String> {
        self.fs_type_str.borrow()
    }
}
impl Vfat_ExtBiosParamBlockFat32 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vfat_RootDirectory {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat>,
    pub(crate) _self_shared: SharedType<Self>,
    records: RefCell<Vec<OptRc<Vfat_RootDirectoryRec>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vfat_RootDirectory {
    type Root = Vfat;
    type Parent = Vfat;

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
        *self_rc.records.borrow_mut() = Vec::new();
        let l_records = usize::from(*self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.boot_sector().bpb().max_root_dir_rec());
        for _i in 0_usize..l_records {
            let t = Self::read_into::<_, Vfat_RootDirectoryRec>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.records.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl Vfat_RootDirectory {
}
impl Vfat_RootDirectory {
    pub fn records(&self) -> Ref<'_, Vec<OptRc<Vfat_RootDirectoryRec>>> {
        self.records.borrow()
    }
}
impl Vfat_RootDirectory {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vfat_RootDirectoryRec {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat_RootDirectory>,
    pub(crate) _self_shared: SharedType<Self>,
    file_name: RefCell<Vec<u8>>,
    attrs: RefCell<OptRc<Vfat_RootDirectoryRec_AttrFlags>>,
    reserved: RefCell<Vec<u8>>,
    last_write_time: RefCell<OptRc<DosDatetime>>,
    start_clus: RefCell<u16>,
    file_size: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Vfat_RootDirectoryRec {
    type Root = Vfat;
    type Parent = Vfat_RootDirectory;

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
        *self_rc.file_name.borrow_mut() = _io.read_bytes(11_usize)?;
        let t = Self::read_into::<_, Vfat_RootDirectoryRec_AttrFlags>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.attrs.borrow_mut() = t;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(10_usize)?;
        let t = Self::read_into::<_, DosDatetime>(&*_io, None, None)?.into();
        *self_rc.last_write_time.borrow_mut() = t;
        *self_rc.start_clus.borrow_mut() = _io.read_u2le()?;
        *self_rc.file_size.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl Vfat_RootDirectoryRec {
}
impl Vfat_RootDirectoryRec {
    pub fn file_name(&self) -> Ref<'_, Vec<u8>> {
        self.file_name.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn attrs(&self) -> Ref<'_, OptRc<Vfat_RootDirectoryRec_AttrFlags>> {
        self.attrs.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn last_write_time(&self) -> Ref<'_, OptRc<DosDatetime>> {
        self.last_write_time.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn start_clus(&self) -> Ref<'_, u16> {
        self.start_clus.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn file_size(&self) -> Ref<'_, u32> {
        self.file_size.borrow()
    }
}
impl Vfat_RootDirectoryRec {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vfat_RootDirectoryRec_AttrFlags {
    pub(crate) _root: SharedType<Vfat>,
    pub(crate) _parent: SharedType<Vfat_RootDirectoryRec>,
    pub(crate) _self_shared: SharedType<Self>,
    read_only: RefCell<bool>,
    hidden: RefCell<bool>,
    system: RefCell<bool>,
    volume_id: RefCell<bool>,
    is_directory: RefCell<bool>,
    archive: RefCell<bool>,
    reserved: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_long_name: Cell<bool>,
    long_name: RefCell<bool>,
}
impl KStruct for Vfat_RootDirectoryRec_AttrFlags {
    type Root = Vfat;
    type Parent = Vfat_RootDirectoryRec;

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
        *self_rc.read_only.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.hidden.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.system.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.volume_id.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.is_directory.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.archive.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(2)?;
        Ok(())
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn long_name(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_long_name.get() {
            return Ok(self.long_name.borrow());
        }
        self.f_long_name.set(true);
        *self.long_name.borrow_mut() = ( ((*self.read_only()) && (*self.hidden()) && (*self.system()) && (*self.volume_id())) ).try_into()?;
        Ok(self.long_name.borrow())
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn read_only(&self) -> Ref<'_, bool> {
        self.read_only.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn hidden(&self) -> Ref<'_, bool> {
        self.hidden.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn system(&self) -> Ref<'_, bool> {
        self.system.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn volume_id(&self) -> Ref<'_, bool> {
        self.volume_id.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn is_directory(&self) -> Ref<'_, bool> {
        self.is_directory.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn archive(&self) -> Ref<'_, bool> {
        self.archive.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Vfat_RootDirectoryRec_AttrFlags {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
