// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://moddingwiki.shikadi.net/wiki/PAK_Format_(Westwood)> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Dune2Pak {
    pub(crate) _root: SharedType<Dune2Pak>,
    pub(crate) _parent: SharedType<Dune2Pak>,
    pub(crate) _self_shared: SharedType<Self>,
    dir: RefCell<OptRc<Dune2Pak_Files>>,
    _io: RefCell<BytesReader>,
    f_dir_size: Cell<bool>,
    dir_size: RefCell<u32>,
}
impl KStruct for Dune2Pak {
    type Root = Dune2Pak;
    type Parent = Dune2Pak;

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
        let t = Self::read_into::<_, Dune2Pak_Files>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.dir.borrow_mut() = t;
        Ok(())
    }
}
impl Dune2Pak {
    pub fn dir_size(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_dir_size.get() {
            return Ok(self.dir_size.borrow());
        }
        self.f_dir_size.set(true);
        let _pos = _io.pos();
        _io.seek(0_usize)?;
        *self.dir_size.borrow_mut() = _io.read_u4le()?;
        _io.seek(_pos)?;
        Ok(self.dir_size.borrow())
    }
}
impl Dune2Pak {
    pub fn dir(&self) -> Ref<'_, OptRc<Dune2Pak_Files>> {
        self.dir.borrow()
    }
}
impl Dune2Pak {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dune2Pak_File {
    pub(crate) _root: SharedType<Dune2Pak>,
    pub(crate) _parent: SharedType<Dune2Pak_Files>,
    pub(crate) _self_shared: SharedType<Self>,
    idx: RefCell<u32>,
    ofs: RefCell<u32>,
    file_name: RefCell<String>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<Vec<u8>>,
    f_next_ofs: Cell<bool>,
    next_ofs: RefCell<i32>,
    f_next_ofs0: Cell<bool>,
    next_ofs0: RefCell<u32>,
}
impl KStruct for Dune2Pak_File {
    type Root = Dune2Pak;
    type Parent = Dune2Pak_Files;

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
        *self_rc.ofs.borrow_mut() = _io.read_u4le()?;
        if *self_rc.ofs() != 0 {
            *self_rc.file_name.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "UTF-8")?;
        }
        Ok(())
    }
}
impl Dune2Pak_File {
    pub fn idx(&self) -> Ref<'_, u32> {
        self.idx.borrow()
    }
}
impl Dune2Pak_File {
    pub fn set_params(&mut self, idx: u32) {
        *self.idx.borrow_mut() = idx;
    }
}
impl Dune2Pak_File {
    pub fn body(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        self.f_body.set(true);
        if *self.ofs() != 0 {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from(*self.ofs())?)?;
            *self.body.borrow_mut() = io.read_bytes(usize::try_from((u32::try_from(*self.next_ofs()?)?).saturating_sub(*self.ofs()))?)?;
            io.seek(_pos)?;
        }
        Ok(self.body.borrow())
    }
    pub fn next_ofs(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_next_ofs.get() {
            return Ok(self.next_ofs.borrow());
        }
        self.f_next_ofs.set(true);
        if *self.ofs() != 0 {
            *self.next_ofs.borrow_mut() = (if *self.next_ofs0()? == 0 { u32::try_from(self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io().size())? } else { *self.next_ofs0()? }).try_into()?;
        }
        Ok(self.next_ofs.borrow())
    }
    pub fn next_ofs0(
        &self
    ) -> KResult<Ref<'_, u32>> {
        let _io = self._io.borrow();
        if self.f_next_ofs0.get() {
            return Ok(self.next_ofs0.borrow());
        }
        self.f_next_ofs0.set(true);
        if *self.ofs() != 0 {
            *self.next_ofs0.borrow_mut() = (*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?.dir().files().get(usize::try_from((*self.idx()).saturating_add(1_u32))?).ok_or(KError::CastError)?.ofs()).try_into()?;
        }
        Ok(self.next_ofs0.borrow())
    }
}
impl Dune2Pak_File {
    pub fn ofs(&self) -> Ref<'_, u32> {
        self.ofs.borrow()
    }
}
impl Dune2Pak_File {
    pub fn file_name(&self) -> Ref<'_, String> {
        self.file_name.borrow()
    }
}
impl Dune2Pak_File {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Dune2Pak_Files {
    pub(crate) _root: SharedType<Dune2Pak>,
    pub(crate) _parent: SharedType<Dune2Pak>,
    pub(crate) _self_shared: SharedType<Self>,
    files: RefCell<Vec<OptRc<Dune2Pak_File>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Dune2Pak_Files {
    type Root = Dune2Pak;
    type Parent = Dune2Pak;

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
        *self_rc.files.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let f = |t : &mut Dune2Pak_File| Ok(t.set_params((_i).try_into().map_err(|_| KError::CastError)?));
                let t = Self::read_into_with_init::<_, Dune2Pak_File>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
                self_rc.files.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl Dune2Pak_Files {
}
impl Dune2Pak_Files {
    pub fn files(&self) -> Ref<'_, Vec<OptRc<Dune2Pak_File>>> {
        self.files.borrow()
    }
}
impl Dune2Pak_Files {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
