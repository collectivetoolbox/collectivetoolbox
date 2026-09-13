// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * \sa <https://web.archive.org/web/20170215190034/http://rewiki.regengedanken.de/wiki/.AGG_(Heroes_of_Might_and_Magic)> Source
 */

#[derive(Default, Debug, Clone)]
pub struct HeroesOfMightAndMagicAgg {
    pub(crate) _root: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _parent: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _self_shared: SharedType<Self>,
    num_files: RefCell<u16>,
    entries: RefCell<Vec<OptRc<HeroesOfMightAndMagicAgg_Entry>>>,
    _io: RefCell<BytesReader>,
    filenames_raw: RefCell<Vec<Vec<u8>>>,
    f_filenames: Cell<bool>,
    filenames: RefCell<Vec<OptRc<HeroesOfMightAndMagicAgg_Filename>>>,
}
impl KStruct for HeroesOfMightAndMagicAgg {
    type Root = HeroesOfMightAndMagicAgg;
    type Parent = HeroesOfMightAndMagicAgg;

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
        *self_rc.num_files.borrow_mut() = _io.read_u2le()?;
        *self_rc.entries.borrow_mut() = Vec::new();
        let l_entries = usize::from(*self_rc.num_files());
        for _i in 0_usize..l_entries {
            let t = Self::read_into::<_, HeroesOfMightAndMagicAgg_Entry>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.entries.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl HeroesOfMightAndMagicAgg {
    pub fn filenames(
        &self
    ) -> KResult<Ref<'_, Vec<OptRc<HeroesOfMightAndMagicAgg_Filename>>>> {
        let _io = self._io.borrow();
        if self.f_filenames.get() {
            return Ok(self.filenames.borrow());
        }
        self.f_filenames.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from((*self.entries().last().ok_or(KError::EmptyIterator)?.offset()).saturating_add(*self.entries().last().ok_or(KError::EmptyIterator)?.size()))?)?;
        *self.filenames_raw.borrow_mut() = Vec::new();
        *self.filenames.borrow_mut() = Vec::new();
        let l_filenames = usize::from(*self.num_files());
        for _i in 0_usize..l_filenames {
            self.filenames_raw.borrow_mut().push(_io.read_bytes(15_usize)?.into());
            let filenames_raw = self.filenames_raw.borrow();
            let _io_filenames_raw = BytesReader::from(filenames_raw.last().ok_or(KError::EmptyIterator)?.clone());
            let t = Self::read_into::<BytesReader, HeroesOfMightAndMagicAgg_Filename>(&_io_filenames_raw, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            self.filenames.borrow_mut().push(t);
        }
        _io.seek(_pos)?;
        Ok(self.filenames.borrow())
    }
}
impl HeroesOfMightAndMagicAgg {
    pub fn num_files(&self) -> Ref<'_, u16> {
        self.num_files.borrow()
    }
}
impl HeroesOfMightAndMagicAgg {
    pub fn entries(&self) -> Ref<'_, Vec<OptRc<HeroesOfMightAndMagicAgg_Entry>>> {
        self.entries.borrow()
    }
}
impl HeroesOfMightAndMagicAgg {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl HeroesOfMightAndMagicAgg {
    pub fn filenames_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {
        self.filenames_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct HeroesOfMightAndMagicAgg_Entry {
    pub(crate) _root: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _parent: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _self_shared: SharedType<Self>,
    hash: RefCell<u16>,
    offset: RefCell<u32>,
    size: RefCell<u32>,
    size2: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<Vec<u8>>,
}
impl KStruct for HeroesOfMightAndMagicAgg_Entry {
    type Root = HeroesOfMightAndMagicAgg;
    type Parent = HeroesOfMightAndMagicAgg;

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
        *self_rc.hash.borrow_mut() = _io.read_u2le()?;
        *self_rc.offset.borrow_mut() = _io.read_u4le()?;
        *self_rc.size.borrow_mut() = _io.read_u4le()?;
        *self_rc.size2.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn body(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        self.f_body.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.offset())?)?;
        *self.body.borrow_mut() = _io.read_bytes(usize::try_from(*self.size())?)?;
        _io.seek(_pos)?;
        Ok(self.body.borrow())
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn hash(&self) -> Ref<'_, u16> {
        self.hash.borrow()
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn offset(&self) -> Ref<'_, u32> {
        self.offset.borrow()
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn size2(&self) -> Ref<'_, u32> {
        self.size2.borrow()
    }
}
impl HeroesOfMightAndMagicAgg_Entry {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct HeroesOfMightAndMagicAgg_Filename {
    pub(crate) _root: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _parent: SharedType<HeroesOfMightAndMagicAgg>,
    pub(crate) _self_shared: SharedType<Self>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for HeroesOfMightAndMagicAgg_Filename {
    type Root = HeroesOfMightAndMagicAgg;
    type Parent = HeroesOfMightAndMagicAgg;

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
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes_term(0, false, true, true)?, "ASCII")?;
        Ok(())
    }
}
impl HeroesOfMightAndMagicAgg_Filename {
}
impl HeroesOfMightAndMagicAgg_Filename {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl HeroesOfMightAndMagicAgg_Filename {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
