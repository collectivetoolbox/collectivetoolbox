// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Firmware image found with MediaTek MT76xx wifi chipsets.
 */

#[derive(Default, Debug, Clone)]
pub struct AndesFirmware {
    pub(crate) _root: SharedType<AndesFirmware>,
    pub(crate) _parent: SharedType<AndesFirmware>,
    pub(crate) _self_shared: SharedType<Self>,
    image_header: RefCell<OptRc<AndesFirmware_ImageHeader>>,
    ilm: RefCell<Vec<u8>>,
    dlm: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndesFirmware {
    type Root = AndesFirmware;
    type Parent = AndesFirmware;

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
        let t = Self::read_into::<_, AndesFirmware_ImageHeader>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.image_header.borrow_mut() = t;
        *self_rc.ilm.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.image_header().ilm_len())?)?;
        *self_rc.dlm.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.image_header().dlm_len())?)?;
        Ok(())
    }
}
impl AndesFirmware {
}
impl AndesFirmware {
    pub fn image_header(&self) -> Ref<'_, OptRc<AndesFirmware_ImageHeader>> {
        self.image_header.borrow()
    }
}
impl AndesFirmware {
    pub fn ilm(&self) -> Ref<'_, Vec<u8>> {
        self.ilm.borrow()
    }
}
impl AndesFirmware {
    pub fn dlm(&self) -> Ref<'_, Vec<u8>> {
        self.dlm.borrow()
    }
}
impl AndesFirmware {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct AndesFirmware_ImageHeader {
    pub(crate) _root: SharedType<AndesFirmware>,
    pub(crate) _parent: SharedType<AndesFirmware>,
    pub(crate) _self_shared: SharedType<Self>,
    ilm_len: RefCell<u32>,
    dlm_len: RefCell<u32>,
    fw_ver: RefCell<u16>,
    build_ver: RefCell<u16>,
    extra: RefCell<u32>,
    build_time: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for AndesFirmware_ImageHeader {
    type Root = AndesFirmware;
    type Parent = AndesFirmware;

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
        *self_rc.ilm_len.borrow_mut() = _io.read_u4le()?;
        *self_rc.dlm_len.borrow_mut() = _io.read_u4le()?;
        *self_rc.fw_ver.borrow_mut() = _io.read_u2le()?;
        *self_rc.build_ver.borrow_mut() = _io.read_u2le()?;
        *self_rc.extra.borrow_mut() = _io.read_u4le()?;
        *self_rc.build_time.borrow_mut() = bytes_to_str(&_io.read_bytes(16_usize)?, "UTF-8")?;
        Ok(())
    }
}
impl AndesFirmware_ImageHeader {
}
impl AndesFirmware_ImageHeader {
    pub fn ilm_len(&self) -> Ref<'_, u32> {
        self.ilm_len.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn dlm_len(&self) -> Ref<'_, u32> {
        self.dlm_len.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn fw_ver(&self) -> Ref<'_, u16> {
        self.fw_ver.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn build_ver(&self) -> Ref<'_, u16> {
        self.build_ver.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn extra(&self) -> Ref<'_, u32> {
        self.extra.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn build_time(&self) -> Ref<'_, String> {
        self.build_time.borrow()
    }
}
impl AndesFirmware_ImageHeader {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
