// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * MBR (Master Boot Record) partition table is a traditional way of
 * MS-DOS to partition larger hard disc drives into distinct
 * partitions.
 *
 * This table is stored in the end of the boot sector (first sector) of
 * the drive, after the bootstrap code. Original DOS 2.0 specification
 * allowed only 4 partitions per disc, but DOS 3.2 introduced concept
 * of "extended partitions", which work as nested extra "boot records"
 * which are pointed to by original ("primary") partitions in MBR.
 */

#[derive(Default, Debug, Clone)]
pub struct MbrPartitionTable {
    pub(crate) _root: SharedType<MbrPartitionTable>,
    pub(crate) _parent: SharedType<MbrPartitionTable>,
    pub(crate) _self_shared: SharedType<Self>,
    bootstrap_code: RefCell<Vec<u8>>,
    partitions: RefCell<Vec<OptRc<MbrPartitionTable_PartitionEntry>>>,
    boot_signature: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MbrPartitionTable {
    type Root = MbrPartitionTable;
    type Parent = MbrPartitionTable;

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
        *self_rc.bootstrap_code.borrow_mut() = _io.read_bytes(446_usize)?;
        *self_rc.partitions.borrow_mut() = Vec::new();
        let l_partitions = usize::try_from(4)?;
        for _i in 0_usize..l_partitions {
            let t = Self::read_into::<_, MbrPartitionTable_PartitionEntry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.partitions.borrow_mut().push(t);
        }
        *self_rc.boot_signature.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.boot_signature() == vec![0x55u8, 0xaau8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/2".to_string() }));
        }
        Ok(())
    }
}
impl MbrPartitionTable {
}
impl MbrPartitionTable {
    pub fn bootstrap_code(&self) -> Ref<'_, Vec<u8>> {
        self.bootstrap_code.borrow()
    }
}
impl MbrPartitionTable {
    pub fn partitions(&self) -> Ref<'_, Vec<OptRc<MbrPartitionTable_PartitionEntry>>> {
        self.partitions.borrow()
    }
}
impl MbrPartitionTable {
    pub fn boot_signature(&self) -> Ref<'_, Vec<u8>> {
        self.boot_signature.borrow()
    }
}
impl MbrPartitionTable {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MbrPartitionTable_Chs {
    pub(crate) _root: SharedType<MbrPartitionTable>,
    pub(crate) _parent: SharedType<MbrPartitionTable_PartitionEntry>,
    pub(crate) _self_shared: SharedType<Self>,
    head: RefCell<u8>,
    b2: RefCell<u8>,
    b3: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_cylinder: Cell<bool>,
    cylinder: RefCell<i32>,
    f_sector: Cell<bool>,
    sector: RefCell<i32>,
}
impl KStruct for MbrPartitionTable_Chs {
    type Root = MbrPartitionTable;
    type Parent = MbrPartitionTable_PartitionEntry;

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
        *self_rc.head.borrow_mut() = _io.read_u1()?;
        *self_rc.b2.borrow_mut() = _io.read_u1()?;
        *self_rc.b3.borrow_mut() = _io.read_u1()?;
        Ok(())
    }
}
impl MbrPartitionTable_Chs {
    pub fn cylinder(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_cylinder.get() {
            return Ok(self.cylinder.borrow());
        }
        self.f_cylinder.set(true);
        *self.cylinder.borrow_mut() = ((i32::from(*self.b3())).saturating_add((((i32::from(*self.b2())) & (192_i32))).wrapping_shl(2_u32))).try_into()?;
        Ok(self.cylinder.borrow())
    }
    pub fn sector(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_sector.get() {
            return Ok(self.sector.borrow());
        }
        self.f_sector.set(true);
        *self.sector.borrow_mut() = (((i32::from(*self.b2())) & (63_i32))).try_into()?;
        Ok(self.sector.borrow())
    }
}
impl MbrPartitionTable_Chs {
    pub fn head(&self) -> Ref<'_, u8> {
        self.head.borrow()
    }
}
impl MbrPartitionTable_Chs {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl MbrPartitionTable_Chs {
    pub fn b3(&self) -> Ref<'_, u8> {
        self.b3.borrow()
    }
}
impl MbrPartitionTable_Chs {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MbrPartitionTable_PartitionEntry {
    pub(crate) _root: SharedType<MbrPartitionTable>,
    pub(crate) _parent: SharedType<MbrPartitionTable>,
    pub(crate) _self_shared: SharedType<Self>,
    status: RefCell<u8>,
    chs_start: RefCell<OptRc<MbrPartitionTable_Chs>>,
    partition_type: RefCell<u8>,
    chs_end: RefCell<OptRc<MbrPartitionTable_Chs>>,
    lba_start: RefCell<u32>,
    num_sectors: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MbrPartitionTable_PartitionEntry {
    type Root = MbrPartitionTable;
    type Parent = MbrPartitionTable;

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
        *self_rc.status.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, MbrPartitionTable_Chs>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.chs_start.borrow_mut() = t;
        *self_rc.partition_type.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, MbrPartitionTable_Chs>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.chs_end.borrow_mut() = t;
        *self_rc.lba_start.borrow_mut() = _io.read_u4le()?;
        *self_rc.num_sectors.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl MbrPartitionTable_PartitionEntry {
}
impl MbrPartitionTable_PartitionEntry {
    pub fn status(&self) -> Ref<'_, u8> {
        self.status.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn chs_start(&self) -> Ref<'_, OptRc<MbrPartitionTable_Chs>> {
        self.chs_start.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn partition_type(&self) -> Ref<'_, u8> {
        self.partition_type.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn chs_end(&self) -> Ref<'_, OptRc<MbrPartitionTable_Chs>> {
        self.chs_end.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn lba_start(&self) -> Ref<'_, u32> {
        self.lba_start.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn num_sectors(&self) -> Ref<'_, u32> {
        self.num_sectors.borrow()
    }
}
impl MbrPartitionTable_PartitionEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
