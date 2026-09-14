// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * DOS MZ file format is a traditional format for executables in MS-DOS
 * environment. Many modern formats (i.e. Windows PE) still maintain
 * compatibility stub with this format.
 *
 * As opposed to .com file format (which basically sports one 64K code
 * segment of raw CPU instructions), DOS MZ .exe file format allowed
 * more flexible memory management, loading of larger programs and
 * added support for relocations.
 * \sa <http://www.delorie.com/djgpp/doc/exe/> Source
 */

#[derive(Default, Debug, Clone)]
pub struct DosMz {
    pub(crate) _root: SharedType<DosMz>,
    pub(crate) _parent: SharedType<DosMz>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<DosMz_ExeHeader>>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    body_raw: RefCell<Vec<u8>>,
    f_relocations: Cell<bool>,
    relocations: RefCell<Vec<OptRc<DosMz_Relocation>>>,
}
impl KStruct for DosMz {
    type Root = DosMz;
    type Parent = DosMz;

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
        let t = Self::read_into::<_, DosMz_ExeHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.header().len_body()?)?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DosMz {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn relocations(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<DosMz_Relocation>>>> {
        let _io = self._io.borrow();
        if self.f_relocations.get() {
            return Ok(self.relocations.borrow());
        }
        self.f_relocations.set(true);
        if ((to_i128(*self.header().mz().ofs_relocations())) != (to_i128(0))) {
            let io = KStream::clone(&*self.header()._io());
            let _pos = io.pos();
            io.seek(usize::from(*self.header().mz().ofs_relocations()))?;
            *self.relocations.borrow_mut() = Vec::new();
            let l_relocations = usize::from(*self.header().mz().num_relocations());
            for _i in 0_usize..l_relocations {
                let t = Self::read_into::<_, DosMz_Relocation>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
                self.relocations.borrow_mut().push(t);
            }
            io.seek(_pos)?;
        }
        Ok(self.relocations.borrow())
    }
}
impl DosMz {
    pub fn header(&self) -> Ref<'_, OptRc<DosMz_ExeHeader>> {
        self.header.borrow()
    }
}
impl DosMz {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl DosMz {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DosMz {
    pub fn body_raw(&self) -> Ref<'_, Vec<u8>> {
        self.body_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DosMz_ExeHeader {
    pub(crate) _root: SharedType<DosMz>,
    pub(crate) _parent: SharedType<DosMz>,
    pub(crate) _self_shared: SharedType<Self>,
    mz: RefCell<OptRc<DosMz_MzHeader>>,
    rest_of_header: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    rest_of_header_raw: RefCell<Vec<u8>>,
    f_len_body: Cell<bool>,
    len_body: RefCell<i32>,
}
impl KStruct for DosMz_ExeHeader {
    type Root = DosMz;
    type Parent = DosMz;

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
        let t = Self::read_into::<_, DosMz_MzHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.mz.borrow_mut() = t;
        *self_rc.rest_of_header.borrow_mut() = _io.read_bytes(usize::try_from((*self_rc.mz().len_header()?).saturating_sub(28_i32))?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DosMz_ExeHeader {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn len_body(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_body.get() {
            return Ok(self.len_body.borrow());
        }
        self.f_len_body.set(true);
        *self.len_body.borrow_mut() = ((if ((to_i128(*self.mz().last_page_extra_bytes())) == (to_i128(0))) { (i32::from(*self.mz().num_pages())).saturating_mul(512_i32) } else { (((i32::from(*self.mz().num_pages())).saturating_sub(1_i32)).saturating_mul(512_i32)).saturating_add(i32::from(*self.mz().last_page_extra_bytes())) }).saturating_sub(*self.mz().len_header()?)).try_into()?;
        Ok(self.len_body.borrow())
    }
}
impl DosMz_ExeHeader {
    pub fn mz(&self) -> Ref<'_, OptRc<DosMz_MzHeader>> {
        self.mz.borrow()
    }
}
impl DosMz_ExeHeader {
    pub fn rest_of_header(&self) -> Ref<'_, Vec<u8>> {
        self.rest_of_header.borrow()
    }
}
impl DosMz_ExeHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DosMz_ExeHeader {
    pub fn rest_of_header_raw(&self) -> Ref<'_, Vec<u8>> {
        self.rest_of_header_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DosMz_MzHeader {
    pub(crate) _root: SharedType<DosMz>,
    pub(crate) _parent: SharedType<DosMz_ExeHeader>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<String>,
    last_page_extra_bytes: RefCell<u16>,
    num_pages: RefCell<u16>,
    num_relocations: RefCell<u16>,
    header_size: RefCell<u16>,
    min_allocation: RefCell<u16>,
    max_allocation: RefCell<u16>,
    initial_ss: RefCell<u16>,
    initial_sp: RefCell<u16>,
    checksum: RefCell<u16>,
    initial_ip: RefCell<u16>,
    initial_cs: RefCell<u16>,
    ofs_relocations: RefCell<u16>,
    overlay_id: RefCell<u16>,
    _io: RefCell<BytesReader>,
    magic_raw: RefCell<Vec<u8>>,
    f_len_header: Cell<bool>,
    len_header: RefCell<i32>,
}
impl KStruct for DosMz_MzHeader {
    type Root = DosMz;
    type Parent = DosMz_ExeHeader;

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
        *self_rc.magic.borrow_mut() = bytes_to_str(&_io.read_bytes(2_usize)?, "ASCII")?;
        let _item = &*self_rc.magic();
        if !(_item == "MZ" || _item == "ZM") {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotAnyOf, src_path: "/types/mz_header/seq/0".to_string() }));
        }
        *self_rc.last_page_extra_bytes.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_pages.borrow_mut() = _io.read_u2le()?;
        *self_rc.num_relocations.borrow_mut() = _io.read_u2le()?;
        *self_rc.header_size.borrow_mut() = _io.read_u2le()?;
        *self_rc.min_allocation.borrow_mut() = _io.read_u2le()?;
        *self_rc.max_allocation.borrow_mut() = _io.read_u2le()?;
        *self_rc.initial_ss.borrow_mut() = _io.read_u2le()?;
        *self_rc.initial_sp.borrow_mut() = _io.read_u2le()?;
        *self_rc.checksum.borrow_mut() = _io.read_u2le()?;
        *self_rc.initial_ip.borrow_mut() = _io.read_u2le()?;
        *self_rc.initial_cs.borrow_mut() = _io.read_u2le()?;
        *self_rc.ofs_relocations.borrow_mut() = _io.read_u2le()?;
        *self_rc.overlay_id.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DosMz_MzHeader {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn len_header(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_header.get() {
            return Ok(self.len_header.borrow());
        }
        self.f_len_header.set(true);
        *self.len_header.borrow_mut() = ((i32::from(*self.header_size())).saturating_mul(16_i32)).try_into()?;
        Ok(self.len_header.borrow())
    }
}
impl DosMz_MzHeader {
    pub fn magic(&self) -> Ref<'_, String> {
        self.magic.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn last_page_extra_bytes(&self) -> Ref<'_, u16> {
        self.last_page_extra_bytes.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn num_pages(&self) -> Ref<'_, u16> {
        self.num_pages.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn num_relocations(&self) -> Ref<'_, u16> {
        self.num_relocations.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn header_size(&self) -> Ref<'_, u16> {
        self.header_size.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn min_allocation(&self) -> Ref<'_, u16> {
        self.min_allocation.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn max_allocation(&self) -> Ref<'_, u16> {
        self.max_allocation.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn initial_ss(&self) -> Ref<'_, u16> {
        self.initial_ss.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn initial_sp(&self) -> Ref<'_, u16> {
        self.initial_sp.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn checksum(&self) -> Ref<'_, u16> {
        self.checksum.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn initial_ip(&self) -> Ref<'_, u16> {
        self.initial_ip.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn initial_cs(&self) -> Ref<'_, u16> {
        self.initial_cs.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn ofs_relocations(&self) -> Ref<'_, u16> {
        self.ofs_relocations.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn overlay_id(&self) -> Ref<'_, u16> {
        self.overlay_id.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl DosMz_MzHeader {
    pub fn magic_raw(&self) -> Ref<'_, Vec<u8>> {
        self.magic_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct DosMz_Relocation {
    pub(crate) _root: SharedType<DosMz>,
    pub(crate) _parent: SharedType<DosMz>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs: RefCell<u16>,
    seg: RefCell<u16>,
    _io: RefCell<BytesReader>,
}
impl KStruct for DosMz_Relocation {
    type Root = DosMz;
    type Parent = DosMz;

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
        *self_rc.ofs.borrow_mut() = _io.read_u2le()?;
        *self_rc.seg.borrow_mut() = _io.read_u2le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl DosMz_Relocation {
}
impl DosMz_Relocation {
    pub fn ofs(&self) -> Ref<'_, u16> {
        self.ofs.borrow()
    }
}
impl DosMz_Relocation {
    pub fn seg(&self) -> Ref<'_, u16> {
        self.seg.borrow()
    }
}
impl DosMz_Relocation {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
