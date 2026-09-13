// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://source.android.com/docs/core/architecture/bootloader/boot-image-header> Source
 */

#[derive(Default, Debug, Clone)]
pub struct AndroidImg {
    pub(crate) _root: SharedType<AndroidImg>,
    pub(crate) _parent: SharedType<AndroidImg>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    kernel: RefCell<OptRc<AndroidImg_Load>>,
    ramdisk: RefCell<OptRc<AndroidImg_Load>>,
    second: RefCell<OptRc<AndroidImg_Load>>,
    tags_load: RefCell<u32>,
    page_size: RefCell<u32>,
    header_version: RefCell<u32>,
    os_version: RefCell<OptRc<AndroidImg_OsVersion>>,
    name: RefCell<String>,
    cmdline: RefCell<String>,
    sha: RefCell<Vec<u8>>,
    extra_cmdline: RefCell<String>,
    recovery_dtbo: RefCell<OptRc<AndroidImg_SizeOffset>>,
    boot_header_size: RefCell<u32>,
    dtb: RefCell<OptRc<AndroidImg_LoadLong>>,
    _io: RefCell<BytesReader>,
    f_base: Cell<bool>,
    base: RefCell<i32>,
    f_dtb_img: Cell<bool>,
    dtb_img: RefCell<Vec<u8>>,
    f_dtb_offset: Cell<bool>,
    dtb_offset: RefCell<i32>,
    f_kernel_img: Cell<bool>,
    kernel_img: RefCell<Vec<u8>>,
    f_kernel_offset: Cell<bool>,
    kernel_offset: RefCell<i32>,
    f_ramdisk_img: Cell<bool>,
    ramdisk_img: RefCell<Vec<u8>>,
    f_ramdisk_offset: Cell<bool>,
    ramdisk_offset: RefCell<i32>,
    f_recovery_dtbo_img: Cell<bool>,
    recovery_dtbo_img: RefCell<Vec<u8>>,
    f_second_img: Cell<bool>,
    second_img: RefCell<Vec<u8>>,
    f_second_offset: Cell<bool>,
    second_offset: RefCell<i32>,
    f_tags_offset: Cell<bool>,
    tags_offset: RefCell<i32>,
}
impl KStruct for AndroidImg {
    type Root = AndroidImg;
    type Parent = AndroidImg;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(usize::try_from(8)?)?;
        if !(*self_rc.magic() == vec![0x41u8, 0x4eu8, 0x44u8, 0x52u8, 0x4fu8, 0x49u8, 0x44u8, 0x21u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        let t = Self::read_into::<_, AndroidImg_Load>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.kernel.borrow_mut() = t;
        let t = Self::read_into::<_, AndroidImg_Load>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.ramdisk.borrow_mut() = t;
        let t = Self::read_into::<_, AndroidImg_Load>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.second.borrow_mut() = t;
        *self_rc.tags_load.borrow_mut() = _io.read_u4le()?;
        *self_rc.page_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.header_version.borrow_mut() = _io.read_u4le()?;
        let t = Self::read_into::<_, AndroidImg_OsVersion>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.os_version.borrow_mut() = t;
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(usize::try_from(16)?)?, 0, false), "ASCII")?;
        *self_rc.cmdline.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(usize::try_from(512)?)?, 0, false), "ASCII")?;
        *self_rc.sha.borrow_mut() = _io.read_bytes(usize::try_from(32)?)?;
        *self_rc.extra_cmdline.borrow_mut() = bytes_to_str(&bytes_terminate(&_io.read_bytes(usize::try_from(1024)?)?, 0, false), "ASCII")?;
        if (((*self_rc.header_version()) as u32) > ((0) as u32)) {
            let t = Self::read_into::<_, AndroidImg_SizeOffset>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.recovery_dtbo.borrow_mut() = t;
        }
        if (((*self_rc.header_version()) as u32) > ((0) as u32)) {
            *self_rc.boot_header_size.borrow_mut() = _io.read_u4le()?;
        }
        if (((*self_rc.header_version()) as u32) > ((1) as u32)) {
            let t = Self::read_into::<_, AndroidImg_LoadLong>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.dtb.borrow_mut() = t;
        }
        Ok(())
    }
}
impl AndroidImg {

    /**
     * base loading address
     */
    pub fn base(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_base.get() {
            return Ok(self.base.borrow());
        }
        self.f_base.set(true);
        *self.base.borrow_mut() = ((((*self.kernel().addr()) as u32) - ((32768) as u32))).try_into()?;
        Ok(self.base.borrow())
    }
    pub fn dtb_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_dtb_img.get() {
            return Ok(self.dtb_img.borrow());
        }
        self.f_dtb_img.set(true);
        if  (((((*self.header_version()) as u32) > ((1) as u32))) && ((((*self.dtb().size()) as u32) > ((0) as u32))))  {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((((((((((((((((((((((((*self.page_size()) as u32) + ((*self.kernel().size()) as u32))) as u32) + ((*self.ramdisk().size()) as u32))) as u32) + ((*self.second().size()) as u32))) as u32) + ((*self.recovery_dtbo().size()) as u32))) as u32) + ((*self.page_size()) as u32))) as u32) - ((1) as u32))) as u32) / ((*self.page_size()) as u32))) as u32) * ((*self.page_size()) as u32)))?)?;
            *self.dtb_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.dtb().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.dtb_img.borrow())
    }

    /**
     * dtb offset from base
     */
    pub fn dtb_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_dtb_offset.get() {
            return Ok(self.dtb_offset.borrow());
        }
        self.f_dtb_offset.set(true);
        if (((*self.header_version()) as u32) > ((1) as u32)) {
            *self.dtb_offset.borrow_mut() = (if (((*self.dtb().addr()) as u64) > ((0) as u64)) { ((((*self.dtb().addr()) as u64) - ((*self.base()?) as u64))) as u64 } else { (0) as u64 }).try_into()?;
        }
        Ok(self.dtb_offset.borrow())
    }
    pub fn kernel_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_kernel_img.get() {
            return Ok(self.kernel_img.borrow());
        }
        self.f_kernel_img.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.page_size())?)?;
        *self.kernel_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.kernel().size())?)?;
        _io.seek(_pos)?;
        Ok(self.kernel_img.borrow())
    }

    /**
     * kernel offset from base
     */
    pub fn kernel_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_kernel_offset.get() {
            return Ok(self.kernel_offset.borrow());
        }
        self.f_kernel_offset.set(true);
        *self.kernel_offset.borrow_mut() = ((((*self.kernel().addr()) as u32) - ((*self.base()?) as u32))).try_into()?;
        Ok(self.kernel_offset.borrow())
    }
    pub fn ramdisk_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_ramdisk_img.get() {
            return Ok(self.ramdisk_img.borrow());
        }
        self.f_ramdisk_img.set(true);
        if (((*self.ramdisk().size()) as u32) > ((0) as u32)) {
            let _pos = _io.pos();
            _io.seek(usize::try_from((((((((((((((((*self.page_size()) as u32) + ((*self.kernel().size()) as u32))) as u32) + ((*self.page_size()) as u32))) as u32) - ((1) as u32))) as u32) / ((*self.page_size()) as u32))) as u32) * ((*self.page_size()) as u32)))?)?;
            *self.ramdisk_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.ramdisk().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.ramdisk_img.borrow())
    }

    /**
     * ramdisk offset from base
     */
    pub fn ramdisk_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ramdisk_offset.get() {
            return Ok(self.ramdisk_offset.borrow());
        }
        self.f_ramdisk_offset.set(true);
        *self.ramdisk_offset.borrow_mut() = (if (((*self.ramdisk().addr()) as u32) > ((0) as u32)) { ((((*self.ramdisk().addr()) as u32) - ((*self.base()?) as u32))) as u32 } else { (0) as u32 }).try_into()?;
        Ok(self.ramdisk_offset.borrow())
    }
    pub fn recovery_dtbo_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_recovery_dtbo_img.get() {
            return Ok(self.recovery_dtbo_img.borrow());
        }
        self.f_recovery_dtbo_img.set(true);
        if  (((((*self.header_version()) as u32) > ((0) as u32))) && ((((*self.recovery_dtbo().size()) as u32) > ((0) as u32))))  {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.recovery_dtbo().offset())?)?;
            *self.recovery_dtbo_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.recovery_dtbo().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.recovery_dtbo_img.borrow())
    }
    pub fn second_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_second_img.get() {
            return Ok(self.second_img.borrow());
        }
        self.f_second_img.set(true);
        if (((*self.second().size()) as u32) > ((0) as u32)) {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((((((((((((((((((*self.page_size()) as u32) + ((*self.kernel().size()) as u32))) as u32) + ((*self.ramdisk().size()) as u32))) as u32) + ((*self.page_size()) as u32))) as u32) - ((1) as u32))) as u32) / ((*self.page_size()) as u32))) as u32) * ((*self.page_size()) as u32)))?)?;
            *self.second_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.second().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.second_img.borrow())
    }

    /**
     * 2nd bootloader offset from base
     */
    pub fn second_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_second_offset.get() {
            return Ok(self.second_offset.borrow());
        }
        self.f_second_offset.set(true);
        *self.second_offset.borrow_mut() = (if (((*self.second().addr()) as u32) > ((0) as u32)) { ((((*self.second().addr()) as u32) - ((*self.base()?) as u32))) as u32 } else { (0) as u32 }).try_into()?;
        Ok(self.second_offset.borrow())
    }

    /**
     * tags offset from base
     */
    pub fn tags_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_tags_offset.get() {
            return Ok(self.tags_offset.borrow());
        }
        self.f_tags_offset.set(true);
        *self.tags_offset.borrow_mut() = ((((*self.tags_load()) as u32) - ((*self.base()?) as u32))).try_into()?;
        Ok(self.tags_offset.borrow())
    }
}
impl AndroidImg {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl AndroidImg {
    pub fn kernel(&self) -> Ref<'_, OptRc<AndroidImg_Load>> {
        self.kernel.borrow()
    }
}
impl AndroidImg {
    pub fn ramdisk(&self) -> Ref<'_, OptRc<AndroidImg_Load>> {
        self.ramdisk.borrow()
    }
}
impl AndroidImg {
    pub fn second(&self) -> Ref<'_, OptRc<AndroidImg_Load>> {
        self.second.borrow()
    }
}
impl AndroidImg {
    pub fn tags_load(&self) -> Ref<'_, u32> {
        self.tags_load.borrow()
    }
}
impl AndroidImg {
    pub fn page_size(&self) -> Ref<'_, u32> {
        self.page_size.borrow()
    }
}
impl AndroidImg {
    pub fn header_version(&self) -> Ref<'_, u32> {
        self.header_version.borrow()
    }
}
impl AndroidImg {
    pub fn os_version(&self) -> Ref<'_, OptRc<AndroidImg_OsVersion>> {
        self.os_version.borrow()
    }
}
impl AndroidImg {
    pub fn name(&self) -> Ref<'_, String> {
        self.name.borrow()
    }
}
impl AndroidImg {
    pub fn cmdline(&self) -> Ref<'_, String> {
        self.cmdline.borrow()
    }
}
impl AndroidImg {
    pub fn sha(&self) -> Ref<'_, Vec<u8>> {
        self.sha.borrow()
    }
}
impl AndroidImg {
    pub fn extra_cmdline(&self) -> Ref<'_, String> {
        self.extra_cmdline.borrow()
    }
}
impl AndroidImg {
    pub fn recovery_dtbo(&self) -> Ref<'_, OptRc<AndroidImg_SizeOffset>> {
        self.recovery_dtbo.borrow()
    }
}
impl AndroidImg {
    pub fn boot_header_size(&self) -> Ref<'_, u32> {
        self.boot_header_size.borrow()
    }
}
impl AndroidImg {
    pub fn dtb(&self) -> Ref<'_, OptRc<AndroidImg_LoadLong>> {
        self.dtb.borrow()
    }
}
impl AndroidImg {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidImg_Load {
    pub(crate) _root: SharedType<AndroidImg>,
    pub(crate) _parent: SharedType<AndroidImg>,
    pub(crate) _self_shared: SharedType<Self>,
    size: RefCell<u32>,
    addr: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidImg_Load {
    type Root = AndroidImg;
    type Parent = AndroidImg;

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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.addr.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl AndroidImg_Load {
}
impl AndroidImg_Load {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl AndroidImg_Load {
    pub fn addr(&self) -> Ref<'_, u32> {
        self.addr.borrow()
    }
}
impl AndroidImg_Load {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidImg_LoadLong {
    pub(crate) _root: SharedType<AndroidImg>,
    pub(crate) _parent: SharedType<AndroidImg>,
    pub(crate) _self_shared: SharedType<Self>,
    size: RefCell<u32>,
    addr: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidImg_LoadLong {
    type Root = AndroidImg;
    type Parent = AndroidImg;

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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.addr.borrow_mut() = _io.read_u8le()?;
        Ok(())
    }
}
impl AndroidImg_LoadLong {
}
impl AndroidImg_LoadLong {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl AndroidImg_LoadLong {
    pub fn addr(&self) -> Ref<'_, u64> {
        self.addr.borrow()
    }
}
impl AndroidImg_LoadLong {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidImg_OsVersion {
    pub(crate) _root: SharedType<AndroidImg>,
    pub(crate) _parent: SharedType<AndroidImg>,
    pub(crate) _self_shared: SharedType<Self>,
    version: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_major: Cell<bool>,
    major: RefCell<i32>,
    f_minor: Cell<bool>,
    minor: RefCell<i32>,
    f_month: Cell<bool>,
    month: RefCell<i32>,
    f_patch: Cell<bool>,
    patch: RefCell<i32>,
    f_year: Cell<bool>,
    year: RefCell<i32>,
}
impl KStruct for AndroidImg_OsVersion {
    type Root = AndroidImg;
    type Parent = AndroidImg;

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
        *self_rc.version.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl AndroidImg_OsVersion {
    pub fn major(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_major.get() {
            return Ok(self.major.borrow());
        }
        self.f_major.set(true);
        *self.major.borrow_mut() = (((((((*self.version()) as u32) >> ((25) as u32))) as u32) & ((127) as u32))).try_into()?;
        Ok(self.major.borrow())
    }
    pub fn minor(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_minor.get() {
            return Ok(self.minor.borrow());
        }
        self.f_minor.set(true);
        *self.minor.borrow_mut() = (((((((*self.version()) as u32) >> ((18) as u32))) as u32) & ((127) as u32))).try_into()?;
        Ok(self.minor.borrow())
    }
    pub fn month(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_month.get() {
            return Ok(self.month.borrow());
        }
        self.f_month.set(true);
        *self.month.borrow_mut() = ((((*self.version()) as u32) & ((15) as u32))).try_into()?;
        Ok(self.month.borrow())
    }
    pub fn patch(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_patch.get() {
            return Ok(self.patch.borrow());
        }
        self.f_patch.set(true);
        *self.patch.borrow_mut() = (((((((*self.version()) as u32) >> ((11) as u32))) as u32) & ((127) as u32))).try_into()?;
        Ok(self.patch.borrow())
    }
    pub fn year(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_year.get() {
            return Ok(self.year.borrow());
        }
        self.f_year.set(true);
        *self.year.borrow_mut() = ((((((((((*self.version()) as u32) >> ((4) as u32))) as u32) & ((127) as u32))) as u32) + ((2000) as u32))).try_into()?;
        Ok(self.year.borrow())
    }
}
impl AndroidImg_OsVersion {
    pub fn version(&self) -> Ref<'_, u32> {
        self.version.borrow()
    }
}
impl AndroidImg_OsVersion {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndroidImg_SizeOffset {
    pub(crate) _root: SharedType<AndroidImg>,
    pub(crate) _parent: SharedType<AndroidImg>,
    pub(crate) _self_shared: SharedType<Self>,
    size: RefCell<u32>,
    offset: RefCell<u64>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndroidImg_SizeOffset {
    type Root = AndroidImg;
    type Parent = AndroidImg;

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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.offset.borrow_mut() = _io.read_u8le()?;
        Ok(())
    }
}
impl AndroidImg_SizeOffset {
}
impl AndroidImg_SizeOffset {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl AndroidImg_SizeOffset {
    pub fn offset(&self) -> Ref<'_, u64> {
        self.offset.borrow()
    }
}
impl AndroidImg_SizeOffset {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
