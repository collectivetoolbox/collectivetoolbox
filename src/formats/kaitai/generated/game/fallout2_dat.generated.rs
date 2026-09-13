// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct Fallout2Dat {
    pub(crate) _root: SharedType<Fallout2Dat>,
    pub(crate) _parent: SharedType<Fallout2Dat>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_footer: Cell<bool>,
    footer: RefCell<OptRc<Fallout2Dat_Footer>>,
    f_index: Cell<bool>,
    index: RefCell<OptRc<Fallout2Dat_Index>>,
}
impl KStruct for Fallout2Dat {
    type Root = Fallout2Dat;
    type Parent = Fallout2Dat;

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
        Ok(())
    }
}
impl Fallout2Dat {
    pub fn footer(
        &self
    ) -> KResult<Ref<'_, OptRc<Fallout2Dat_Footer>>> {
        let _io = self._io.borrow();
        if self.f_footer.get() {
            return Ok(self.footer.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from((((_io.size()) as i32) - ((8) as i32)))?)?;
        let t = Self::read_into::<_, Fallout2Dat_Footer>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.footer.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.footer.borrow())
    }
    pub fn index(
        &self
    ) -> KResult<Ref<'_, OptRc<Fallout2Dat_Index>>> {
        let _io = self._io.borrow();
        if self.f_index.get() {
            return Ok(self.index.borrow());
        }
        let _pos = _io.pos();
        _io.seek(usize::try_from(((((((_io.size()) as i32) - ((8) as i32))) as u32) - ((*self.footer()?.index_size()) as u32)))?)?;
        let t = Self::read_into::<_, Fallout2Dat_Index>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.index.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.index.borrow())
    }
}
impl Fallout2Dat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Fallout2Dat_Compression {
    None,
    Zlib,
    Unknown(i64),
}

impl TryFrom<i64> for Fallout2Dat_Compression {
    type Error = KError;
    fn try_from(flag: i64) -> KResult<Fallout2Dat_Compression> {
        match flag {
            0 => Ok(Fallout2Dat_Compression::None),
            1 => Ok(Fallout2Dat_Compression::Zlib),
            _ => Ok(Fallout2Dat_Compression::Unknown(flag)),
        }
    }
}

impl From<&Fallout2Dat_Compression> for i64 {
    fn from(v: &Fallout2Dat_Compression) -> Self {
        match *v {
            Fallout2Dat_Compression::None => 0,
            Fallout2Dat_Compression::Zlib => 1,
            Fallout2Dat_Compression::Unknown(v) => v
        }
    }
}

impl Default for Fallout2Dat_Compression {
    fn default() -> Self { Fallout2Dat_Compression::Unknown(0) }
}


#[derive(Default, Debug, Clone)]
pub struct Fallout2Dat_File {
    pub(crate) _root: SharedType<Fallout2Dat>,
    pub(crate) _parent: SharedType<Fallout2Dat_Index>,
    pub(crate) _self_shared: SharedType<Self>,
    name: RefCell<OptRc<Fallout2Dat_Pstr>>,
    flags: RefCell<Fallout2Dat_Compression>,
    size_unpacked: RefCell<u32>,
    size_packed: RefCell<u32>,
    offset: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_contents: Cell<bool>,
    contents: RefCell<Vec<u8>>,
    f_contents_raw: Cell<bool>,
    contents_raw: RefCell<Vec<u8>>,
    f_contents_zlib: Cell<bool>,
    contents_zlib: RefCell<Vec<u8>>,
}
impl KStruct for Fallout2Dat_File {
    type Root = Fallout2Dat;
    type Parent = Fallout2Dat_Index;

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
        let t = Self::read_into::<_, Fallout2Dat_Pstr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.name.borrow_mut() = t;
        *self_rc.flags.borrow_mut() = i64::try_from(_io.read_u1()?)?.try_into()?;
        *self_rc.size_unpacked.borrow_mut() = _io.read_u4le()?;
        *self_rc.size_packed.borrow_mut() = _io.read_u4le()?;
        *self_rc.offset.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl Fallout2Dat_File {
    pub fn contents(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_contents.get() {
            return Ok(self.contents.borrow());
        }
        self.f_contents.set(true);
        if  ((*self.flags() == Fallout2Dat_Compression::Zlib) || (*self.flags() == Fallout2Dat_Compression::None))  {
            *self.contents.borrow_mut() = if *self.flags() == Fallout2Dat_Compression::Zlib { self.contents_zlib()?.to_vec() } else { self.contents_raw()?.to_vec() }.to_vec();
        }
        Ok(self.contents.borrow())
    }
    pub fn contents_raw(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_contents_raw.get() {
            return Ok(self.contents_raw.borrow());
        }
        self.f_contents_raw.set(true);
        if *self.flags() == Fallout2Dat_Compression::None {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from(*self.offset())?)?;
            *self.contents_raw.borrow_mut() = io.read_bytes(usize::try_from(*self.size_unpacked())?)?;
            io.seek(_pos)?;
        }
        Ok(self.contents_raw.borrow())
    }
    pub fn contents_zlib(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_contents_zlib.get() {
            return Ok(self.contents_zlib.borrow());
        }
        self.f_contents_zlib.set(true);
        if *self.flags() == Fallout2Dat_Compression::Zlib {
            let io = KStream::clone(&*self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?._io());
            let _pos = io.pos();
            io.seek(usize::try_from(*self.offset())?)?;
            *self.contents_zlib.borrow_mut() = io.read_bytes(usize::try_from(*self.size_packed())?)?;
            io.seek(_pos)?;
        }
        Ok(self.contents_zlib.borrow())
    }
}
impl Fallout2Dat_File {
    pub fn name(&self) -> Ref<'_, OptRc<Fallout2Dat_Pstr>> {
        self.name.borrow()
    }
}
impl Fallout2Dat_File {
    pub fn flags(&self) -> Ref<'_, Fallout2Dat_Compression> {
        self.flags.borrow()
    }
}
impl Fallout2Dat_File {
    pub fn size_unpacked(&self) -> Ref<'_, u32> {
        self.size_unpacked.borrow()
    }
}
impl Fallout2Dat_File {
    pub fn size_packed(&self) -> Ref<'_, u32> {
        self.size_packed.borrow()
    }
}
impl Fallout2Dat_File {
    pub fn offset(&self) -> Ref<'_, u32> {
        self.offset.borrow()
    }
}
impl Fallout2Dat_File {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Fallout2Dat_Footer {
    pub(crate) _root: SharedType<Fallout2Dat>,
    pub(crate) _parent: SharedType<Fallout2Dat>,
    pub(crate) _self_shared: SharedType<Self>,
    index_size: RefCell<u32>,
    file_size: RefCell<u32>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Fallout2Dat_Footer {
    type Root = Fallout2Dat;
    type Parent = Fallout2Dat;

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
        *self_rc.index_size.borrow_mut() = _io.read_u4le()?;
        *self_rc.file_size.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl Fallout2Dat_Footer {
}
impl Fallout2Dat_Footer {
    pub fn index_size(&self) -> Ref<'_, u32> {
        self.index_size.borrow()
    }
}
impl Fallout2Dat_Footer {
    pub fn file_size(&self) -> Ref<'_, u32> {
        self.file_size.borrow()
    }
}
impl Fallout2Dat_Footer {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Fallout2Dat_Index {
    pub(crate) _root: SharedType<Fallout2Dat>,
    pub(crate) _parent: SharedType<Fallout2Dat>,
    pub(crate) _self_shared: SharedType<Self>,
    file_count: RefCell<u32>,
    files: RefCell<Vec<OptRc<Fallout2Dat_File>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Fallout2Dat_Index {
    type Root = Fallout2Dat;
    type Parent = Fallout2Dat;

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
        *self_rc.file_count.borrow_mut() = _io.read_u4le()?;
        *self_rc.files.borrow_mut() = Vec::new();
        let l_files = *self_rc.file_count();
        for _i in 0..l_files {
            let t = Self::read_into::<_, Fallout2Dat_File>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.files.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl Fallout2Dat_Index {
}
impl Fallout2Dat_Index {
    pub fn file_count(&self) -> Ref<'_, u32> {
        self.file_count.borrow()
    }
}
impl Fallout2Dat_Index {
    pub fn files(&self) -> Ref<'_, Vec<OptRc<Fallout2Dat_File>>> {
        self.files.borrow()
    }
}
impl Fallout2Dat_Index {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Fallout2Dat_Pstr {
    pub(crate) _root: SharedType<Fallout2Dat>,
    pub(crate) _parent: SharedType<Fallout2Dat_File>,
    pub(crate) _self_shared: SharedType<Self>,
    size: RefCell<u32>,
    str: RefCell<String>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Fallout2Dat_Pstr {
    type Root = Fallout2Dat;
    type Parent = Fallout2Dat_File;

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
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.size())?)?, "ASCII")?;
        Ok(())
    }
}
impl Fallout2Dat_Pstr {
}
impl Fallout2Dat_Pstr {
    pub fn size(&self) -> Ref<'_, u32> {
        self.size.borrow()
    }
}
impl Fallout2Dat_Pstr {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl Fallout2Dat_Pstr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
