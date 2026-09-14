// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Format of `bootloader-*.img` files found in factory images of certain Android devices from Huawei:
 *
 * * Nexus 6P "angler": [sample][sample-angler] ([other samples][others-angler]),
 *   [releasetools.py](https://android.googlesource.com/device/huawei/angler/+/cf92cd8/releasetools.py#29)
 *
 * [sample-angler]: https://androidfilehost.com/?fid=11410963190603870158 "bootloader-angler-angler-03.84.img"
 * [others-angler]: https://androidfilehost.com/?w=search&s=bootloader-angler&type=files
 *
 * All image versions can be found in factory images at
 * <https://developers.google.com/android/images> for the specific device. To
 * avoid having to download an entire ZIP archive when you only need one file
 * from it, install [remotezip](https://github.com/gtsystem/python-remotezip) and
 * use its [command line
 * tool](https://github.com/gtsystem/python-remotezip#command-line-tool) to list
 * members in the archive and then to download only the file you want.
 * \sa <https://android.googlesource.com/device/huawei/angler/+/673cfb9/releasetools.py> Source
 * \sa <https://source.codeaurora.org/quic/la/device/qcom/common/tree/meta_image/meta_format.h?h=LA.UM.6.1.1&id=a68d284aee85> Source
 * \sa <https://source.codeaurora.org/quic/la/device/qcom/common/tree/meta_image/meta_image.c?h=LA.UM.6.1.1&id=a68d284aee85> Source
 */

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrHuawei {
    pub(crate) _root: SharedType<AndroidBootldrHuawei>,
    pub(crate) _parent: SharedType<AndroidBootldrHuawei>,
    pub(crate) _self_shared: SharedType<Self>,
    meta_header: RefCell<OptRc<AndroidBootldrHuawei_MetaHdr>>,
    header_ext: RefCell<Vec<u8>>,
    image_header: RefCell<OptRc<AndroidBootldrHuawei_ImageHdr>>,
    _io: RefCell<BytesReader>,
    header_ext_raw: RefCell<Vec<u8>>,
    image_header_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidBootldrHuawei {
    type Root = AndroidBootldrHuawei;
    type Parent = AndroidBootldrHuawei;

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
        let t = Self::read_into::<_, AndroidBootldrHuawei_MetaHdr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.meta_header.borrow_mut() = t;
        *self_rc.header_ext.borrow_mut() = _io.read_bytes(usize::try_from((i32::from(*self_rc.meta_header().len_meta_header())).saturating_sub(76_i32))?)?;
        let _raw_image_header = _io.read_bytes(usize::from(*self_rc.meta_header().len_image_header()))?;
        *self_rc.image_header_raw.borrow_mut() = _raw_image_header.clone();
        let _io_image_header = BytesReader::from(_raw_image_header);
        let t = Self::read_into::<BytesReader, AndroidBootldrHuawei_ImageHdr>(&_io_image_header, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.image_header.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrHuawei {
}
impl AndroidBootldrHuawei {
    pub fn meta_header(&self) -> Ref<'_, OptRc<AndroidBootldrHuawei_MetaHdr>> {
        self.meta_header.borrow()
    }
}
impl AndroidBootldrHuawei {
    pub fn header_ext(&self) -> Ref<'_, Vec<u8>> {
        self.header_ext.borrow()
    }
}
impl AndroidBootldrHuawei {
    pub fn image_header(&self) -> Ref<'_, OptRc<AndroidBootldrHuawei_ImageHdr>> {
        self.image_header.borrow()
    }
}
impl AndroidBootldrHuawei {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidBootldrHuawei {
    pub fn header_ext_raw(&self) -> Ref<'_, Vec<u8>> {
        self.header_ext_raw.borrow()
    }
}
impl AndroidBootldrHuawei {
    pub fn image_header_raw(&self) -> Ref<'_, Vec<u8>> {
        self.image_header_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrHuawei_ImageHdr {
    pub(crate) _root: SharedType<AndroidBootldrHuawei>,
    pub(crate) _parent: SharedType<AndroidBootldrHuawei>,
    pub(crate) _self_shared: SharedType<Self>,
    entries: RefCell<Vec<OptRc<AndroidBootldrHuawei_ImageHdrEntry>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidBootldrHuawei_ImageHdr {
    type Root = AndroidBootldrHuawei;
    type Parent = AndroidBootldrHuawei;

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
        *self_rc.entries.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, AndroidBootldrHuawei_ImageHdrEntry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.entries.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrHuawei_ImageHdr {
}

/**
 * The C generator program defines `img_header` as a [fixed size
 * array](https://source.codeaurora.org/quic/la/device/qcom/common/tree/meta_image/meta_image.c?h=LA.UM.6.1.1&id=a68d284aee85#n42)
 * of `img_header_entry_t` structs with length `MAX_IMAGES` (which is
 * defined as `16`).
 *
 * This means that technically there will always be 16 `image_hdr`
 * entries, the first *n* entries being used (filled with real values)
 * and the rest left unused with all bytes zero.
 *
 * To check if an entry is used, use the `is_used` attribute.
 */
impl AndroidBootldrHuawei_ImageHdr {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<AndroidBootldrHuawei_ImageHdrEntry>>> {
        self.entries.borrow()
    }
}
impl AndroidBootldrHuawei_ImageHdr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrHuawei_ImageHdrEntry {
    pub(crate) _root: SharedType<AndroidBootldrHuawei>,
    pub(crate) _parent: SharedType<AndroidBootldrHuawei_ImageHdr>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<String>,
    ofs_body: RefCell<u32>,
    len_body: RefCell<u32>,
    _io: RefCell<BytesReader>,
    name_raw: RefCell<Vec<u8>>,
    f_body: Cell<bool>,
    body: RefCell<Vec<u8>>,
    f_is_used: Cell<bool>,
    is_used: RefCell<bool>,
}
impl KStruct for AndroidBootldrHuawei_ImageHdrEntry {
    type Root = AndroidBootldrHuawei;
    type Parent = AndroidBootldrHuawei_ImageHdr;

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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(72_usize)?, Some(0), false, None), "ASCII")?;
        *self_rc.ofs_body.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_body.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrHuawei_ImageHdrEntry {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn body(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        self.f_body.set(true);
        if *self.is_used()? {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from(*self.ofs_body())?)?;
            *self.body.borrow_mut() = io.read_bytes(usize::try_from(*self.len_body())?)?;
            io.seek(_pos)?;
        }
        Ok(self.body.borrow())
    }

    /**
     * \sa <https://source.codeaurora.org/quic/la/device/qcom/common/tree/meta_image/meta_image.c?h=LA.UM.6.1.1&id=a68d284aee85#n119> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_used(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_used.get() {
            return Ok(self.is_used.borrow());
        }
        self.f_is_used.set(true);
        *self.is_used.borrow_mut() = ( ((((to_i128(*self.ofs_body())) != (to_i128(0)))) && (((to_i128(*self.len_body())) != (to_i128(0))))) ).try_into()?;
        Ok(self.is_used.borrow())
    }
}

/**
 * partition name
 */
impl AndroidBootldrHuawei_ImageHdrEntry {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl AndroidBootldrHuawei_ImageHdrEntry {
    pub fn ofs_body(&self) -> Ref<'_, u32> {
        self.ofs_body.borrow()
    }
}
impl AndroidBootldrHuawei_ImageHdrEntry {
    pub fn len_body(&self) -> Ref<'_, u32> {
        self.len_body.borrow()
    }
}
impl AndroidBootldrHuawei_ImageHdrEntry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidBootldrHuawei_ImageHdrEntry {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrHuawei_MetaHdr {
    pub(crate) _root: SharedType<AndroidBootldrHuawei>,
    pub(crate) _parent: SharedType<AndroidBootldrHuawei>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    version: RefCell<OptRc<AndroidBootldrHuawei_Version>>,
    image_version: RefCell<String>,
    len_meta_header: RefCell<u16>,
    len_image_header: RefCell<u16>,
    _io: RefCell<BytesReader>,
    image_version_raw: RefCell<Vec<u8>>,
}
impl KStruct for AndroidBootldrHuawei_MetaHdr {
    type Root = AndroidBootldrHuawei;
    type Parent = AndroidBootldrHuawei;

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
        if !(*self_rc.magic() == vec![0x3cu8, 0xd6u8, 0x1au8, 0xceu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/meta_hdr/seq/0".to_string() }));
        }
        let t = Self::read_into::<_, AndroidBootldrHuawei_Version>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.version.borrow_mut() = t;
        *self_rc.image_version.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(64_usize)?, Some(0), false, None), "ASCII")?;
        *self_rc.len_meta_header.borrow_mut() = _io.read_u2le()?;
        *self_rc.len_image_header.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrHuawei_MetaHdr {
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn version(&self) -> Ref<'_, OptRc<AndroidBootldrHuawei_Version>> {
        self.version.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn image_version(&self) -> Ref<'_, String> {
        self.image_version.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn len_meta_header(&self) -> Ref<'_, u16> {
        self.len_meta_header.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn len_image_header(&self) -> Ref<'_, u16> {
        self.len_image_header.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl AndroidBootldrHuawei_MetaHdr {
    pub fn image_version_raw(&self) -> Ref<'_, Vec<u8>> {
        self.image_version_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidBootldrHuawei_Version {
    pub(crate) _root: SharedType<AndroidBootldrHuawei>,
    pub(crate) _parent: SharedType<AndroidBootldrHuawei_MetaHdr>,
    pub(crate) _self_shared: SharedType<Self>,
    major: RefCell<u16>,
    minor: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidBootldrHuawei_Version {
    type Root = AndroidBootldrHuawei;
    type Parent = AndroidBootldrHuawei_MetaHdr;

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
        *self_rc.major.borrow_mut() = _io.read_u2le()?;
        *self_rc.minor.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidBootldrHuawei_Version {
}
impl AndroidBootldrHuawei_Version {
    pub fn major(&self) -> Ref<'_, u16> {
        self.major.borrow()
    }
}
impl AndroidBootldrHuawei_Version {
    pub fn minor(&self) -> Ref<'_, u16> {
        self.minor.borrow()
    }
}
impl AndroidBootldrHuawei_Version {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
