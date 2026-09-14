// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * A bootloader image which only seems to have been used on a few ASUS
 * devices. The encoding is ASCII, because the `releasetools.py` script
 * is written using Python 2, where the default encoding is ASCII.
 *
 * A test file can be found in the firmware files for the "fugu" device,
 * which can be downloaded from <https://developers.google.com/android/images>
 * \sa <https://android.googlesource.com/device/asus/fugu/+/android-8.1.0_r5/releasetools.py> Source
 */

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrAsus {
    pub(crate) _root: SharedType<AndroidBootldrAsus>,
    pub(crate) _parent: SharedType<AndroidBootldrAsus>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    revision: RefCell<u16>,
    reserved1: RefCell<u16>,
    reserved2: RefCell<u32>,
    images: RefCell<Vec<OptRc<AndroidBootldrAsus_Image>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidBootldrAsus {
    type Root = AndroidBootldrAsus;
    type Parent = AndroidBootldrAsus;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(8_usize)?;
        if !(*self_rc.magic() == vec![0x42u8, 0x4fu8, 0x4fu8, 0x54u8, 0x4cu8, 0x44u8, 0x52u8, 0x21u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.revision.borrow_mut() = _io.read_u2le()?;
        let min_val: u16 = (2).try_into()?;
        if !(*self_rc.revision() >= min_val) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::LessThan, src_path: "/seq/1".to_string() }));
        }
        *self_rc.reserved1.borrow_mut() = _io.read_u2le()?;
        *self_rc.reserved2.borrow_mut() = _io.read_u4le()?;
        *self_rc.images.borrow_mut() = Vec::new();
        let l_images = 3_usize;
        for _i in 0_usize..l_images {
            let t = Self::read_into::<_, AndroidBootldrAsus_Image>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.images.borrow_mut().push(t);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrAsus {
}
impl AndroidBootldrAsus {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl AndroidBootldrAsus {
    pub fn revision(&self) -> Ref<'_, u16> {
        self.revision.borrow()
    }
}
impl AndroidBootldrAsus {
    pub fn reserved1(&self) -> Ref<'_, u16> {
        self.reserved1.borrow()
    }
}
impl AndroidBootldrAsus {
    pub fn reserved2(&self) -> Ref<'_, u32> {
        self.reserved2.borrow()
    }
}

/**
 * Only three images are included: `ifwi.bin`, `droidboot.img`
 * and `splashscreen.img`
 */
impl AndroidBootldrAsus {
    pub fn images(&self) -> Ref<'_, Vec<OptRc<AndroidBootldrAsus_Image>>> {
        self.images.borrow()
    }
}
impl AndroidBootldrAsus {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrAsus_Image {
    pub(crate) _root: SharedType<AndroidBootldrAsus>,
    pub(crate) _parent: SharedType<AndroidBootldrAsus>,
    pub(crate) _self_shared: SharedType<Self>,
    chunk_id: RefCell<String>,
    len_body: RefCell<u32>,
    flags: RefCell<u8>,
    reserved1: RefCell<u8>,
    reserved2: RefCell<u8>,
    reserved3: RefCell<u8>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    chunk_id_raw: RefCell<Vec<u8>>,
    body_raw: RefCell<Vec<u8>>,
    f_file_name: Cell<bool>,
    file_name: RefCell<String>,
}
impl KStruct for AndroidBootldrAsus_Image {
    type Root = AndroidBootldrAsus;
    type Parent = AndroidBootldrAsus;

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
        *self_rc.chunk_id.borrow_mut() = bytes_to_str(&_io.read_bytes(8_usize)?, "ASCII")?;
        let _item = &*self_rc.chunk_id();
        if !(*_item == "IFWI!!!!" || *_item == "DROIDBT!" || *_item == "SPLASHS!") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotAnyOf, src_path: "/types/image/seq/0".to_string() }));
        }
        *self_rc.len_body.borrow_mut() = _io.read_u4le()?;
        *self_rc.flags.borrow_mut() = _io.read_u1()?;
        let _borrowed = self_rc.flags();
        let _tmpa = *_borrowed;
        if !(((_tmpa & 1_u8) != 0_u8)) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::Expr, src_path: "/types/image/seq/2".to_string() }));
        }
        *self_rc.reserved1.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved2.borrow_mut() = _io.read_u1()?;
        *self_rc.reserved3.borrow_mut() = _io.read_u1()?;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_body())?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrAsus_Image {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn file_name(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_file_name.get() {
            return Ok(self.file_name.borrow());
        }
        self.f_file_name.set(true);
        *self.file_name.borrow_mut() = if (self.chunk_id().as_str() == "IFWI!!!!") { "ifwi.bin".to_string() } else { if (self.chunk_id().as_str() == "DROIDBT!") { "droidboot.img".to_string() } else { if (self.chunk_id().as_str() == "SPLASHS!") { "splashscreen.img".to_string() } else { "".to_string() }.to_string() }.to_string() }.to_string();
        Ok(self.file_name.borrow())
    }
}
impl AndroidBootldrAsus_Image {
    pub fn chunk_id(&self) -> Ref<'_, String> {
        self.chunk_id.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn len_body(&self) -> Ref<'_, u32> {
        self.len_body.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn flags(&self) -> Ref<'_, u8> {
        self.flags.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn reserved1(&self) -> Ref<'_, u8> {
        self.reserved1.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn reserved2(&self) -> Ref<'_, u8> {
        self.reserved2.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn reserved3(&self) -> Ref<'_, u8> {
        self.reserved3.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn chunk_id_raw(&self) -> Ref<'_, Vec<u8>> {
        self.chunk_id_raw.borrow()
    }
}
impl AndroidBootldrAsus_Image {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}
