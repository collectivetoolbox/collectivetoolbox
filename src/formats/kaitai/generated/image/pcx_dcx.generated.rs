// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};
use super::pcx::Pcx;

/**
 * DCX is a simple extension of PCX image format allowing to bundle
 * many PCX images (typically, pages of a document) in one file. It saw
 * some limited use in DOS-era fax software, but was largely
 * superseded with multi-page TIFFs and PDFs since then.
 */

#[derive(Default, Debug, Clone)]
pub struct PcxDcx {
    pub(crate) _root: SharedType<PcxDcx>,
    pub(crate) _parent: SharedType<PcxDcx>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    files: RefCell<Vec<OptRc<PcxDcx_PcxOffset>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for PcxDcx {
    type Root = PcxDcx;
    type Parent = PcxDcx;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(4_usize)?;
        if !(*self_rc.magic() == vec![0xb1u8, 0x68u8, 0xdeu8, 0x3au8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/seq/0".to_string() }));
        }
        *self_rc.files.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            loop {
                let t = Self::read_into::<_, PcxDcx_PcxOffset>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.files.borrow_mut().push(t);
                let _t_files = self_rc.files.borrow();
                let Some(_tmpa) = _t_files.last() else { break; };
                _i = _i.saturating_add(1);
                if *_tmpa.ofs_body() == 0 { break; }
            }
        }
        Ok(())
    }
}
impl PcxDcx {
}
impl PcxDcx {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl PcxDcx {
    pub fn files(&self) -> Ref<'_, Vec<OptRc<PcxDcx_PcxOffset>>> {
        self.files.borrow()
    }
}
impl PcxDcx {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct PcxDcx_PcxOffset {
    pub(crate) _root: SharedType<PcxDcx>,
    pub(crate) _parent: SharedType<PcxDcx>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs_body: RefCell<u32>,
    _io: RefCell<BytesReader>,
    f_body: Cell<bool>,
    body: RefCell<OptRc<Pcx>>,
}
impl KStruct for PcxDcx_PcxOffset {
    type Root = PcxDcx;
    type Parent = PcxDcx;

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
        *self_rc.ofs_body.borrow_mut() = _io.read_u4le()?;
        Ok(())
    }
}
impl PcxDcx_PcxOffset {
    pub fn body(
        &self
    ) -> KResult<Ref<'_, OptRc<Pcx>>> {
        let _io = self._io.borrow();
        if self.f_body.get() {
            return Ok(self.body.borrow());
        }
        if *self.ofs_body() != 0 {
            let _pos = _io.pos();
            _io.seek(usize::try_from(*self.ofs_body())?)?;
            let t = Self::read_into::<_, Pcx>(&*_io, None, None)?.into();
            *self.body.borrow_mut() = t;
            _io.seek(_pos)?;
        }
        Ok(self.body.borrow())
    }
}
impl PcxDcx_PcxOffset {
    pub fn ofs_body(&self) -> Ref<'_, u32> {
        self.ofs_body.borrow()
    }
}
impl PcxDcx_PcxOffset {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
