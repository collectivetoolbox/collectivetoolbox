// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * PCX is a bitmap image format originally used by PC Paintbrush from
 * ZSoft Corporation. Originally, it was a relatively simple 128-byte
 * header + uncompressed bitmap format, but latest versions introduced
 * more complicated palette support and RLE compression.
 *
 * There's an option to encode 32-bit or 16-bit RGBA pixels, and thus
 * it can potentially include transparency. Theoretically, it's
 * possible to encode resolution or pixel density in the some of the
 * header fields too, but in reality there's no uniform standard for
 * these, so different implementations treat these differently.
 *
 * PCX format was never made a formal standard. "ZSoft Corporation
 * Technical Reference Manual" for "Image File (.PCX) Format", last
 * updated in 1991, is likely the closest authoritative source.
 * \sa <https://web.archive.org/web/20100206055706/http://www.qzx.com/pc-gpe/pcx.txt> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Pcx {
    pub(crate) _root: SharedType<Pcx>,
    pub(crate) _parent: SharedType<Pcx>,
    pub(crate) _self_shared: SharedType<Self>,
    hdr: RefCell<OptRc<Pcx_Header>>,
    _io: RefCell<BytesReader>,
    hdr_raw: RefCell<Vec<u8>>,
    f_palette_256: Cell<bool>,
    palette_256: RefCell<OptRc<Pcx_TPalette256>>,
}
impl KStruct for Pcx {
    type Root = Pcx;
    type Parent = Pcx;

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
        let _raw_hdr = _io.read_bytes(128_usize)?;
        *self_rc.hdr_raw.borrow_mut() = _raw_hdr.clone();
        let _io_hdr = BytesReader::from(_raw_hdr);
        let t = Self::read_into::<BytesReader, Pcx_Header>(&_io_hdr, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.hdr.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Pcx {

    /**
     * \sa https://web.archive.org/web/20100206055706/http://www.qzx.com/pc-gpe/pcx.txt - "VGA 256 Color Palette Information"
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn palette_256(
        &self
    ) -> KResult<Ref<'_, OptRc<Pcx_TPalette256>>> {
        let _io = self._io.borrow();
        if self.f_palette_256.get() {
            return Ok(self.palette_256.borrow());
        }
        if  ((*self.hdr().version() == Pcx_Versions::V30) && (((to_i128(*self.hdr().bits_per_pixel())) == (to_i128(8)))) && (((to_i128(*self.hdr().num_planes())) == (to_i128(1)))))  {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((i64::try_from(_io.size())?)).saturating_sub(769_i32))?)?;
            let t = Self::read_into::<_, Pcx_TPalette256>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            *self.palette_256.borrow_mut() = t;
            _io.seek(_pos)?;
        }
        Ok(self.palette_256.borrow())
    }
}
impl Pcx {
    pub fn hdr(&self) -> Ref<'_, OptRc<Pcx_Header>> {
        self.hdr.borrow()
    }
}
impl Pcx {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Pcx {
    pub fn hdr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.hdr_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Pcx_Encodings {
    Rle,
    Unknown(i64),
}

impl TryFrom<i64> for Pcx_Encodings {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Pcx_Encodings> {
        match flag {
            1 => Ok(Pcx_Encodings::Rle),
            _ => Ok(Pcx_Encodings::Unknown(flag)),
        }
    }
}

impl From<&Pcx_Encodings> for i64 {
    fn from(v: &Pcx_Encodings) -> Self {
        match *v {
            Pcx_Encodings::Rle => 1,
            Pcx_Encodings::Unknown(v) => v
        }
    }
}

impl Default for Pcx_Encodings {
    fn default() -> Self { Pcx_Encodings::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Pcx_Versions {
    V25,
    V28WithPalette,
    V28WithoutPalette,
    PaintbrushForWindows,
    V30,
    Unknown(i64),
}

impl TryFrom<i64> for Pcx_Versions {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Pcx_Versions> {
        match flag {
            0 => Ok(Pcx_Versions::V25),
            2 => Ok(Pcx_Versions::V28WithPalette),
            3 => Ok(Pcx_Versions::V28WithoutPalette),
            4 => Ok(Pcx_Versions::PaintbrushForWindows),
            5 => Ok(Pcx_Versions::V30),
            _ => Ok(Pcx_Versions::Unknown(flag)),
        }
    }
}

impl From<&Pcx_Versions> for i64 {
    fn from(v: &Pcx_Versions) -> Self {
        match *v {
            Pcx_Versions::V25 => 0,
            Pcx_Versions::V28WithPalette => 2,
            Pcx_Versions::V28WithoutPalette => 3,
            Pcx_Versions::PaintbrushForWindows => 4,
            Pcx_Versions::V30 => 5,
            Pcx_Versions::Unknown(v) => v
        }
    }
}

impl Default for Pcx_Versions {
    fn default() -> Self { Pcx_Versions::Unknown(0) }
}


/**
 * \sa https://web.archive.org/web/20100206055706/http://www.qzx.com/pc-gpe/pcx.txt - "ZSoft .PCX FILE HEADER FORMAT"
 */

#[derive(Default, Debug, Clone)]
pub struct Pcx_Header {
    pub(crate) _root: SharedType<Pcx>,
    pub(crate) _parent: SharedType<Pcx>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    version: RefCell<Pcx_Versions>,
    encoding: RefCell<Pcx_Encodings>,
    bits_per_pixel: RefCell<u8>,
    img_x_min: RefCell<u16>,
    img_y_min: RefCell<u16>,
    img_x_max: RefCell<u16>,
    img_y_max: RefCell<u16>,
    hdpi: RefCell<u16>,
    vdpi: RefCell<u16>,
    palette_16: RefCell<Vec<u8>>,
    reserved: RefCell<Vec<u8>>,
    num_planes: RefCell<u8>,
    bytes_per_line: RefCell<u16>,
    palette_info: RefCell<u16>,
    h_screen_size: RefCell<u16>,
    v_screen_size: RefCell<u16>,
    _io: RefCell<BytesReader>,
    palette_16_raw: RefCell<Vec<u8>>,
}
impl KStruct for Pcx_Header {
    type Root = Pcx;
    type Parent = Pcx;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.magic() == vec![0xau8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/0".to_string() }));
        }
        *self_rc.version.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.encoding.borrow_mut() = i64::from(_io.read_u1()?).try_into()?;
        *self_rc.bits_per_pixel.borrow_mut() = _io.read_u1()?;
        *self_rc.img_x_min.borrow_mut() = _io.read_u2le()?;
        *self_rc.img_y_min.borrow_mut() = _io.read_u2le()?;
        *self_rc.img_x_max.borrow_mut() = _io.read_u2le()?;
        *self_rc.img_y_max.borrow_mut() = _io.read_u2le()?;
        *self_rc.hdpi.borrow_mut() = _io.read_u2le()?;
        *self_rc.vdpi.borrow_mut() = _io.read_u2le()?;
        *self_rc.palette_16.borrow_mut() = _io.read_bytes(48_usize)?;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.reserved() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/11".to_string() }));
        }
        *self_rc.num_planes.borrow_mut() = _io.read_u1()?;
        *self_rc.bytes_per_line.borrow_mut() = _io.read_u2le()?;
        *self_rc.palette_info.borrow_mut() = _io.read_u2le()?;
        *self_rc.h_screen_size.borrow_mut() = _io.read_u2le()?;
        *self_rc.v_screen_size.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Pcx_Header {
}

/**
 * Technically, this field was supposed to be "manufacturer"
 * mark to distinguish between various software vendors, and
 * 0x0a was supposed to mean "ZSoft", but everyone else ended
 * up writing a 0x0a into this field, so that's what majority
 * of modern software expects to have in this attribute.
 */
impl Pcx_Header {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Pcx_Header {
    pub fn version(&self) -> Ref<'_, Pcx_Versions> {
        self.version.borrow()
    }
}
impl Pcx_Header {
    pub fn encoding(&self) -> Ref<'_, Pcx_Encodings> {
        self.encoding.borrow()
    }
}
impl Pcx_Header {
    pub fn bits_per_pixel(&self) -> Ref<'_, u8> {
        self.bits_per_pixel.borrow()
    }
}
impl Pcx_Header {
    pub fn img_x_min(&self) -> Ref<'_, u16> {
        self.img_x_min.borrow()
    }
}
impl Pcx_Header {
    pub fn img_y_min(&self) -> Ref<'_, u16> {
        self.img_y_min.borrow()
    }
}
impl Pcx_Header {
    pub fn img_x_max(&self) -> Ref<'_, u16> {
        self.img_x_max.borrow()
    }
}
impl Pcx_Header {
    pub fn img_y_max(&self) -> Ref<'_, u16> {
        self.img_y_max.borrow()
    }
}
impl Pcx_Header {
    pub fn hdpi(&self) -> Ref<'_, u16> {
        self.hdpi.borrow()
    }
}
impl Pcx_Header {
    pub fn vdpi(&self) -> Ref<'_, u16> {
        self.vdpi.borrow()
    }
}
impl Pcx_Header {
    pub fn palette_16(&self) -> Ref<'_, Vec<u8>> {
        self.palette_16.borrow()
    }
}
impl Pcx_Header {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}
impl Pcx_Header {
    pub fn num_planes(&self) -> Ref<'_, u8> {
        self.num_planes.borrow()
    }
}
impl Pcx_Header {
    pub fn bytes_per_line(&self) -> Ref<'_, u16> {
        self.bytes_per_line.borrow()
    }
}
impl Pcx_Header {
    pub fn palette_info(&self) -> Ref<'_, u16> {
        self.palette_info.borrow()
    }
}
impl Pcx_Header {
    pub fn h_screen_size(&self) -> Ref<'_, u16> {
        self.h_screen_size.borrow()
    }
}
impl Pcx_Header {
    pub fn v_screen_size(&self) -> Ref<'_, u16> {
        self.v_screen_size.borrow()
    }
}
impl Pcx_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Pcx_Header {
    pub fn palette_16_raw(&self) -> Ref<'_, Vec<u8>> {
        self.palette_16_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Pcx_Rgb {
    pub(crate) _root: SharedType<Pcx>,
    pub(crate) _parent: SharedType<Pcx_TPalette256>,
    pub(crate) _self_shared: SharedType<Self>,
    r: RefCell<u8>,
    g: RefCell<u8>,
    b: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Pcx_Rgb {
    type Root = Pcx;
    type Parent = Pcx_TPalette256;

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
        *self_rc.r.borrow_mut() = _io.read_u1()?;
        *self_rc.g.borrow_mut() = _io.read_u1()?;
        *self_rc.b.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Pcx_Rgb {
}
impl Pcx_Rgb {
    pub fn r(&self) -> Ref<'_, u8> {
        self.r.borrow()
    }
}
impl Pcx_Rgb {
    pub fn g(&self) -> Ref<'_, u8> {
        self.g.borrow()
    }
}
impl Pcx_Rgb {
    pub fn b(&self) -> Ref<'_, u8> {
        self.b.borrow()
    }
}
impl Pcx_Rgb {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Pcx_TPalette256 {
    pub(crate) _root: SharedType<Pcx>,
    pub(crate) _parent: SharedType<Pcx>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    colors: RefCell<Vec<OptRc<Pcx_Rgb>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Pcx_TPalette256 {
    type Root = Pcx;
    type Parent = Pcx;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.magic() == vec![0xcu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/t_palette_256/seq/0".to_string() }));
        }
        *self_rc.colors.borrow_mut() = Vec::new();
        let l_colors = 256_usize;
        for _i in 0_usize..l_colors {
            let t = Self::read_into::<_, Pcx_Rgb>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.colors.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Pcx_TPalette256 {
}
impl Pcx_TPalette256 {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Pcx_TPalette256 {
    pub fn colors(&self) -> Ref<'_, Vec<OptRc<Pcx_Rgb>>> {
        self.colors.borrow()
    }
}
impl Pcx_TPalette256 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
