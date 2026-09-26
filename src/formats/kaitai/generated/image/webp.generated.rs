// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://developers.google.com/speed/webp/docs/riff_container> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Webp {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    len_data: RefCell<u32>,
    webp: RefCell<Vec<u8>>,
    payload: RefCell<OptRc<Webp_Chunks>>,
    _io: RefCell<BytesReader>,
    payload_raw: RefCell<Vec<u8>>,
}
impl KStruct for Webp {
    type Root = Webp;
    type Parent = Webp;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic() == vec![0x52u8, 0x49u8, 0x46u8, 0x46u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.len_data.borrow_mut() = _io.read_u4le()?;
        *self_rc.webp.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.webp() == vec![0x57u8, 0x45u8, 0x42u8, 0x50u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/2".to_string() }));
        }
        let _raw_payload = _io.read_bytes(usize::try_from((*self_rc.len_data()).saturating_sub(u32::try_from(4_i32)?))?)?;
        *self_rc.payload_raw.borrow_mut() = _raw_payload.clone();
        let _io_payload = BytesReader::from(_raw_payload);
        let t = Self::read_into::<BytesReader, Webp_Chunks>(&_io_payload, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.payload.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp {
}
impl Webp {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Webp {
    pub fn len_data(&self) -> Ref<'_, u32> {
        self.len_data.borrow()
    }
}
impl Webp {
    pub fn webp(&self) -> Ref<'_, Vec<u8>> {
        self.webp.borrow()
    }
}
impl Webp {
    pub fn payload(&self) -> Ref<'_, OptRc<Webp_Chunks>> {
        self.payload.borrow()
    }
}
impl Webp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Webp {
    pub fn payload_raw(&self) -> Ref<'_, Vec<u8>> {
        self.payload_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Webp_ChunkNames {
    XmpVar,
    Vp8,
    Xmp,
    Exif,
    Anmf,
    Alph,
    Vp8l,
    Frgm,
    Anim,
    Iccp,
    Vp8x,
    Unknown(i64),
}

impl TryFrom<i64> for Webp_ChunkNames {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Webp_ChunkNames> {
        match flag {
            5262680 => Ok(Webp_ChunkNames::XmpVar),
            540561494 => Ok(Webp_ChunkNames::Vp8),
            542133592 => Ok(Webp_ChunkNames::Xmp),
            1179211845 => Ok(Webp_ChunkNames::Exif),
            1179471425 => Ok(Webp_ChunkNames::Anmf),
            1213221953 => Ok(Webp_ChunkNames::Alph),
            1278758998 => Ok(Webp_ChunkNames::Vp8l),
            1296519750 => Ok(Webp_ChunkNames::Frgm),
            1296649793 => Ok(Webp_ChunkNames::Anim),
            1346585417 => Ok(Webp_ChunkNames::Iccp),
            1480085590 => Ok(Webp_ChunkNames::Vp8x),
            _ => Ok(Webp_ChunkNames::Unknown(flag)),
        }
    }
}

impl From<&Webp_ChunkNames> for i64 {
    fn from(v: &Webp_ChunkNames) -> Self {
        match *v {
            Webp_ChunkNames::XmpVar => 5262680,
            Webp_ChunkNames::Vp8 => 540561494,
            Webp_ChunkNames::Xmp => 542133592,
            Webp_ChunkNames::Exif => 1179211845,
            Webp_ChunkNames::Anmf => 1179471425,
            Webp_ChunkNames::Alph => 1213221953,
            Webp_ChunkNames::Vp8l => 1278758998,
            Webp_ChunkNames::Frgm => 1296519750,
            Webp_ChunkNames::Anim => 1296649793,
            Webp_ChunkNames::Iccp => 1346585417,
            Webp_ChunkNames::Vp8x => 1480085590,
            Webp_ChunkNames::Unknown(v) => v
        }
    }
}

impl Default for Webp_ChunkNames {
    fn default() -> Self { Webp_ChunkNames::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Webp_CompressionMethod {
    None,
    WebpLossless,
    Unknown(i64),
}

impl TryFrom<i64> for Webp_CompressionMethod {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Webp_CompressionMethod> {
        match flag {
            0 => Ok(Webp_CompressionMethod::None),
            1 => Ok(Webp_CompressionMethod::WebpLossless),
            _ => Ok(Webp_CompressionMethod::Unknown(flag)),
        }
    }
}

impl From<&Webp_CompressionMethod> for i64 {
    fn from(v: &Webp_CompressionMethod) -> Self {
        match *v {
            Webp_CompressionMethod::None => 0,
            Webp_CompressionMethod::WebpLossless => 1,
            Webp_CompressionMethod::Unknown(v) => v
        }
    }
}

impl Default for Webp_CompressionMethod {
    fn default() -> Self { Webp_CompressionMethod::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Webp_FilteringMethod {
    None,
    Horizontal,
    Vertical,
    Gradient,
    Unknown(i64),
}

impl TryFrom<i64> for Webp_FilteringMethod {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Webp_FilteringMethod> {
        match flag {
            0 => Ok(Webp_FilteringMethod::None),
            1 => Ok(Webp_FilteringMethod::Horizontal),
            2 => Ok(Webp_FilteringMethod::Vertical),
            3 => Ok(Webp_FilteringMethod::Gradient),
            _ => Ok(Webp_FilteringMethod::Unknown(flag)),
        }
    }
}

impl From<&Webp_FilteringMethod> for i64 {
    fn from(v: &Webp_FilteringMethod) -> Self {
        match *v {
            Webp_FilteringMethod::None => 0,
            Webp_FilteringMethod::Horizontal => 1,
            Webp_FilteringMethod::Vertical => 2,
            Webp_FilteringMethod::Gradient => 3,
            Webp_FilteringMethod::Unknown(v) => v
        }
    }
}

impl Default for Webp_FilteringMethod {
    fn default() -> Self { Webp_FilteringMethod::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Webp_Preprocessing {
    None,
    LevelReduction,
    Unknown(i64),
}

impl TryFrom<i64> for Webp_Preprocessing {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Webp_Preprocessing> {
        match flag {
            0 => Ok(Webp_Preprocessing::None),
            1 => Ok(Webp_Preprocessing::LevelReduction),
            _ => Ok(Webp_Preprocessing::Unknown(flag)),
        }
    }
}

impl From<&Webp_Preprocessing> for i64 {
    fn from(v: &Webp_Preprocessing) -> Self {
        match *v {
            Webp_Preprocessing::None => 0,
            Webp_Preprocessing::LevelReduction => 1,
            Webp_Preprocessing::Unknown(v) => v
        }
    }
}

impl Default for Webp_Preprocessing {
    fn default() -> Self { Webp_Preprocessing::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Webp_Alph {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved: RefCell<u64>,
    preprocessing: RefCell<Webp_Preprocessing>,
    filtering: RefCell<Webp_FilteringMethod>,
    compression: RefCell<Webp_CompressionMethod>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Alph {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(2)?;
        let expected: u64 = (0).try_into()?;
        if !(*self_rc.reserved() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/alph/seq/0".to_string() }));
        }
        *self_rc.preprocessing.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        if matches!(*self_rc.preprocessing(), Webp_Preprocessing::Unknown(_)) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotInEnum, src_path: "/types/alph/seq/1".to_string() }));
        }
        *self_rc.filtering.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        *self_rc.compression.borrow_mut() = i64::try_from(_io.read_bits_int_be(2)?)?.try_into()?;
        if matches!(*self_rc.compression(), Webp_CompressionMethod::Unknown(_)) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotInEnum, src_path: "/types/alph/seq/3".to_string() }));
        }
        io.align_to_byte()?;
        *self_rc.data.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Alph {
}
impl Webp_Alph {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Webp_Alph {
    pub fn preprocessing(&self) -> Ref<'_, Webp_Preprocessing> {
        self.preprocessing.borrow()
    }
}
impl Webp_Alph {
    pub fn filtering(&self) -> Ref<'_, Webp_FilteringMethod> {
        self.filtering.borrow()
    }
}
impl Webp_Alph {
    pub fn compression(&self) -> Ref<'_, Webp_CompressionMethod> {
        self.compression.borrow()
    }
}
impl Webp_Alph {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Webp_Alph {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://developers.google.com/speed/webp/docs/riff_container#animation> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Webp_Anim {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    background_color: RefCell<OptRc<Webp_Anim_BgColor>>,
    loop_count: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Anim {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        let t = Self::read_into::<_, Webp_Anim_BgColor>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.background_color.borrow_mut() = t;
        *self_rc.loop_count.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Anim {
}
impl Webp_Anim {
    pub fn background_color(&self) -> Ref<'_, OptRc<Webp_Anim_BgColor>> {
        self.background_color.borrow()
    }
}
impl Webp_Anim {
    pub fn loop_count(&self) -> Ref<'_, u16> {
        self.loop_count.borrow()
    }
}
impl Webp_Anim {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Anim_BgColor {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Anim>,
    pub(crate) _self_shared: SharedType<Self>,
    blue: RefCell<u8>,
    green: RefCell<u8>,
    red: RefCell<u8>,
    alpha: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Anim_BgColor {
    type Root = Webp;
    type Parent = Webp_Anim;

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
        *self_rc.blue.borrow_mut() = _io.read_u1()?;
        *self_rc.green.borrow_mut() = _io.read_u1()?;
        *self_rc.red.borrow_mut() = _io.read_u1()?;
        *self_rc.alpha.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Anim_BgColor {
}
impl Webp_Anim_BgColor {
    pub fn blue(&self) -> Ref<'_, u8> {
        self.blue.borrow()
    }
}
impl Webp_Anim_BgColor {
    pub fn green(&self) -> Ref<'_, u8> {
        self.green.borrow()
    }
}
impl Webp_Anim_BgColor {
    pub fn red(&self) -> Ref<'_, u8> {
        self.red.borrow()
    }
}
impl Webp_Anim_BgColor {
    pub fn alpha(&self) -> Ref<'_, u8> {
        self.alpha.borrow()
    }
}
impl Webp_Anim_BgColor {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Anmf {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    frame_x_div_2: RefCell<u64>,
    frame_y_div_2: RefCell<u64>,
    frame_width_minus_1: RefCell<u64>,
    frame_height_minus_1: RefCell<u64>,
    duration: RefCell<u64>,
    reserved: RefCell<u64>,
    blending_method: RefCell<bool>,
    disposal_method: RefCell<bool>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_frame_height: Cell<bool>,
    frame_height: RefCell<u64>,
    f_frame_width: Cell<bool>,
    frame_width: RefCell<u64>,
    f_frame_x: Cell<bool>,
    frame_x: RefCell<u64>,
    f_frame_y: Cell<bool>,
    frame_y: RefCell<u64>,
}
impl KStruct for Webp_Anmf {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.frame_x_div_2.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.frame_y_div_2.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.frame_width_minus_1.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.frame_height_minus_1.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.duration.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.reserved.borrow_mut() = _io.read_bits_int_be(6)?;
        let expected: u64 = (0).try_into()?;
        if !(*self_rc.reserved() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/anmf/seq/5".to_string() }));
        }
        *self_rc.blending_method.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.disposal_method.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        io.align_to_byte()?;
        *self_rc.data.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Anmf {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn frame_height(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_frame_height.get() {
            return Ok(self.frame_height.borrow());
        }
        self.f_frame_height.set(true);
        *self.frame_height.borrow_mut() = ((*self.frame_height_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.frame_height.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn frame_width(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_frame_width.get() {
            return Ok(self.frame_width.borrow());
        }
        self.f_frame_width.set(true);
        *self.frame_width.borrow_mut() = ((*self.frame_width_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.frame_width.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn frame_x(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_frame_x.get() {
            return Ok(self.frame_x.borrow());
        }
        self.f_frame_x.set(true);
        *self.frame_x.borrow_mut() = ((*self.frame_x_div_2()).saturating_mul(2_u64)).try_into()?;
        Ok(self.frame_x.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn frame_y(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_frame_y.get() {
            return Ok(self.frame_y.borrow());
        }
        self.f_frame_y.set(true);
        *self.frame_y.borrow_mut() = ((*self.frame_y_div_2()).saturating_mul(2_u64)).try_into()?;
        Ok(self.frame_y.borrow())
    }
}
impl Webp_Anmf {
    pub fn frame_x_div_2(&self) -> Ref<'_, u64> {
        self.frame_x_div_2.borrow()
    }
}
impl Webp_Anmf {
    pub fn frame_y_div_2(&self) -> Ref<'_, u64> {
        self.frame_y_div_2.borrow()
    }
}
impl Webp_Anmf {
    pub fn frame_width_minus_1(&self) -> Ref<'_, u64> {
        self.frame_width_minus_1.borrow()
    }
}
impl Webp_Anmf {
    pub fn frame_height_minus_1(&self) -> Ref<'_, u64> {
        self.frame_height_minus_1.borrow()
    }
}
impl Webp_Anmf {
    pub fn duration(&self) -> Ref<'_, u64> {
        self.duration.borrow()
    }
}
impl Webp_Anmf {
    pub fn reserved(&self) -> Ref<'_, u64> {
        self.reserved.borrow()
    }
}
impl Webp_Anmf {
    pub fn blending_method(&self) -> Ref<'_, bool> {
        self.blending_method.borrow()
    }
}
impl Webp_Anmf {
    pub fn disposal_method(&self) -> Ref<'_, bool> {
        self.disposal_method.borrow()
    }
}
impl Webp_Anmf {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Webp_Anmf {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Chunk {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunks>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<Webp_ChunkNames>,
    len_data: RefCell<u32>,
    data: RefCell<Option<Webp_Chunk_Data>>,
    padding: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    data_raw: RefCell<Vec<u8>>,
}
#[derive(Debug, Clone)]
pub enum Webp_Chunk_Data {
    Webp_Alph(OptRc<Webp_Alph>),
    Webp_Anim(OptRc<Webp_Anim>),
    Webp_Anmf(OptRc<Webp_Anmf>),
    Webp_Vp8(OptRc<Webp_Vp8>),
    Webp_Vp8l(OptRc<Webp_Vp8l>),
    Webp_Vp8x(OptRc<Webp_Vp8x>),
    Webp_Xmp(OptRc<Webp_Xmp>),
    Bytes(Vec<u8>),
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Alph> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Alph(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Alph>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Alph>) -> Self {
        Self::Webp_Alph(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Anim> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Anim(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Anim>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Anim>) -> Self {
        Self::Webp_Anim(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Anmf> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Anmf(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Anmf>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Anmf>) -> Self {
        Self::Webp_Anmf(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Vp8> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Vp8(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Vp8>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Vp8>) -> Self {
        Self::Webp_Vp8(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Vp8l> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Vp8l(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Vp8l>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Vp8l>) -> Self {
        Self::Webp_Vp8l(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Vp8x> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Vp8x(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Vp8x>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Vp8x>) -> Self {
        Self::Webp_Vp8x(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for OptRc<Webp_Xmp> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Webp_Xmp(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<OptRc<Webp_Xmp>> for Webp_Chunk_Data {
    fn from(v: OptRc<Webp_Xmp>) -> Self {
        Self::Webp_Xmp(v)
    }
}
impl TryFrom<&Webp_Chunk_Data> for Vec<u8> {
    type Error = KError;
    fn try_from(v: &Webp_Chunk_Data) -> Result<Self, Self::Error> {
        if let Webp_Chunk_Data::Bytes(x) = v {
            return Ok(x.clone());
        }
        Err(KError::CastError)
    }
}
impl From<Vec<u8>> for Webp_Chunk_Data {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}
impl KStruct for Webp_Chunk {
    type Root = Webp;
    type Parent = Webp_Chunks;

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
        *self_rc.name.borrow_mut() = i64::from(_io.read_u4le()?).try_into()?;
        if matches!(*self_rc.name(), Webp_ChunkNames::Unknown(_)) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotInEnum, src_path: "/types/chunk/seq/0".to_string() }));
        }
        *self_rc.len_data.borrow_mut() = _io.read_u4le()?;
        match *self_rc.name() {
            Webp_ChunkNames::Alph => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Alph>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Anim => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Anim>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Anmf => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Anmf>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Vp8 => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Vp8>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Vp8l => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Vp8l>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Vp8x => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Vp8x>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::Xmp => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Xmp>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            Webp_ChunkNames::XmpVar => {
                *self_rc.data_raw.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_data())?)?.into();
                let data_raw = self_rc.data_raw.borrow();
                let _t_data_raw_io = BytesReader::from(data_raw.clone());
                let t = Self::read_into::<BytesReader, Webp_Xmp>(&_t_data_raw_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                *self_rc.data.borrow_mut() = Some(t);
            }
            _ => {
                *self_rc.data.borrow_mut() = Some(_io.read_bytes_full()?.into());
            }
        }
        if ((to_i128((*self_rc.len_data()).checked_rem(2_u32).ok_or(KError::CastError)?)) != (to_i128(0))) {
            *self_rc.padding.borrow_mut() = _io.read_bytes(1_usize)?;
            if !(*self_rc.padding() == vec![0x0u8]) {
                return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/chunk/seq/3".to_string() }));
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Chunk {
}
impl Webp_Chunk {
    pub fn name(&self) -> Ref<'_, Webp_ChunkNames> {
        self.name.borrow()
    }
}
impl Webp_Chunk {
    pub fn len_data(&self) -> Ref<'_, u32> {
        self.len_data.borrow()
    }
}
impl Webp_Chunk {
    pub fn data(&self) -> Ref<'_, Option<Webp_Chunk_Data>> {
        self.data.borrow()
    }
}
impl Webp_Chunk {
    pub fn padding(&self) -> Ref<'_, Vec<u8>> {
        self.padding.borrow()
    }
}
impl Webp_Chunk {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Webp_Chunk {
    pub fn data_raw(&self) -> Ref<'_, Vec<u8>> {
        self.data_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Chunks {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp>,
    pub(crate) _self_shared: SharedType<Self>,
    chunks: RefCell<Vec<OptRc<Webp_Chunk>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Chunks {
    type Root = Webp;
    type Parent = Webp;

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
        *self_rc.chunks.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, Webp_Chunk>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.chunks.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Chunks {
}
impl Webp_Chunks {
    pub fn chunks(&self) -> Ref<'_, Vec<OptRc<Webp_Chunk>>> {
        self.chunks.borrow()
    }
}
impl Webp_Chunks {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://www.rfc-editor.org/rfc/rfc6386#section-9.1> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Webp_Vp8 {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    frame_type: RefCell<bool>,
    version: RefCell<u64>,
    show_frame: RefCell<bool>,
    len_first_partition: RefCell<u64>,
    start_code: RefCell<Vec<u8>>,
    width: RefCell<u64>,
    horizontal_scale: RefCell<u64>,
    height: RefCell<u64>,
    vertical_scale: RefCell<u64>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Vp8 {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.frame_type.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        if !(*self_rc.frame_type() == false) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8/seq/0".to_string() }));
        }
        *self_rc.version.borrow_mut() = _io.read_bits_int_le(3)?;
        let max_val: u64 = (3).try_into()?;
        if !(*self_rc.version() <= max_val) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::GreaterThan, src_path: "/types/vp8/seq/1".to_string() }));
        }
        *self_rc.show_frame.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.len_first_partition.borrow_mut() = _io.read_bits_int_le(19)?;
        io.align_to_byte()?;
        *self_rc.start_code.borrow_mut() = _io.read_bytes(3_usize)?;
        if !(*self_rc.start_code() == vec![0x9du8, 0x1u8, 0x2au8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8/seq/4".to_string() }));
        }
        *self_rc.width.borrow_mut() = _io.read_bits_int_le(14)?;
        *self_rc.horizontal_scale.borrow_mut() = _io.read_bits_int_le(2)?;
        *self_rc.height.borrow_mut() = _io.read_bits_int_le(14)?;
        *self_rc.vertical_scale.borrow_mut() = _io.read_bits_int_le(2)?;
        io.align_to_byte()?;
        *self_rc.data.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Vp8 {
}
impl Webp_Vp8 {
    pub fn frame_type(&self) -> Ref<'_, bool> {
        self.frame_type.borrow()
    }
}
impl Webp_Vp8 {
    pub fn version(&self) -> Ref<'_, u64> {
        self.version.borrow()
    }
}
impl Webp_Vp8 {
    pub fn show_frame(&self) -> Ref<'_, bool> {
        self.show_frame.borrow()
    }
}
impl Webp_Vp8 {
    pub fn len_first_partition(&self) -> Ref<'_, u64> {
        self.len_first_partition.borrow()
    }
}
impl Webp_Vp8 {
    pub fn start_code(&self) -> Ref<'_, Vec<u8>> {
        self.start_code.borrow()
    }
}
impl Webp_Vp8 {
    pub fn width(&self) -> Ref<'_, u64> {
        self.width.borrow()
    }
}
impl Webp_Vp8 {
    pub fn horizontal_scale(&self) -> Ref<'_, u64> {
        self.horizontal_scale.borrow()
    }
}
impl Webp_Vp8 {
    pub fn height(&self) -> Ref<'_, u64> {
        self.height.borrow()
    }
}
impl Webp_Vp8 {
    pub fn vertical_scale(&self) -> Ref<'_, u64> {
        self.vertical_scale.borrow()
    }
}
impl Webp_Vp8 {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Webp_Vp8 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

/**
 * \sa <https://developers.google.com/speed/webp/docs/webp_lossless_bitstream_specification> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Webp_Vp8l {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    signature: RefCell<u8>,
    image_width_minus_1: RefCell<u64>,
    image_height_minus_1: RefCell<u64>,
    alpha_is_used: RefCell<bool>,
    version_number: RefCell<u64>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_image_height: Cell<bool>,
    image_height: RefCell<u64>,
    f_image_width: Cell<bool>,
    image_width: RefCell<u64>,
}
impl KStruct for Webp_Vp8l {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.signature.borrow_mut() = _io.read_u1()?;
        let expected: u8 = (47).try_into()?;
        if !(*self_rc.signature() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8l/seq/0".to_string() }));
        }
        *self_rc.image_width_minus_1.borrow_mut() = _io.read_bits_int_le(14)?;
        *self_rc.image_height_minus_1.borrow_mut() = _io.read_bits_int_le(14)?;
        *self_rc.alpha_is_used.borrow_mut() = _io.read_bits_int_le(1)? != 0;
        *self_rc.version_number.borrow_mut() = _io.read_bits_int_le(3)?;
        let expected: u64 = (0).try_into()?;
        if !(*self_rc.version_number() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8l/seq/4".to_string() }));
        }
        io.align_to_byte()?;
        *self_rc.data.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Vp8l {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn image_height(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_image_height.get() {
            return Ok(self.image_height.borrow());
        }
        self.f_image_height.set(true);
        *self.image_height.borrow_mut() = ((*self.image_height_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.image_height.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn image_width(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_image_width.get() {
            return Ok(self.image_width.borrow());
        }
        self.f_image_width.set(true);
        *self.image_width.borrow_mut() = ((*self.image_width_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.image_width.borrow())
    }
}
impl Webp_Vp8l {
    pub fn signature(&self) -> Ref<'_, u8> {
        self.signature.borrow()
    }
}
impl Webp_Vp8l {
    pub fn image_width_minus_1(&self) -> Ref<'_, u64> {
        self.image_width_minus_1.borrow()
    }
}
impl Webp_Vp8l {
    pub fn image_height_minus_1(&self) -> Ref<'_, u64> {
        self.image_height_minus_1.borrow()
    }
}

/**
 * A hint only - it should not impact decoding. It should be `false` when
 * all alpha values are 255 in the picture, and `true` otherwise.
 */
impl Webp_Vp8l {
    pub fn alpha_is_used(&self) -> Ref<'_, bool> {
        self.alpha_is_used.borrow()
    }
}
impl Webp_Vp8l {
    pub fn version_number(&self) -> Ref<'_, u64> {
        self.version_number.borrow()
    }
}
impl Webp_Vp8l {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl Webp_Vp8l {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Vp8x {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    reserved1: RefCell<u64>,
    icc_profile: RefCell<bool>,
    alpha: RefCell<bool>,
    exif: RefCell<bool>,
    xmp: RefCell<bool>,
    animation: RefCell<bool>,
    reserved2: RefCell<bool>,
    reserved3: RefCell<u64>,
    canvas_width_minus_1: RefCell<u64>,
    canvas_height_minus_1: RefCell<u64>,
    _io: RefCell<BytesReader>,
    f_canvas_height: Cell<bool>,
    canvas_height: RefCell<u64>,
    f_canvas_width: Cell<bool>,
    canvas_width: RefCell<u64>,
}
impl KStruct for Webp_Vp8x {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.reserved1.borrow_mut() = _io.read_bits_int_be(2)?;
        let expected: u64 = (0).try_into()?;
        if !(*self_rc.reserved1() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8x/seq/0".to_string() }));
        }
        *self_rc.icc_profile.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.alpha.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.exif.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.xmp.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.animation.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.reserved2.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        if !(*self_rc.reserved2() == false) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8x/seq/6".to_string() }));
        }
        *self_rc.reserved3.borrow_mut() = _io.read_bits_int_be(24)?;
        let expected: u64 = (0).try_into()?;
        if !(*self_rc.reserved3() == expected) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/vp8x/seq/7".to_string() }));
        }
        *self_rc.canvas_width_minus_1.borrow_mut() = _io.read_bits_int_le(24)?;
        *self_rc.canvas_height_minus_1.borrow_mut() = _io.read_bits_int_le(24)?;
        let max_val: u64 = (((4294967295_u64).checked_div(*self_rc.canvas_width()?).ok_or(KError::CastError)?).saturating_sub(1_u64)).try_into()?;
        if !(*self_rc.canvas_height_minus_1() <= max_val) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::GreaterThan, src_path: "/types/vp8x/seq/9".to_string() }));
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Vp8x {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn canvas_height(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_canvas_height.get() {
            return Ok(self.canvas_height.borrow());
        }
        self.f_canvas_height.set(true);
        *self.canvas_height.borrow_mut() = ((*self.canvas_height_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.canvas_height.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn canvas_width(
        &self
    ) -> KResult<Ref<'_, u64>> {
        let _io = self._io.borrow();
        if self.f_canvas_width.get() {
            return Ok(self.canvas_width.borrow());
        }
        self.f_canvas_width.set(true);
        *self.canvas_width.borrow_mut() = ((*self.canvas_width_minus_1()).saturating_add(1_u64)).try_into()?;
        Ok(self.canvas_width.borrow())
    }
}
impl Webp_Vp8x {
    pub fn reserved1(&self) -> Ref<'_, u64> {
        self.reserved1.borrow()
    }
}
impl Webp_Vp8x {
    pub fn icc_profile(&self) -> Ref<'_, bool> {
        self.icc_profile.borrow()
    }
}
impl Webp_Vp8x {
    pub fn alpha(&self) -> Ref<'_, bool> {
        self.alpha.borrow()
    }
}
impl Webp_Vp8x {
    pub fn exif(&self) -> Ref<'_, bool> {
        self.exif.borrow()
    }
}
impl Webp_Vp8x {
    pub fn xmp(&self) -> Ref<'_, bool> {
        self.xmp.borrow()
    }
}
impl Webp_Vp8x {
    pub fn animation(&self) -> Ref<'_, bool> {
        self.animation.borrow()
    }
}
impl Webp_Vp8x {
    pub fn reserved2(&self) -> Ref<'_, bool> {
        self.reserved2.borrow()
    }
}
impl Webp_Vp8x {
    pub fn reserved3(&self) -> Ref<'_, u64> {
        self.reserved3.borrow()
    }
}
impl Webp_Vp8x {
    pub fn canvas_width_minus_1(&self) -> Ref<'_, u64> {
        self.canvas_width_minus_1.borrow()
    }
}
impl Webp_Vp8x {
    pub fn canvas_height_minus_1(&self) -> Ref<'_, u64> {
        self.canvas_height_minus_1.borrow()
    }
}
impl Webp_Vp8x {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Webp_Xmp {
    pub(crate) _root: SharedType<Webp>,
    pub(crate) _parent: SharedType<Webp_Chunk>,
    pub(crate) _self_shared: SharedType<Self>,
    data: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Webp_Xmp {
    type Root = Webp;
    type Parent = Webp_Chunk;

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
        *self_rc.data.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Webp_Xmp {
}
impl Webp_Xmp {
    pub fn data(&self) -> Ref<'_, String> {
        self.data.borrow()
    }
}
impl Webp_Xmp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
