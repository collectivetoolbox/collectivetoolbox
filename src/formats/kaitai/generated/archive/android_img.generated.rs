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
    name_raw: RefCell<Vec<u8>>,
    cmdline_raw: RefCell<Vec<u8>>,
    sha_raw: RefCell<Vec<u8>>,
    extra_cmdline_raw: RefCell<Vec<u8>>,
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
        *self_rc.magic.borrow_mut() = _io.read_bytes(8_usize)?;
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
        *self_rc.name.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(16_usize)?, Some(0), false, None), "ASCII")?;
        *self_rc.cmdline.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(512_usize)?, Some(0), false, None), "ASCII")?;
        *self_rc.sha.borrow_mut() = _io.read_bytes(32_usize)?;
        *self_rc.extra_cmdline.borrow_mut() = bytes_to_str(&bytes_terminate_pad(&_io.read_bytes(1024_usize)?, Some(0), false, None), "ASCII")?;
        if ((to_i128(*self_rc.header_version())) > (to_i128(0))) {
            let t = Self::read_into::<_, AndroidImg_SizeOffset>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.recovery_dtbo.borrow_mut() = t;
        }
        if ((to_i128(*self_rc.header_version())) > (to_i128(0))) {
            *self_rc.boot_header_size.borrow_mut() = _io.read_u4le()?;
        }
        if ((to_i128(*self_rc.header_version())) > (to_i128(1))) {
            let t = Self::read_into::<_, AndroidImg_LoadLong>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            *self_rc.dtb.borrow_mut() = t;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidImg {

    /**
     * base loading address
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn base(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_base.get() {
            return Ok(self.base.borrow());
        }
        self.f_base.set(true);
        *self.base.borrow_mut() = ((*self.kernel().addr()).saturating_sub(32768_u32)).try_into()?;
        Ok(self.base.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn dtb_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_dtb_img.get() {
            return Ok(self.dtb_img.borrow());
        }
        self.f_dtb_img.set(true);
        if  ((((to_i128(*self.header_version())) > (to_i128(1)))) && (((to_i128(*self.dtb().size())) > (to_i128(0)))))  {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((((((((*self.page_size()).saturating_add(*self.kernel().size())).saturating_add(*self.ramdisk().size())).saturating_add(*self.second().size())).saturating_add(*self.recovery_dtbo().size())).saturating_add(*self.page_size())).saturating_sub(1_u32)).checked_div(*self.page_size()).ok_or(KError::CastError)?).saturating_mul(*self.page_size()))?)?;
            *self.dtb_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.dtb().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.dtb_img.borrow())
    }

    /**
     * dtb offset from base
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn dtb_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_dtb_offset.get() {
            return Ok(self.dtb_offset.borrow());
        }
        self.f_dtb_offset.set(true);
        if ((to_i128(*self.header_version())) > (to_i128(1))) {
            *self.dtb_offset.borrow_mut() = (if ((to_i128(*self.dtb().addr())) > (to_i128(0))) { (*self.dtb().addr()).saturating_sub(u64::try_from(*self.base()?)?) } else { 0_u64 }).try_into()?;
        }
        Ok(self.dtb_offset.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
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
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn kernel_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_kernel_offset.get() {
            return Ok(self.kernel_offset.borrow());
        }
        self.f_kernel_offset.set(true);
        *self.kernel_offset.borrow_mut() = ((*self.kernel().addr()).saturating_sub(u32::try_from(*self.base()?)?)).try_into()?;
        Ok(self.kernel_offset.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ramdisk_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_ramdisk_img.get() {
            return Ok(self.ramdisk_img.borrow());
        }
        self.f_ramdisk_img.set(true);
        if ((to_i128(*self.ramdisk().size())) > (to_i128(0))) {
            let _pos = _io.pos();
            _io.seek(usize::try_from((((((*self.page_size()).saturating_add(*self.kernel().size())).saturating_add(*self.page_size())).saturating_sub(1_u32)).checked_div(*self.page_size()).ok_or(KError::CastError)?).saturating_mul(*self.page_size()))?)?;
            *self.ramdisk_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.ramdisk().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.ramdisk_img.borrow())
    }

    /**
     * ramdisk offset from base
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn ramdisk_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_ramdisk_offset.get() {
            return Ok(self.ramdisk_offset.borrow());
        }
        self.f_ramdisk_offset.set(true);
        *self.ramdisk_offset.borrow_mut() = (if ((to_i128(*self.ramdisk().addr())) > (to_i128(0))) { (*self.ramdisk().addr()).saturating_sub(u32::try_from(*self.base()?)?) } else { 0_u32 }).try_into()?;
        Ok(self.ramdisk_offset.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn recovery_dtbo_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_recovery_dtbo_img.get() {
            return Ok(self.recovery_dtbo_img.borrow());
        }
        self.f_recovery_dtbo_img.set(true);
        if  ((((to_i128(*self.header_version())) > (to_i128(0)))) && (((to_i128(*self.recovery_dtbo().size())) > (to_i128(0)))))  {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.recovery_dtbo().offset())?)?;
            *self.recovery_dtbo_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.recovery_dtbo().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.recovery_dtbo_img.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn second_img(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_second_img.get() {
            return Ok(self.second_img.borrow());
        }
        self.f_second_img.set(true);
        if ((to_i128(*self.second().size())) > (to_i128(0))) {
            let _pos = _io.pos();
            _io.seek(usize::try_from(((((((*self.page_size()).saturating_add(*self.kernel().size())).saturating_add(*self.ramdisk().size())).saturating_add(*self.page_size())).saturating_sub(1_u32)).checked_div(*self.page_size()).ok_or(KError::CastError)?).saturating_mul(*self.page_size()))?)?;
            *self.second_img.borrow_mut() = _io.read_bytes(usize::try_from(*self.second().size())?)?;
            _io.seek(_pos)?;
        }
        Ok(self.second_img.borrow())
    }

    /**
     * 2nd bootloader offset from base
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn second_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_second_offset.get() {
            return Ok(self.second_offset.borrow());
        }
        self.f_second_offset.set(true);
        *self.second_offset.borrow_mut() = (if ((to_i128(*self.second().addr())) > (to_i128(0))) { (*self.second().addr()).saturating_sub(u32::try_from(*self.base()?)?) } else { 0_u32 }).try_into()?;
        Ok(self.second_offset.borrow())
    }

    /**
     * tags offset from base
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn tags_offset(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_tags_offset.get() {
            return Ok(self.tags_offset.borrow());
        }
        self.f_tags_offset.set(true);
        *self.tags_offset.borrow_mut() = ((*self.tags_load()).saturating_sub(u32::try_from(*self.base()?)?)).try_into()?;
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
impl AndroidImg {
    pub fn name_raw(&self) -> Ref<'_, Vec<u8>> {
        self.name_raw.borrow()
    }
}
impl AndroidImg {
    pub fn cmdline_raw(&self) -> Ref<'_, Vec<u8>> {
        self.cmdline_raw.borrow()
    }
}
impl AndroidImg {
    pub fn sha_raw(&self) -> Ref<'_, Vec<u8>> {
        self.sha_raw.borrow()
    }
}
impl AndroidImg {
    pub fn extra_cmdline_raw(&self) -> Ref<'_, Vec<u8>> {
        self.extra_cmdline_raw.borrow()
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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.addr.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.addr.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
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
        *self_rc.version.borrow_mut() = _io.read_u4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl AndroidImg_OsVersion {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn major(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_major.get() {
            return Ok(self.major.borrow());
        }
        self.f_major.set(true);
        *self.major.borrow_mut() = ((((*self.version()).wrapping_shr(25_u32)) & (127_u32))).try_into()?;
        Ok(self.major.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn minor(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_minor.get() {
            return Ok(self.minor.borrow());
        }
        self.f_minor.set(true);
        *self.minor.borrow_mut() = ((((*self.version()).wrapping_shr(18_u32)) & (127_u32))).try_into()?;
        Ok(self.minor.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn month(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_month.get() {
            return Ok(self.month.borrow());
        }
        self.f_month.set(true);
        *self.month.borrow_mut() = (((*self.version()) & (15_u32))).try_into()?;
        Ok(self.month.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn patch(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_patch.get() {
            return Ok(self.patch.borrow());
        }
        self.f_patch.set(true);
        *self.patch.borrow_mut() = ((((*self.version()).wrapping_shr(11_u32)) & (127_u32))).try_into()?;
        Ok(self.patch.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn year(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_year.get() {
            return Ok(self.year.borrow());
        }
        self.f_year.set(true);
        *self.year.borrow_mut() = (((((*self.version()).wrapping_shr(4_u32)) & (127_u32))).saturating_add(2000_u32)).try_into()?;
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
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.offset.borrow_mut() = _io.read_u8le()?;
        *self_rc._io.borrow_mut() = io.clone();
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
