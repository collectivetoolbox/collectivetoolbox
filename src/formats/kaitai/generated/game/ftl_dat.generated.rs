// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct FtlDat {
    pub(crate) _root: SharedType<FtlDat>,
    pub(crate) _parent: SharedType<FtlDat>,
    pub(crate) _self_shared: SharedType<Self>,
    num_files: RefCell<u32>,
    files: RefCell<Vec<OptRc<FtlDat_File>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FtlDat {
    type Root = FtlDat;
    type Parent = FtlDat;

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
        *self_rc.num_files.borrow_mut() = _io.read_u4le()?;
        *self_rc.files.borrow_mut() = Vec::new();
        let l_files = usize::try_from(*self_rc.num_files())?;
        for _i in 0_usize..l_files {
            let t = Self::read_into::<_, FtlDat_File>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
            self_rc.files.borrow_mut().push(t);
        }
        Ok(())
    }
}
impl FtlDat {
}

/**
 * Number of files in the archive
 */
impl FtlDat {
    pub fn num_files(&self) -> Ref<'_, u32> {
        self.num_files.borrow()
    }
}
impl FtlDat {
    pub fn files(&self) -> Ref<'_, Vec<OptRc<FtlDat_File>>> {
        self.files.borrow()
    }
}
impl FtlDat {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FtlDat_File {
    pub(crate) _root: SharedType<FtlDat>,
    pub(crate) _parent: SharedType<FtlDat>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs_meta: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_meta: Cell<bool>,
    meta: RefCell<OptRc<FtlDat_Meta>>,
}
impl KStruct for FtlDat_File {
    type Root = FtlDat;
    type Parent = FtlDat;

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
        *self_rc.ofs_meta.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl FtlDat_File {
    pub fn meta(
        &self
    ) -> KResult<Ref<'_, OptRc<FtlDat_Meta>>> {
        let _io = self._io.borrow();
        if self.f_meta.get() {
            return Ok(self.meta.borrow());
        }
        if *self.ofs_meta() != 0 {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.ofs_meta())?)?;
            let t = Self::read_into::<_, FtlDat_Meta>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
            *self.meta.borrow_mut() = t;
            _io.seek(_pos)?;
        }
        Ok(self.meta.borrow())
    }
}
impl FtlDat_File {
    pub fn ofs_meta(&self) -> Ref<'_, u32> {
        self.ofs_meta.borrow()
    }
}
impl FtlDat_File {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FtlDat_Meta {
    pub(crate) _root: SharedType<FtlDat>,
    pub(crate) _parent: SharedType<FtlDat_File>,
    pub(crate) _self_shared: SharedType<Self>,
    len_file: RefCell<u32>,
    len_filename: RefCell<u32>,
    filename: RefCell<String>,
    body: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for FtlDat_Meta {
    type Root = FtlDat;
    type Parent = FtlDat_File;

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
        *self_rc.len_file.borrow_mut() = _io.read_u4le()?;
        *self_rc.len_filename.borrow_mut() = _io.read_u4le()?;
        *self_rc.filename.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.len_filename())?)?, "UTF-8")?;
        *self_rc.body.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_file())?)?;
        Ok(())
    }
}
impl FtlDat_Meta {
}
impl FtlDat_Meta {
    pub fn len_file(&self) -> Ref<'_, u32> {
        self.len_file.borrow()
    }
}
impl FtlDat_Meta {
    pub fn len_filename(&self) -> Ref<'_, u32> {
        self.len_filename.borrow()
    }
}
impl FtlDat_Meta {
    pub fn filename(&self) -> Ref<'_, String> {
        self.filename.borrow()
    }
}
impl FtlDat_Meta {
    pub fn body(&self) -> Ref<'_, Vec<u8>> {
        self.body.borrow()
    }
}
impl FtlDat_Meta {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
