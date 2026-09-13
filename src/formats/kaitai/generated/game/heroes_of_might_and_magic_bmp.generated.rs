// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct HeroesOfMightAndMagicBmp {
    pub(crate) _root: SharedType<HeroesOfMightAndMagicBmp>,
    pub(crate) _parent: SharedType<HeroesOfMightAndMagicBmp>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<u16>,
    width: RefCell<u16>,
    height: RefCell<u16>,
    data: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for HeroesOfMightAndMagicBmp {
    type Root = HeroesOfMightAndMagicBmp;
    type Parent = HeroesOfMightAndMagicBmp;

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
        *self_rc.magic.borrow_mut() = _io.read_u2le()?;
        *self_rc.width.borrow_mut() = _io.read_u2le()?;
        *self_rc.height.borrow_mut() = _io.read_u2le()?;
        *self_rc.data.borrow_mut() = _io.read_bytes(usize::try_from((((*self_rc.width()) as u16) * ((*self_rc.height()) as u16)))?)?;
        Ok(())
    }
}
impl HeroesOfMightAndMagicBmp {
}
impl HeroesOfMightAndMagicBmp {
    pub fn magic(&self) -> Ref<'_, u16> {
        self.magic.borrow()
    }
}
impl HeroesOfMightAndMagicBmp {
    pub fn width(&self) -> Ref<'_, u16> {
        self.width.borrow()
    }
}
impl HeroesOfMightAndMagicBmp {
    pub fn height(&self) -> Ref<'_, u16> {
        self.height.borrow()
    }
}
impl HeroesOfMightAndMagicBmp {
    pub fn data(&self) -> Ref<'_, Vec<u8>> {
        self.data.borrow()
    }
}
impl HeroesOfMightAndMagicBmp {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
