// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * CHG is a container format file used by
 * [MONOMAKH-SAPR](https://www.liraland.com/mono/), a software
 * package for analysis & design of reinforced concrete multi-storey
 * buildings with arbitrary configuration in plan.
 *
 * CHG is a simple container, which bundles several project files
 * together.
 *
 * Written and tested by Vladimir Shulzhitskiy, 2017
 */

#[derive(Default, Debug, Clone)]
pub struct MonomakhSaprChg {
    pub(crate) _root: SharedType<MonomakhSaprChg>,
    pub(crate) _parent: SharedType<MonomakhSaprChg>,
    pub(crate) _self_shared: SharedType<Self>,
    title: RefCell<String>,
    ent: RefCell<Vec<OptRc<MonomakhSaprChg_Block>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MonomakhSaprChg {
    type Root = MonomakhSaprChg;
    type Parent = MonomakhSaprChg;

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
        *self_rc.title.borrow_mut() = bytes_to_str(&_io.read_bytes(10_usize)?, "ascii")?;
        *self_rc.ent.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let t = Self::read_into::<_, MonomakhSaprChg_Block>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.ent.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl MonomakhSaprChg {
}
impl MonomakhSaprChg {
    pub fn title(&self) -> Ref<'_, String> {
        self.title.borrow()
    }
}
impl MonomakhSaprChg {
    pub fn ent(&self) -> Ref<'_, Vec<OptRc<MonomakhSaprChg_Block>>> {
        self.ent.borrow()
    }
}
impl MonomakhSaprChg {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct MonomakhSaprChg_Block {
    pub(crate) _root: SharedType<MonomakhSaprChg>,
    pub(crate) _parent: SharedType<MonomakhSaprChg>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<String>,
    file_size: RefCell<u64>,
    file: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for MonomakhSaprChg_Block {
    type Root = MonomakhSaprChg;
    type Parent = MonomakhSaprChg;

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
        *self_rc.header.borrow_mut() = bytes_to_str(&_io.read_bytes(13_usize)?, "ascii")?;
        *self_rc.file_size.borrow_mut() = _io.read_u8le()?;
        *self_rc.file.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.file_size())?)?;
        Ok(())
    }
}
impl MonomakhSaprChg_Block {
}
impl MonomakhSaprChg_Block {
    pub fn header(&self) -> Ref<'_, String> {
        self.header.borrow()
    }
}
impl MonomakhSaprChg_Block {
    pub fn file_size(&self) -> Ref<'_, u64> {
        self.file_size.borrow()
    }
}
impl MonomakhSaprChg_Block {
    pub fn file(&self) -> Ref<'_, Vec<u8>> {
        self.file.borrow()
    }
}
impl MonomakhSaprChg_Block {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
