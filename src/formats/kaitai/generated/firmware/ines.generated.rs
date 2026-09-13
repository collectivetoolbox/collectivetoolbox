// SPDX-License-Identifier: WTFPL
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://www.nesdev.org/wiki/INES> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Ines_Header>>,
    trainer: RefCell<Vec<u8>>,
    prg_rom: RefCell<Vec<u8>>,
    chr_rom: RefCell<Vec<u8>>,
    playchoice10: RefCell<OptRc<Ines_Playchoice10>>,
    title: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines {
    type Root = Ines;
    type Parent = Ines;

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
        let t = Self::read_into::<_, Ines_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        if *self_rc.header().f6().trainer() {
            *self_rc.trainer.borrow_mut() = _io.read_bytes(512_usize)?;
        }
        *self_rc.prg_rom.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.header().len_prg_rom())).saturating_mul(16384_i32))?)?;
        *self_rc.chr_rom.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.header().len_chr_rom())).saturating_mul(8192_i32))?)?;
        if *self_rc.header().f7().playchoice10() {
            let t = Self::read_into::<_, Ines_Playchoice10>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.playchoice10.borrow_mut() = t;
        }
        if !(_io.is_eof()) {
            *self_rc.title.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-8")?;
        }
        Ok(())
    }
}
impl Ines {
}
impl Ines {
    pub fn header(&self) -> Ref<'_, OptRc<Ines_Header>> {
        self.header.borrow()
    }
}
impl Ines {
    pub fn trainer(&self) -> Ref<'_, Vec<u8>> {
        self.trainer.borrow()
    }
}
impl Ines {
    pub fn prg_rom(&self) -> Ref<'_, Vec<u8>> {
        self.prg_rom.borrow()
    }
}
impl Ines {
    pub fn chr_rom(&self) -> Ref<'_, Vec<u8>> {
        self.chr_rom.borrow()
    }
}
impl Ines {
    pub fn playchoice10(&self) -> Ref<'_, OptRc<Ines_Playchoice10>> {
        self.playchoice10.borrow()
    }
}
impl Ines {
    pub fn title(&self) -> Ref<'_, String> {
        self.title.borrow()
    }
}
impl Ines {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ines_Header {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    len_prg_rom: RefCell<u8>,
    len_chr_rom: RefCell<u8>,
    f6: RefCell<OptRc<Ines_Header_F6>>,
    f7: RefCell<OptRc<Ines_Header_F7>>,
    len_prg_ram: RefCell<u8>,
    f9: RefCell<OptRc<Ines_Header_F9>>,
    f10: RefCell<OptRc<Ines_Header_F10>>,
    reserved: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_mapper: Cell<bool>,
    mapper: RefCell<u64>,
}
impl KStruct for Ines_Header {
    type Root = Ines;
    type Parent = Ines;

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
        if !(*self_rc.magic() == vec![0x4eu8, 0x45u8, 0x53u8, 0x1au8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/0".to_string() }));
        }
        *self_rc.len_prg_rom.borrow_mut() = _io.read_u1()?;
        *self_rc.len_chr_rom.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Ines_Header_F6>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.f6.borrow_mut() = t;
        let t = Self::read_into::<_, Ines_Header_F7>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.f7.borrow_mut() = t;
        *self_rc.len_prg_ram.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, Ines_Header_F9>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.f9.borrow_mut() = t;
        let t = Self::read_into::<_, Ines_Header_F10>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.f10.borrow_mut() = t;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(5_usize)?;
        if !(*self_rc.reserved() == vec![0x0u8, 0x0u8, 0x0u8, 0x0u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/8".to_string() }));
        }
        Ok(())
    }
}
impl Ines_Header {

    /**
     * \sa <https://www.nesdev.org/wiki/Mapper> Source
     */
    pub fn mapper(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_mapper.get() {
            return Ok(self.mapper.borrow());
        }
        self.f_mapper.set(true);
        *self.mapper.borrow_mut() = (((*self.f6().lower_mapper()) | (u64::try_from((i32::try_from(*self.f7().upper_mapper())?).wrapping_shl(4_u32))?))).try_into()?;
        Ok(self.mapper.borrow())
    }
}
impl Ines_Header {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * Size of PRG ROM in 16 KB units
 */
impl Ines_Header {
    pub fn len_prg_rom(&self) -> Ref<'_, u8> {
        self.len_prg_rom.borrow()
    }
}

/**
 * Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
 */
impl Ines_Header {
    pub fn len_chr_rom(&self) -> Ref<'_, u8> {
        self.len_chr_rom.borrow()
    }
}
impl Ines_Header {
    pub fn f6(&self) -> Ref<'_, OptRc<Ines_Header_F6>> {
        self.f6.borrow()
    }
}
impl Ines_Header {
    pub fn f7(&self) -> Ref<'_, OptRc<Ines_Header_F7>> {
        self.f7.borrow()
    }
}

/**
 * Size of PRG RAM in 8 KB units (Value 0 infers 8 KB for compatibility; see PRG RAM circuit on nesdev.com)
 */
impl Ines_Header {
    pub fn len_prg_ram(&self) -> Ref<'_, u8> {
        self.len_prg_ram.borrow()
    }
}
impl Ines_Header {
    pub fn f9(&self) -> Ref<'_, OptRc<Ines_Header_F9>> {
        self.f9.borrow()
    }
}

/**
 * this one is unofficial
 */
impl Ines_Header {
    pub fn f10(&self) -> Ref<'_, OptRc<Ines_Header_F10>> {
        self.f10.borrow()
    }
}
impl Ines_Header {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl Ines_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://www.nesdev.org/wiki/INES#Flags_10> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines_Header_F10 {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<u64>,
    bus_conflict: RefCell<bool>,
    prg_ram: RefCell<bool>,
    reserved2: RefCell<u64>,
    tv_system: RefCell<Ines_Header_F10_TvSystem>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Header_F10 {
    type Root = Ines;
    type Parent = Ines_Header;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.bus_conflict.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.prg_ram.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved2.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.tv_system.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        Ok(())
    }
}
impl Ines_Header_F10 {
}
impl Ines_Header_F10 {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}

/**
 * If 0, no bus conflicts. If 1, bus conflicts.
 */
impl Ines_Header_F10 {
    pub fn bus_conflict(&self) -> Ref<'_, bool> {
        self.bus_conflict.borrow()
    }
}

/**
 * If 0, PRG ram is present. If 1, not present.
 */
impl Ines_Header_F10 {
    pub fn prg_ram(&self) -> Ref<'_, bool> {
        self.prg_ram.borrow()
    }
}
impl Ines_Header_F10 {
    pub fn reserved2(&self) -> Ref<'_, u64> {
        self.reserved2.borrow()
    }
}

/**
 * if 0, NTSC. If 2, PAL. If 1 or 3, dual compatible.
 */
impl Ines_Header_F10 {
    pub fn tv_system(&self) -> Ref<'_, Ines_Header_F10_TvSystem> {
        self.tv_system.borrow()
    }
}
impl Ines_Header_F10 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Ines_Header_F10_TvSystem {
    Ntsc,
    Dual1,
    Pal,
    Dual2,
    Unknown(i64),
}

impl TryFrom<i64> for Ines_Header_F10_TvSystem {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Ines_Header_F10_TvSystem> {
        match flag {
            0 => Ok(Ines_Header_F10_TvSystem::Ntsc),
            1 => Ok(Ines_Header_F10_TvSystem::Dual1),
            2 => Ok(Ines_Header_F10_TvSystem::Pal),
            3 => Ok(Ines_Header_F10_TvSystem::Dual2),
            _ => Ok(Ines_Header_F10_TvSystem::Unknown(flag)),
        }
    }
}

impl From<&Ines_Header_F10_TvSystem> for i64 {
    fn from(v: &Ines_Header_F10_TvSystem) -> Self {
        match *v {
            Ines_Header_F10_TvSystem::Ntsc => 0,
            Ines_Header_F10_TvSystem::Dual1 => 1,
            Ines_Header_F10_TvSystem::Pal => 2,
            Ines_Header_F10_TvSystem::Dual2 => 3,
            Ines_Header_F10_TvSystem::Unknown(v) => v
        }
    }
}

impl Default for Ines_Header_F10_TvSystem {
    fn default() -> Self { Ines_Header_F10_TvSystem::Unknown(0) }
}


/**
 * \sa <https://www.nesdev.org/wiki/INES#Flags_6> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines_Header_F6 {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    lower_mapper: RefCell<u64>,
    four_screen: RefCell<bool>,
    trainer: RefCell<bool>,
    has_battery_ram: RefCell<bool>,
    mirroring: RefCell<Ines_Header_F6_Mirroring>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Header_F6 {
    type Root = Ines;
    type Parent = Ines_Header;

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
        *self_rc.lower_mapper.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc.four_screen.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.trainer.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.has_battery_ram.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.mirroring.borrow_mut() = i64::try_from(_io.read_bits_int_be(1)?)?.try_into()?;
        Ok(())
    }
}
impl Ines_Header_F6 {
}

/**
 * Lower nibble of mapper number
 */
impl Ines_Header_F6 {
    pub fn lower_mapper(&self) -> Ref<'_, u64> {
        self.lower_mapper.borrow()
    }
}

/**
 * Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
 */
impl Ines_Header_F6 {
    pub fn four_screen(&self) -> Ref<'_, bool> {
        self.four_screen.borrow()
    }
}

/**
 * 512-byte trainer at $7000-$71FF (stored before PRG data)
 */
impl Ines_Header_F6 {
    pub fn trainer(&self) -> Ref<'_, bool> {
        self.trainer.borrow()
    }
}

/**
 * If on the cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
 */
impl Ines_Header_F6 {
    pub fn has_battery_ram(&self) -> Ref<'_, bool> {
        self.has_battery_ram.borrow()
    }
}

/**
 * if 0, horizontal arrangement. if 1, vertical arrangement
 */
impl Ines_Header_F6 {
    pub fn mirroring(&self) -> Ref<'_, Ines_Header_F6_Mirroring> {
        self.mirroring.borrow()
    }
}
impl Ines_Header_F6 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Ines_Header_F6_Mirroring {
    Horizontal,
    Vertical,
    Unknown(i64),
}

impl TryFrom<i64> for Ines_Header_F6_Mirroring {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Ines_Header_F6_Mirroring> {
        match flag {
            0 => Ok(Ines_Header_F6_Mirroring::Horizontal),
            1 => Ok(Ines_Header_F6_Mirroring::Vertical),
            _ => Ok(Ines_Header_F6_Mirroring::Unknown(flag)),
        }
    }
}

impl From<&Ines_Header_F6_Mirroring> for i64 {
    fn from(v: &Ines_Header_F6_Mirroring) -> Self {
        match *v {
            Ines_Header_F6_Mirroring::Horizontal => 0,
            Ines_Header_F6_Mirroring::Vertical => 1,
            Ines_Header_F6_Mirroring::Unknown(v) => v
        }
    }
}

impl Default for Ines_Header_F6_Mirroring {
    fn default() -> Self { Ines_Header_F6_Mirroring::Unknown(0) }
}


/**
 * \sa <https://www.nesdev.org/wiki/INES#Flags_7> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines_Header_F7 {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    upper_mapper: RefCell<u64>,
    format: RefCell<u64>,
    playchoice10: RefCell<bool>,
    vs_unisystem: RefCell<bool>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Header_F7 {
    type Root = Ines;
    type Parent = Ines_Header;

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
        *self_rc.upper_mapper.borrow_mut() = _io.read_bits_int_be(4)?;
        *self_rc.format.borrow_mut() = _io.read_bits_int_be(2)?;
        *self_rc.playchoice10.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.vs_unisystem.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        Ok(())
    }
}
impl Ines_Header_F7 {
}

/**
 * Upper nibble of mapper number
 */
impl Ines_Header_F7 {
    pub fn upper_mapper(&self) -> Ref<'_, u64> {
        self.upper_mapper.borrow()
    }
}

/**
 * If equal to 2, flags 8-15 are in NES 2.0 format
 */
impl Ines_Header_F7 {
    pub fn format(&self) -> Ref<'_, u64> {
        self.format.borrow()
    }
}

/**
 * Determines if it made for a Nintendo PlayChoice-10 or not
 */
impl Ines_Header_F7 {
    pub fn playchoice10(&self) -> Ref<'_, bool> {
        self.playchoice10.borrow()
    }
}

/**
 * Determines if it is made for a Nintendo VS Unisystem or not
 */
impl Ines_Header_F7 {
    pub fn vs_unisystem(&self) -> Ref<'_, bool> {
        self.vs_unisystem.borrow()
    }
}
impl Ines_Header_F7 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://www.nesdev.org/wiki/INES#Flags_9> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines_Header_F9 {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u64>,
    tv_system: RefCell<Ines_Header_F9_TvSystem>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Header_F9 {
    type Root = Ines;
    type Parent = Ines_Header;

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
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(7)?;
        *self_rc.tv_system.borrow_mut() = i64::try_from(_io.read_bits_int_be(1)?)?.try_into()?;
        Ok(())
    }
}
impl Ines_Header_F9 {
}
impl Ines_Header_F9 {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}

/**
 * if 0, NTSC. If 1, PAL.
 */
impl Ines_Header_F9 {
    pub fn tv_system(&self) -> Ref<'_, Ines_Header_F9_TvSystem> {
        self.tv_system.borrow()
    }
}
impl Ines_Header_F9 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Ines_Header_F9_TvSystem {
    Ntsc,
    Pal,
    Unknown(i64),
}

impl TryFrom<i64> for Ines_Header_F9_TvSystem {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Ines_Header_F9_TvSystem> {
        match flag {
            0 => Ok(Ines_Header_F9_TvSystem::Ntsc),
            1 => Ok(Ines_Header_F9_TvSystem::Pal),
            _ => Ok(Ines_Header_F9_TvSystem::Unknown(flag)),
        }
    }
}

impl From<&Ines_Header_F9_TvSystem> for i64 {
    fn from(v: &Ines_Header_F9_TvSystem) -> Self {
        match *v {
            Ines_Header_F9_TvSystem::Ntsc => 0,
            Ines_Header_F9_TvSystem::Pal => 1,
            Ines_Header_F9_TvSystem::Unknown(v) => v
        }
    }
}

impl Default for Ines_Header_F9_TvSystem {
    fn default() -> Self { Ines_Header_F9_TvSystem::Unknown(0) }
}


/**
 * \sa <https://www.nesdev.org/wiki/PC10_ROM-Images> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ines_Playchoice10 {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines>,
    pub(crate) _self_shared: SharedType<Self>,
    inst_rom: RefCell<Vec<u8>>,
    prom: RefCell<OptRc<Ines_Playchoice10_Prom>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Playchoice10 {
    type Root = Ines;
    type Parent = Ines;

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
        *self_rc.inst_rom.borrow_mut() = _io.read_bytes(8192_usize)?;
        let t = Self::read_into::<_, Ines_Playchoice10_Prom>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.prom.borrow_mut() = t;
        Ok(())
    }
}
impl Ines_Playchoice10 {
}
impl Ines_Playchoice10 {
    pub fn inst_rom(&self) -> Ref<'_, Vec<u8>> {
        self.inst_rom.borrow()
    }
}
impl Ines_Playchoice10 {
    pub fn prom(&self) -> Ref<'_, OptRc<Ines_Playchoice10_Prom>> {
        self.prom.borrow()
    }
}
impl Ines_Playchoice10 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ines_Playchoice10_Prom {
    pub(crate) _root: SharedType<Ines>,
    pub(crate) _parent: SharedType<Ines_Playchoice10>,
    pub(crate) _self_shared: SharedType<Self>,
    data: RefCell<Vec<u8>>,
    counter_out: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ines_Playchoice10_Prom {
    type Root = Ines;
    type Parent = Ines_Playchoice10;

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
        *self_rc.data.borrow_mut() = _io.read_bytes(16_usize)?;
        *self_rc.counter_out.borrow_mut() = _io.read_bytes(16_usize)?;
        Ok(())
    }
}
impl Ines_Playchoice10_Prom {
}
impl Ines_Playchoice10_Prom {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Ines_Playchoice10_Prom {
    pub fn counter_out(&self) -> Ref<'_, Vec<u8>> {
        self.counter_out.borrow()
    }
}
impl Ines_Playchoice10_Prom {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
