// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Microsoft Windows uses specific file format to store applications
 * icons - ICO. This is a container that contains one or more image
 * files (effectively, DIB parts of BMP files or full PNG files are
 * contained inside).
 * \sa <https://learn.microsoft.com/en-us/previous-versions/ms997538(v=msdn.10)> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Ico {
    pub(crate) _root: SharedType<Ico>,
    pub(crate) _parent: SharedType<Ico>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    num_images: RefCell<u16>,
    images: RefCell<Vec<OptRc<Ico_IconDirEntry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Ico {
    type Root = Ico;
    type Parent = Ico;

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
        if !(*self_rc.magic() == vec![0x0u8, 0x0u8, 0x1u8, 0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.num_images.borrow_mut() = _io.read_u2le()?;
        *self_rc.images.borrow_mut() = Vec::new();
        let l_images = usize::from(*self_rc.num_images());
        for _i in 0_usize..l_images {
            let t = Self::read_into::<_, Ico_IconDirEntry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.images.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Ico {
}
impl Ico {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}

/**
 * Number of images contained in this file
 */
impl Ico {
    pub fn num_images(&self) -> Ref<'_, u16> {
        self.num_images.borrow()
    }
}
impl Ico {
    pub fn images(&self) -> Ref<'_, Vec<OptRc<Ico_IconDirEntry>>> {
        self.images.borrow()
    }
}
impl Ico {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ico_IconDirEntry {
    pub(crate) _root: SharedType<Ico>,
    pub(crate) _parent: SharedType<Ico>,
    pub(crate) _self_shared: SharedType<Self>,
    width: RefCell<u8>,
    height: RefCell<u8>,
    num_colors: RefCell<u8>,
    reserved: RefCell<Vec<u8>>,
    num_planes: RefCell<u16>,
    bpp: RefCell<u16>,
    len_img: RefCell<u32>,
    ofs_img: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_img: Cell<bool>,
    img: RefCell<Vec<u8>>,
    f_is_png: Cell<bool>,
    is_png: RefCell<bool>,
    f_png_header: Cell<bool>,
    png_header: RefCell<Vec<u8>>,
}
impl KStruct for Ico_IconDirEntry {
    type Root = Ico;
    type Parent = Ico;

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
        *self_rc.width.borrow_mut() = _io.read_u1()?;
        *self_rc.height.borrow_mut() = _io.read_u1()?;
        *self_rc.num_colors.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved.borrow_mut() = _io.read_bytes(1_usize)?;
        if !(*self_rc.reserved() == vec![0x0u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/icon_dir_entry/seq/3".to_string() }));
        }
        *self_rc.num_planes.borrow_mut() = _io.read_u2le()?;
        *self_rc.bpp.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_img.borrow_mut() = _io.read_u4le()?;
        *self_rc.ofs_img.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Ico_IconDirEntry {

    /**
     * Raw image data. Use `is_png` to determine whether this is an
     * embedded PNG file (true) or a DIB bitmap (false) and call a
     * relevant parser, if needed to parse image data further.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_img.get() {
            return Ok(self.img.borrow());
        }
        self.f_img.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs_img())?)?;
        *self.img.borrow_mut() = _io.read_bytes(usize::try_from(*self.len_img())?)?;
        _io.seek(_pos)?;
        Ok(self.img.borrow())
    }

    /**
     * True if this image is in PNG format.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_png(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_png.get() {
            return Ok(self.is_png.borrow());
        }
        self.f_is_png.set(true);
        *self.is_png.borrow_mut() = (*self.png_header()? == vec![0x89u8, 0x50u8, 0x4eu8, 0x47u8, 0xdu8, 0xau8, 0x1au8, 0xau8]).try_into()?;
        Ok(self.is_png.borrow())
    }

    /**
     * Pre-reads first 8 bytes of the image to determine if it's an
     * embedded PNG file.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn png_header(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_png_header.get() {
            return Ok(self.png_header.borrow());
        }
        self.f_png_header.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs_img())?)?;
        *self.png_header.borrow_mut() = _io.read_bytes(8_usize)?;
        _io.seek(_pos)?;
        Ok(self.png_header.borrow())
    }
}

/**
 * Width of image, px
 */
impl Ico_IconDirEntry {
    pub fn width(&self) -> Ref<'_, u8> {
        self.width.borrow()
    }
}

/**
 * Height of image, px
 */
impl Ico_IconDirEntry {
    pub fn height(&self) -> Ref<'_, u8> {
        self.height.borrow()
    }
}

/**
 * Number of colors in palette of the image or 0 if image has
 * no palette (i.e. RGB, RGBA, etc)
 */
impl Ico_IconDirEntry {
    pub fn num_colors(&self) -> Ref<'_, u8> {
        self.num_colors.borrow()
    }
}
impl Ico_IconDirEntry {
    pub fn reserved(&self) -> Ref<'_, Vec<u8>> {
        self.reserved.borrow()
    }
}

/**
 * Number of color planes
 */
impl Ico_IconDirEntry {
    pub fn num_planes(&self) -> Ref<'_, u16> {
        self.num_planes.borrow()
    }
}

/**
 * Bits per pixel in the image
 */
impl Ico_IconDirEntry {
    pub fn bpp(&self) -> Ref<'_, u16> {
        self.bpp.borrow()
    }
}

/**
 * Size of the image data
 */
impl Ico_IconDirEntry {
    pub fn len_img(&self) -> Ref<'_, u32> {
        self.len_img.borrow()
    }
}

/**
 * Absolute offset of the image data start in the file
 */
impl Ico_IconDirEntry {
    pub fn ofs_img(&self) -> Ref<'_, u32> {
        self.ofs_img.borrow()
    }
}
impl Ico_IconDirEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
