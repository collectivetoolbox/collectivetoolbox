// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * TAP files are used by emulators of ZX Spectrum computer (released in
 * 1982 by Sinclair Research). TAP file stores blocks of data as if
 * they are written to magnetic tape, which was used as primary media
 * for ZX Spectrum. Contents of this file can be viewed as a very
 * simple linear filesystem, storing named files with some basic
 * metainformation prepended as a header.
 * \sa <https://sinclair.wiki.zxnet.co.uk/wiki/TAP_format> Source
 */

#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap>,
    pub(crate) _self_shared: SharedType<Self>,
    blocks: RefCell<Vec<OptRc<ZxSpectrumTap_Block>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ZxSpectrumTap {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap;

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
        *self_rc.blocks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, ZxSpectrumTap_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.blocks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap {
}
impl ZxSpectrumTap {
    pub fn blocks(&self) -> Ref<'_, Vec<OptRc<ZxSpectrumTap_Block>>> {
        self.blocks.borrow()
    }
}
impl ZxSpectrumTap {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ZxSpectrumTap_FlagEnum {
    Header,
    Data,
    Unknown(i64),
}

impl TryFrom<i64> for ZxSpectrumTap_FlagEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ZxSpectrumTap_FlagEnum> {
        match flag {
            0 => Ok(ZxSpectrumTap_FlagEnum::Header),
            255 => Ok(ZxSpectrumTap_FlagEnum::Data),
            _ => Ok(ZxSpectrumTap_FlagEnum::Unknown(flag)),
        }
    }
}

impl From<&ZxSpectrumTap_FlagEnum> for i64 {
    fn from(v: &ZxSpectrumTap_FlagEnum) -> Self {
        match *v {
            ZxSpectrumTap_FlagEnum::Header => 0,
            ZxSpectrumTap_FlagEnum::Data => 255,
            ZxSpectrumTap_FlagEnum::Unknown(v) => v
        }
    }
}

impl Default for ZxSpectrumTap_FlagEnum {
    fn default() -> Self { ZxSpectrumTap_FlagEnum::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ZxSpectrumTap_HeaderTypeEnum {
    Program,
    NumArray,
    CharArray,
    Bytes,
    Unknown(i64),
}

impl TryFrom<i64> for ZxSpectrumTap_HeaderTypeEnum {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<ZxSpectrumTap_HeaderTypeEnum> {
        match flag {
            0 => Ok(ZxSpectrumTap_HeaderTypeEnum::Program),
            1 => Ok(ZxSpectrumTap_HeaderTypeEnum::NumArray),
            2 => Ok(ZxSpectrumTap_HeaderTypeEnum::CharArray),
            3 => Ok(ZxSpectrumTap_HeaderTypeEnum::Bytes),
            _ => Ok(ZxSpectrumTap_HeaderTypeEnum::Unknown(flag)),
        }
    }
}

impl From<&ZxSpectrumTap_HeaderTypeEnum> for i64 {
    fn from(v: &ZxSpectrumTap_HeaderTypeEnum) -> Self {
        match *v {
            ZxSpectrumTap_HeaderTypeEnum::Program => 0,
            ZxSpectrumTap_HeaderTypeEnum::NumArray => 1,
            ZxSpectrumTap_HeaderTypeEnum::CharArray => 2,
            ZxSpectrumTap_HeaderTypeEnum::Bytes => 3,
            ZxSpectrumTap_HeaderTypeEnum::Unknown(v) => v
        }
    }
}

impl Default for ZxSpectrumTap_HeaderTypeEnum {
    fn default() -> Self { ZxSpectrumTap_HeaderTypeEnum::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap_ArrayParams {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u8>,
    var_name: RefCell<u8>,
    reserved1: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ZxSpectrumTap_ArrayParams {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap_Header;

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
        *self_rc.reserved.borrow_mut() = _io.read_u1()?;
        *self_rc.var_name.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved1.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.reserved1() == vec![0x0u8, 0x80u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/array_params/seq/2".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap_ArrayParams {
}
impl ZxSpectrumTap_ArrayParams {
    pub fn reserved(&self) -> Ref<'_, u8> {
        self.reserved.borrow()
    }
}

/**
 * Variable name (1..26 meaning A$..Z$ +192)
 */
impl ZxSpectrumTap_ArrayParams {
    pub fn var_name(&self) -> Ref<'_, u8> {
        self.var_name.borrow()
    }
}
impl ZxSpectrumTap_ArrayParams {
    pub fn reserved1(&self) -> Ref<'_, Vec<u8>> {
        self.reserved1.borrow()
    }
}
impl ZxSpectrumTap_ArrayParams {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap_Block {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap>,
    pub(crate) _self_shared: SharedType<Self>,
    len_block: RefCell<u16>,
    flag: RefCell<ZxSpectrumTap_FlagEnum>,
    header: RefCell<OptRc<ZxSpectrumTap_Header>>,
    data: RefCell<Vec<u8>>,
    headerless_data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
    headerless_data_raw: RefCell<Vec<u8>>,
}
impl KStruct for ZxSpectrumTap_Block {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap;

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
        *self_rc.len_block.borrow_mut() = _io.read_u2le()?;
        *self_rc.flag.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        if  ((((to_i128(*self_rc.len_block())) == (to_i128(19)))) && (*self_rc.flag() == ZxSpectrumTap_FlagEnum::Header))  {
            let t = Self::read_into::<_, ZxSpectrumTap_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.header.borrow_mut() = t;
        }
        if ((to_i128(*self_rc.len_block())) == (to_i128(19))) {
            *self_rc.data.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.header().len_data())).saturating_add(4_i32))?)?;
        }
        if *self_rc.flag() == ZxSpectrumTap_FlagEnum::Data {
            *self_rc.headerless_data.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.len_block())).saturating_sub(1_i32))?)?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap_Block {
}
impl ZxSpectrumTap_Block {
    pub fn len_block(&self) -> Ref<'_, u16> {
        self.len_block.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn flag(&self) -> Ref<'_, ZxSpectrumTap_FlagEnum> {
        self.flag.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn header(&self) -> Ref<'_, OptRc<ZxSpectrumTap_Header>> {
        self.header.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn headerless_data(&self) -> Ref<'_, Vec<u8>> {
        self.headerless_data.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}
impl ZxSpectrumTap_Block {
    pub fn headerless_data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.headerless_data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap_BytesParams {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    start_address: RefCell<u16>,
    reserved: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    reserved_raw: RefCell<Vec<u8>>,
}
impl KStruct for ZxSpectrumTap_BytesParams {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap_Header;

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
        *self_rc.start_address.borrow_mut() = _io.read_u2le()?;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(2_usize)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap_BytesParams {
}
impl ZxSpectrumTap_BytesParams {
    pub fn start_address(&self) -> Ref<'_, u16> {
        self.start_address.borrow()
    }
}
impl ZxSpectrumTap_BytesParams {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl ZxSpectrumTap_BytesParams {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ZxSpectrumTap_BytesParams {
    pub fn reserved_raw(&self) -> Ref<'_, Vec<u8>> {
        self.reserved_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap_Header {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap_Block>,
    pub(crate) _self_shared: SharedType<Self>,
    header_type: RefCell<ZxSpectrumTap_HeaderTypeEnum>,
    filename: RefCell<Vec<u8>>,
    len_data: RefCell<u16>,
    params: RefCell<Option<ZxSpectrumTap_Header_Params>>,
    checksum: RefCell<u8>,
    _io: RefCell<BytesReader>,
    filename_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum ZxSpectrumTap_Header_Params {
    ZxSpectrumTap_BytesParams(OptRc<ZxSpectrumTap_BytesParams>),
    ZxSpectrumTap_ArrayParams(OptRc<ZxSpectrumTap_ArrayParams>),
    ZxSpectrumTap_ProgramParams(OptRc<ZxSpectrumTap_ProgramParams>),
}
impl TryFrom<&ZxSpectrumTap_Header_Params> for OptRc<ZxSpectrumTap_BytesParams> {
    type Error = KError;
    fn try_from(v: &ZxSpectrumTap_Header_Params) -> Result<Self, Self::Error> {
        if let ZxSpectrumTap_Header_Params::ZxSpectrumTap_BytesParams(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<ZxSpectrumTap_BytesParams>> for ZxSpectrumTap_Header_Params {
    fn from(v: OptRc<ZxSpectrumTap_BytesParams>) -> Self {
        Self::ZxSpectrumTap_BytesParams(v)
    }
}
impl TryFrom<&ZxSpectrumTap_Header_Params> for OptRc<ZxSpectrumTap_ArrayParams> {
    type Error = KError;
    fn try_from(v: &ZxSpectrumTap_Header_Params) -> Result<Self, Self::Error> {
        if let ZxSpectrumTap_Header_Params::ZxSpectrumTap_ArrayParams(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<ZxSpectrumTap_ArrayParams>> for ZxSpectrumTap_Header_Params {
    fn from(v: OptRc<ZxSpectrumTap_ArrayParams>) -> Self {
        Self::ZxSpectrumTap_ArrayParams(v)
    }
}
impl TryFrom<&ZxSpectrumTap_Header_Params> for OptRc<ZxSpectrumTap_ProgramParams> {
    type Error = KError;
    fn try_from(v: &ZxSpectrumTap_Header_Params) -> Result<Self, Self::Error> {
        if let ZxSpectrumTap_Header_Params::ZxSpectrumTap_ProgramParams(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<ZxSpectrumTap_ProgramParams>> for ZxSpectrumTap_Header_Params {
    fn from(v: OptRc<ZxSpectrumTap_ProgramParams>) -> Self {
        Self::ZxSpectrumTap_ProgramParams(v)
    }
}
impl KStruct for ZxSpectrumTap_Header {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap_Block;

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
        *self_rc.header_type.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.filename.borrow_mut() = bytes_terminate_pad(&_io.read_bytes(10_usize)?, None, false, Some(32));
        *self_rc.len_data.borrow_mut() = _io.read_u2le()?;
        match *self_rc.header_type() {
            ZxSpectrumTap_HeaderTypeEnum::Bytes => {
                let t = Self::read_into::<_, ZxSpectrumTap_BytesParams>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.params.borrow_mut() = Some(t);
            }
            ZxSpectrumTap_HeaderTypeEnum::CharArray => {
                let t = Self::read_into::<_, ZxSpectrumTap_ArrayParams>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.params.borrow_mut() = Some(t);
            }
            ZxSpectrumTap_HeaderTypeEnum::NumArray => {
                let t = Self::read_into::<_, ZxSpectrumTap_ArrayParams>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.params.borrow_mut() = Some(t);
            }
            ZxSpectrumTap_HeaderTypeEnum::Program => {
                let t = Self::read_into::<_, ZxSpectrumTap_ProgramParams>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.params.borrow_mut() = Some(t);
            }
            _ => {}
        }
        *self_rc.checksum.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap_Header {
}
impl ZxSpectrumTap_Header {
    pub fn header_type(&self) -> Ref<'_, ZxSpectrumTap_HeaderTypeEnum> {
        self.header_type.borrow()
    }
}
impl ZxSpectrumTap_Header {
    pub fn filename(&self) -> Ref<'_, Vec<u8>> {
        self.filename.borrow()
    }
}
impl ZxSpectrumTap_Header {
    pub fn len_data(&self) -> Ref<'_, u16> {
        self.len_data.borrow()
    }
}
impl ZxSpectrumTap_Header {
    pub fn params(&self) -> Ref<'_, Option<ZxSpectrumTap_Header_Params>> {
        self.params.borrow()
    }
}

/**
 * Bitwise XOR of all bytes including the flag byte
 */
impl ZxSpectrumTap_Header {
    pub fn checksum(&self) -> Ref<'_, u8> {
        self.checksum.borrow()
    }
}
impl ZxSpectrumTap_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ZxSpectrumTap_Header {
    pub fn filename_raw(&self) -> Ref<'_, Vec<u8>> {
        self.filename_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZxSpectrumTap_ProgramParams {
    pub(crate) _root: SharedType<ZxSpectrumTap>,
    pub(crate) _parent: SharedType<ZxSpectrumTap_Header>,
    pub(crate) _self_shared: SharedType<Self>,
    autostart_line: RefCell<u16>,
    len_program: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for ZxSpectrumTap_ProgramParams {
    type Root = ZxSpectrumTap;
    type Parent = ZxSpectrumTap_Header;

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
        *self_rc.autostart_line.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_program.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ZxSpectrumTap_ProgramParams {
}
impl ZxSpectrumTap_ProgramParams {
    pub fn autostart_line(&self) -> Ref<'_, u16> {
        self.autostart_line.borrow()
    }
}
impl ZxSpectrumTap_ProgramParams {
    pub fn len_program(&self) -> Ref<'_, u16> {
        self.len_program.borrow()
    }
}
impl ZxSpectrumTap_ProgramParams {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
