// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * Resource file found in CPB firmware archives, mostly used on older CoolPad
 * phones and/or tablets. The only observed files are called "ResPack.cfg".
 */

#[derive(Default, Debug, Clone)]
pub struct Respack {
    pub(crate) _root: SharedType<Respack>,
    pub(crate) _parent: SharedType<Respack>,
    pub(crate) _self_shared: SharedType<Self>,
    header: RefCell<OptRc<Respack_Header>>,
    json: RefCell<String>,
    _io: RefCell<BytesReader>,
    json_raw: RefCell<Vec<u8>>,
}
impl KStruct for Respack {
    type Root = Respack;
    type Parent = Respack;

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
        let t = Self::read_into::<_, Respack_Header>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.header.borrow_mut() = t;
        *self_rc.json.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.header().len_json())?)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Respack {
}
impl Respack {
    pub fn header(&self) -> Ref<'_, OptRc<Respack_Header>> {
        self.header.borrow()
    }
}
impl Respack {
    pub fn json(&self) -> Ref<'_, String> {
        self.json.borrow()
    }
}
impl Respack {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Respack {
    pub fn json_raw(&self) -> Ref<'_, Vec<u8>> {
        self.json_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Respack_Header {
    pub(crate) _root: SharedType<Respack>,
    pub(crate) _parent: SharedType<Respack>,
    pub(crate) _self_shared: SharedType<Self>,
    magic: RefCell<Vec<u8>>,
    unknown: RefCell<Vec<u8>>,
    len_json: RefCell<u32>,
    md5: RefCell<String>,
    _io: RefCell<BytesReader>,
    unknown_raw: RefCell<Vec<u8>>,
    md5_raw: RefCell<Vec<u8>>,
}
impl KStruct for Respack_Header {
    type Root = Respack;
    type Parent = Respack;

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
        *self_rc.magic.borrow_mut() = _io.read_bytes(2_usize)?;
        if !(*self_rc.magic() == vec![0x52u8, 0x53u8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotEqual, src_path: "/types/header/seq/0".to_string() }));
        }
        *self_rc.unknown.borrow_mut() = _io.read_bytes(8_usize)?;
        *self_rc.len_json.borrow_mut() = _io.read_u4le()?;
        *self_rc.md5.borrow_mut() = bytes_to_str(&_io.read_bytes(32_usize)?, "UTF-8")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Respack_Header {
}
impl Respack_Header {
    pub fn magic(&self) -> Ref<'_, Vec<u8>> {
        self.magic.borrow()
    }
}
impl Respack_Header {
    pub fn unknown(&self) -> Ref<'_, Vec<u8>> {
        self.unknown.borrow()
    }
}
impl Respack_Header {
    pub fn len_json(&self) -> Ref<'_, u32> {
        self.len_json.borrow()
    }
}

/**
 * MD5 of data that follows the header
 */
impl Respack_Header {
    pub fn md5(&self) -> Ref<'_, String> {
        self.md5.borrow()
    }
}
impl Respack_Header {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Respack_Header {
    pub fn unknown_raw(&self) -> Ref<'_, Vec<u8>> {
        self.unknown_raw.borrow()
    }
}
impl Respack_Header {
    pub fn md5_raw(&self) -> Ref<'_, Vec<u8>> {
        self.md5_raw.borrow()
    }
}
