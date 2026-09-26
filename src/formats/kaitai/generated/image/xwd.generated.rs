// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * xwd is a file format written by eponymous X11 screen capture
 * application (xwd stands for "X Window Dump"). Typically, an average
 * user transforms xwd format into something more widespread by any of
 * `xwdtopnm` and `pnmto...` utilities right away.
 *
 * xwd format itself provides a raw uncompressed bitmap with some
 * metainformation, like pixel format, width, height, bit depth,
 * etc. Note that technically format includes machine-dependent fields
 * and thus is probably a poor choice for true cross-platform usage.
 */

#[derive(Default, Debug, Clone)]
pub struct Xwd {
    pub(crate) _root: SharedType<Xwd>,
    pub(crate) _parent: SharedType<Xwd>,
    pub(crate) _self_shared: SharedType<Self>,
    len_header: RefCell<u32>,
    hdr: RefCell<OptRc<Xwd_Header>>,
    color_map: RefCell<Vec<OptRc<Xwd_ColorMapEntry>>>,
    _io: RefCell<BytesReader>,
    hdr_raw: RefCell<Vec<u8>>,
    color_map_raw: RefCell<Vec<u8>>,
}
impl KStruct for Xwd {
    type Root = Xwd;
    type Parent = Xwd;

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
        *self_rc.len_header.borrow_mut() = _io.read_u4be()?;
        let _raw_hdr = _io.read_bytes(usize::try_from((*self_rc.len_header()).saturating_sub(4_u32))?)?;
        *self_rc.hdr_raw.borrow_mut() = _raw_hdr.clone();
        let _io_hdr = BytesReader::from(_raw_hdr);
        let t = Self::read_into::<BytesReader, Xwd_Header>(&_io_hdr, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.hdr.borrow_mut() = t;
        *self_rc.color_map.borrow_mut() = Vec::new();
        let l_color_map = usize::try_from(*self_rc.hdr().color_map_entries())?;
        for _i in 0_usize..l_color_map {
            let _raw_color_map = _io.read_bytes(12_usize)?;
            let _io_color_map = BytesReader::from(_raw_color_map);
            let t = Self::read_into::<BytesReader, Xwd_ColorMapEntry>(&_io_color_map, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.color_map.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Xwd {
}

/**
 * Size of the header in bytes
 */
impl Xwd {
    pub fn len_header(&self) -> Ref<'_, u32> {
        self.len_header.borrow()
    }
}
impl Xwd {
    pub fn hdr(&self) -> Ref<'_, OptRc<Xwd_Header>> {
        self.hdr.borrow()
    }
}
impl Xwd {
    pub fn color_map(&self) -> Ref<'_, Vec<OptRc<Xwd_ColorMapEntry>>> {
        self.color_map.borrow()
    }
}
impl Xwd {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Xwd {
    pub fn hdr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.hdr_raw.borrow()
    }
}
impl Xwd {
    pub fn color_map_raw(&self) -> Ref<'_, Vec<u8>> {
        self.color_map_raw.borrow()
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Xwd_ByteOrder {
    Le,
    Be,
    Unknown(i64),
}

impl TryFrom<i64> for Xwd_ByteOrder {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Xwd_ByteOrder> {
        match flag {
            0 => Ok(Xwd_ByteOrder::Le),
            1 => Ok(Xwd_ByteOrder::Be),
            _ => Ok(Xwd_ByteOrder::Unknown(flag)),
        }
    }
}

impl From<&Xwd_ByteOrder> for i64 {
    fn from(v: &Xwd_ByteOrder) -> Self {
        match *v {
            Xwd_ByteOrder::Le => 0,
            Xwd_ByteOrder::Be => 1,
            Xwd_ByteOrder::Unknown(v) => v
        }
    }
}

impl Default for Xwd_ByteOrder {
    fn default() -> Self { Xwd_ByteOrder::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Xwd_PixmapFormat {
    XYBitmap,
    XYPixmap,
    ZPixmap,
    Unknown(i64),
}

impl TryFrom<i64> for Xwd_PixmapFormat {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Xwd_PixmapFormat> {
        match flag {
            0 => Ok(Xwd_PixmapFormat::XYBitmap),
            1 => Ok(Xwd_PixmapFormat::XYPixmap),
            2 => Ok(Xwd_PixmapFormat::ZPixmap),
            _ => Ok(Xwd_PixmapFormat::Unknown(flag)),
        }
    }
}

impl From<&Xwd_PixmapFormat> for i64 {
    fn from(v: &Xwd_PixmapFormat) -> Self {
        match *v {
            Xwd_PixmapFormat::XYBitmap => 0,
            Xwd_PixmapFormat::XYPixmap => 1,
            Xwd_PixmapFormat::ZPixmap => 2,
            Xwd_PixmapFormat::Unknown(v) => v
        }
    }
}

impl Default for Xwd_PixmapFormat {
    fn default() -> Self { Xwd_PixmapFormat::Unknown(0) }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Xwd_VisualClass {
    StaticGray,
    GrayScale,
    StaticColor,
    PseudoColor,
    TrueColor,
    DirectColor,
    Unknown(i64),
}

impl TryFrom<i64> for Xwd_VisualClass {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Xwd_VisualClass> {
        match flag {
            0 => Ok(Xwd_VisualClass::StaticGray),
            1 => Ok(Xwd_VisualClass::GrayScale),
            2 => Ok(Xwd_VisualClass::StaticColor),
            3 => Ok(Xwd_VisualClass::PseudoColor),
            4 => Ok(Xwd_VisualClass::TrueColor),
            5 => Ok(Xwd_VisualClass::DirectColor),
            _ => Ok(Xwd_VisualClass::Unknown(flag)),
        }
    }
}

impl From<&Xwd_VisualClass> for i64 {
    fn from(v: &Xwd_VisualClass) -> Self {
        match *v {
            Xwd_VisualClass::StaticGray => 0,
            Xwd_VisualClass::GrayScale => 1,
            Xwd_VisualClass::StaticColor => 2,
            Xwd_VisualClass::PseudoColor => 3,
            Xwd_VisualClass::TrueColor => 4,
            Xwd_VisualClass::DirectColor => 5,
            Xwd_VisualClass::Unknown(v) => v
        }
    }
}

impl Default for Xwd_VisualClass {
    fn default() -> Self { Xwd_VisualClass::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Xwd_ColorMapEntry {
    pub(crate) _root: SharedType<Xwd>,
    pub(crate) _parent: SharedType<Xwd>,
    pub(crate) _self_shared: SharedType<Self>,
    entry_number: RefCell<u32>,
    red: RefCell<u16>,
    green: RefCell<u16>,
    blue: RefCell<u16>,
    flags: RefCell<u8>,
    padding: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Xwd_ColorMapEntry {
    type Root = Xwd;
    type Parent = Xwd;

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
        *self_rc.entry_number.borrow_mut() = _io.read_u4be()?;
        *self_rc.red.borrow_mut() = _io.read_u2be()?;
        *self_rc.green.borrow_mut() = _io.read_u2be()?;
        *self_rc.blue.borrow_mut() = _io.read_u2be()?;
        *self_rc.flags.borrow_mut() = _io.read_u1()?;
        *self_rc.padding.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Xwd_ColorMapEntry {
}

/**
 * Number of the color map entry
 */
impl Xwd_ColorMapEntry {
    pub fn entry_number(&self) -> Ref<'_, u32> {
        self.entry_number.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn red(&self) -> Ref<'_, u16> {
        self.red.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn green(&self) -> Ref<'_, u16> {
        self.green.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn blue(&self) -> Ref<'_, u16> {
        self.blue.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn flags(&self) -> Ref<'_, u8> {
        self.flags.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn padding(&self) -> Ref<'_, u8> {
        self.padding.borrow()
    }
}
impl Xwd_ColorMapEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Xwd_Header {
    pub(crate) _root: SharedType<Xwd>,
    pub(crate) _parent: SharedType<Xwd>,
    pub(crate) _self_shared: SharedType<Self>,
    file_version: RefCell<u32>,
    pixmap_format: RefCell<Xwd_PixmapFormat>,
    pixmap_depth: RefCell<u32>,
    pixmap_width: RefCell<u32>,
    pixmap_height: RefCell<u32>,
    x_offset: RefCell<u32>,
    byte_order: RefCell<Xwd_ByteOrder>,
    bitmap_unit: RefCell<u32>,
    bitmap_bit_order: RefCell<u32>,
    bitmap_pad: RefCell<u32>,
    bits_per_pixel: RefCell<u32>,
    bytes_per_line: RefCell<u32>,
    visual_class: RefCell<Xwd_VisualClass>,
    red_mask: RefCell<u32>,
    green_mask: RefCell<u32>,
    blue_mask: RefCell<u32>,
    bits_per_rgb: RefCell<u32>,
    number_of_colors: RefCell<u32>,
    color_map_entries: RefCell<u32>,
    window_width: RefCell<u32>,
    window_height: RefCell<u32>,
    window_x: RefCell<i32>,
    window_y: RefCell<i32>,
    window_border_width: RefCell<u32>,
    creator: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Xwd_Header {
    type Root = Xwd;
    type Parent = Xwd;

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
        *self_rc.file_version.borrow_mut() = _io.read_u4be()?;
        *self_rc.pixmap_format.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc.pixmap_depth.borrow_mut() = _io.read_u4be()?;
        *self_rc.pixmap_width.borrow_mut() = _io.read_u4be()?;
        *self_rc.pixmap_height.borrow_mut() = _io.read_u4be()?;
        *self_rc.x_offset.borrow_mut() = _io.read_u4be()?;
        *self_rc.byte_order.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc.bitmap_unit.borrow_mut() = _io.read_u4be()?;
        *self_rc.bitmap_bit_order.borrow_mut() = _io.read_u4be()?;
        *self_rc.bitmap_pad.borrow_mut() = _io.read_u4be()?;
        *self_rc.bits_per_pixel.borrow_mut() = _io.read_u4be()?;
        *self_rc.bytes_per_line.borrow_mut() = _io.read_u4be()?;
        *self_rc.visual_class.borrow_mut() = i64::from(_io.read_u4be()?).try_into()?;
        *self_rc.red_mask.borrow_mut() = _io.read_u4be()?;
        *self_rc.green_mask.borrow_mut() = _io.read_u4be()?;
        *self_rc.blue_mask.borrow_mut() = _io.read_u4be()?;
        *self_rc.bits_per_rgb.borrow_mut() = _io.read_u4be()?;
        *self_rc.number_of_colors.borrow_mut() = _io.read_u4be()?;
        *self_rc.color_map_entries.borrow_mut() = _io.read_u4be()?;
        *self_rc.window_width.borrow_mut() = _io.read_u4be()?;
        *self_rc.window_height.borrow_mut() = _io.read_u4be()?;
        *self_rc.window_x.borrow_mut() = _io.read_s4be()?;
        *self_rc.window_y.borrow_mut() = _io.read_s4be()?;
        *self_rc.window_border_width.borrow_mut() = _io.read_u4be()?;
        *self_rc.creator.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Xwd_Header {
}

/**
 * X11WD file version (always 07h)
 */
impl Xwd_Header {
    pub fn file_version(&self) -> Ref<'_, u32> {
        self.file_version.borrow()
    }
}

/**
 * Format of the image data
 */
impl Xwd_Header {
    pub fn pixmap_format(&self) -> Ref<'_, Xwd_PixmapFormat> {
        self.pixmap_format.borrow()
    }
}

/**
 * Pixmap depth in pixels - in practice, bits per pixel
 */
impl Xwd_Header {
    pub fn pixmap_depth(&self) -> Ref<'_, u32> {
        self.pixmap_depth.borrow()
    }
}

/**
 * Pixmap width in pixels
 */
impl Xwd_Header {
    pub fn pixmap_width(&self) -> Ref<'_, u32> {
        self.pixmap_width.borrow()
    }
}

/**
 * Pixmap height in pixels
 */
impl Xwd_Header {
    pub fn pixmap_height(&self) -> Ref<'_, u32> {
        self.pixmap_height.borrow()
    }
}

/**
 * Bitmap X offset (number of pixels to ignore at the beginning of each scan-line)
 */
impl Xwd_Header {
    pub fn x_offset(&self) -> Ref<'_, u32> {
        self.x_offset.borrow()
    }
}

/**
 * Byte order of image data
 */
impl Xwd_Header {
    pub fn byte_order(&self) -> Ref<'_, Xwd_ByteOrder> {
        self.byte_order.borrow()
    }
}

/**
 * Bitmap base data size
 */
impl Xwd_Header {
    pub fn bitmap_unit(&self) -> Ref<'_, u32> {
        self.bitmap_unit.borrow()
    }
}

/**
 * Bit-order of image data
 */
impl Xwd_Header {
    pub fn bitmap_bit_order(&self) -> Ref<'_, u32> {
        self.bitmap_bit_order.borrow()
    }
}

/**
 * Bitmap scan-line pad
 */
impl Xwd_Header {
    pub fn bitmap_pad(&self) -> Ref<'_, u32> {
        self.bitmap_pad.borrow()
    }
}

/**
 * Bits per pixel
 */
impl Xwd_Header {
    pub fn bits_per_pixel(&self) -> Ref<'_, u32> {
        self.bits_per_pixel.borrow()
    }
}

/**
 * Bytes per scan-line
 */
impl Xwd_Header {
    pub fn bytes_per_line(&self) -> Ref<'_, u32> {
        self.bytes_per_line.borrow()
    }
}

/**
 * Class of the image
 */
impl Xwd_Header {
    pub fn visual_class(&self) -> Ref<'_, Xwd_VisualClass> {
        self.visual_class.borrow()
    }
}

/**
 * Red mask
 */
impl Xwd_Header {
    pub fn red_mask(&self) -> Ref<'_, u32> {
        self.red_mask.borrow()
    }
}

/**
 * Green mask
 */
impl Xwd_Header {
    pub fn green_mask(&self) -> Ref<'_, u32> {
        self.green_mask.borrow()
    }
}

/**
 * Blue mask
 */
impl Xwd_Header {
    pub fn blue_mask(&self) -> Ref<'_, u32> {
        self.blue_mask.borrow()
    }
}

/**
 * Size of each color mask in bits
 */
impl Xwd_Header {
    pub fn bits_per_rgb(&self) -> Ref<'_, u32> {
        self.bits_per_rgb.borrow()
    }
}

/**
 * Number of colors in image
 */
impl Xwd_Header {
    pub fn number_of_colors(&self) -> Ref<'_, u32> {
        self.number_of_colors.borrow()
    }
}

/**
 * Number of entries in color map
 */
impl Xwd_Header {
    pub fn color_map_entries(&self) -> Ref<'_, u32> {
        self.color_map_entries.borrow()
    }
}

/**
 * Window width
 */
impl Xwd_Header {
    pub fn window_width(&self) -> Ref<'_, u32> {
        self.window_width.borrow()
    }
}

/**
 * Window height
 */
impl Xwd_Header {
    pub fn window_height(&self) -> Ref<'_, u32> {
        self.window_height.borrow()
    }
}

/**
 * Window upper left X coordinate
 */
impl Xwd_Header {
    pub fn window_x(&self) -> Ref<'_, i32> {
        self.window_x.borrow()
    }
}

/**
 * Window upper left Y coordinate
 */
impl Xwd_Header {
    pub fn window_y(&self) -> Ref<'_, i32> {
        self.window_y.borrow()
    }
}

/**
 * Window border width
 */
impl Xwd_Header {
    pub fn window_border_width(&self) -> Ref<'_, u32> {
        self.window_border_width.borrow()
    }
}

/**
 * Program that created this xwd file
 */
impl Xwd_Header {
    pub fn creator(&self) -> Ref<'_, String> {
        self.creator.borrow()
    }
}
impl Xwd_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
