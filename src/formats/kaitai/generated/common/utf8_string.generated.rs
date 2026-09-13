// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * UTF-8 is a popular character encoding scheme that allows to
 * represent strings as sequence of code points defined in Unicode
 * standard. Its features are:
 *
 * * variable width (i.e. one code point might be represented by 1 to 4
 *   bytes)
 * * backward compatibility with ASCII
 * * basic validity checking (and thus distinguishing from other legacy
 *   8-bit encodings)
 * * maintaining sort order of codepoints if sorted as a byte array
 *
 * WARNING: For the vast majority of practical purposes of format
 * definitions in Kaitai Struct, you'd likely NOT want to use this and
 * rather just use `type: str` with `encoding: utf-8`. That will use
 * native string implementations, which are most likely more efficient
 * and will give you native language strings, rather than an array of
 * individual codepoints.  This format definition is provided mostly
 * for educational / research purposes.
 */

#[derive(Default, Debug, Clone)]
pub struct Utf8String {
    pub(crate) _root: SharedType<Utf8String>,
    pub(crate) _parent: SharedType<Utf8String>,
    pub(crate) _self_shared: SharedType<Self>,
    codepoints: RefCell<Vec<OptRc<Utf8String_Utf8Codepoint>>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Utf8String {
    type Root = Utf8String;
    type Parent = Utf8String;

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
        *self_rc.codepoints.borrow_mut() = Vec::new();
        {
            let mut _i = 0_usize;
            while !_io.is_eof() {
                let f = |t : &mut Utf8String_Utf8Codepoint| Ok(t.set_params((_io.pos()).try_into().map_err(|_| KError::CastError)?));
                let t = Self::read_into_with_init::<_, Utf8String_Utf8Codepoint>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
                self_rc.codepoints.borrow_mut().push(t);
                _i = _i.saturating_add(1);
            }
        }
        Ok(())
    }
}
impl Utf8String {
}
impl Utf8String {
    pub fn codepoints(&self) -> Ref<'_, Vec<OptRc<Utf8String_Utf8Codepoint>>> {
        self.codepoints.borrow()
    }
}
impl Utf8String {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Utf8String_Utf8Codepoint {
    pub(crate) _root: SharedType<Utf8String>,
    pub(crate) _parent: SharedType<Utf8String>,
    pub(crate) _self_shared: SharedType<Self>,
    ofs: RefCell<u64>,
    bytes: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_byte0: Cell<bool>,
    byte0: RefCell<u8>,
    f_len_bytes: Cell<bool>,
    len_bytes: RefCell<i32>,
    f_raw0: Cell<bool>,
    raw0: RefCell<i32>,
    f_raw1: Cell<bool>,
    raw1: RefCell<i32>,
    f_raw2: Cell<bool>,
    raw2: RefCell<i32>,
    f_raw3: Cell<bool>,
    raw3: RefCell<i32>,
    f_value_as_int: Cell<bool>,
    value_as_int: RefCell<i32>,
}
impl KStruct for Utf8String_Utf8Codepoint {
    type Root = Utf8String;
    type Parent = Utf8String;

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
        *self_rc.bytes.borrow_mut() = _io.read_bytes(usize::try_from(*self_rc.len_bytes()?)?)?;
        Ok(())
    }
}
impl Utf8String_Utf8Codepoint {
    pub fn ofs(&self) -> Ref<'_, u64> {
        self.ofs.borrow()
    }
}
impl Utf8String_Utf8Codepoint {
    pub fn set_params(&mut self, ofs: u64) {
        *self.ofs.borrow_mut() = ofs;
    }
}
impl Utf8String_Utf8Codepoint {
    pub fn byte0(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_byte0.get() {
            return Ok(self.byte0.borrow());
        }
        self.f_byte0.set(true);
        let _pos = _io.pos();
        _io.seek(usize::try_from(*self.ofs())?)?;
        *self.byte0.borrow_mut() = _io.read_u1()?;
        _io.seek(_pos)?;
        Ok(self.byte0.borrow())
    }
    pub fn len_bytes(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_bytes.get() {
            return Ok(self.len_bytes.borrow());
        }
        self.f_len_bytes.set(true);
        *self.len_bytes.borrow_mut() = (if ((i32::from(*self.byte0()?)) & (128_i32)) == 0 { 1_i32 } else { if ((i32::from(*self.byte0()?)) & (224_i32)) == 192 { 2_i32 } else { if ((i32::from(*self.byte0()?)) & (240_i32)) == 224 { 3_i32 } else { if ((i32::from(*self.byte0()?)) & (248_i32)) == 240 { 4_i32 } else { (0_i32).saturating_sub(1) } } } }).try_into()?;
        Ok(self.len_bytes.borrow())
    }
    pub fn raw0(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_raw0.get() {
            return Ok(self.raw0.borrow());
        }
        self.f_raw0.set(true);
        *self.raw0.borrow_mut() = (((i32::from(*(self.bytes().get(0_usize).ok_or(KError::CastError)?))) & (if *self.len_bytes()? == 1 { 127_i32 } else { if *self.len_bytes()? == 2 { 31_i32 } else { if *self.len_bytes()? == 3 { 15_i32 } else { if *self.len_bytes()? == 4 { 7_i32 } else { 0_i32 } } } }))).try_into()?;
        Ok(self.raw0.borrow())
    }
    pub fn raw1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_raw1.get() {
            return Ok(self.raw1.borrow());
        }
        self.f_raw1.set(true);
        if *self.len_bytes()? >= 2 {
            *self.raw1.borrow_mut() = (((i32::from(*(self.bytes().get(1_usize).ok_or(KError::CastError)?))) & (63_i32))).try_into()?;
        }
        Ok(self.raw1.borrow())
    }
    pub fn raw2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_raw2.get() {
            return Ok(self.raw2.borrow());
        }
        self.f_raw2.set(true);
        if *self.len_bytes()? >= 3 {
            *self.raw2.borrow_mut() = (((i32::from(*(self.bytes().get(2_usize).ok_or(KError::CastError)?))) & (63_i32))).try_into()?;
        }
        Ok(self.raw2.borrow())
    }
    pub fn raw3(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_raw3.get() {
            return Ok(self.raw3.borrow());
        }
        self.f_raw3.set(true);
        if *self.len_bytes()? >= 4 {
            *self.raw3.borrow_mut() = (((i32::from(*(self.bytes().get(3_usize).ok_or(KError::CastError)?))) & (63_i32))).try_into()?;
        }
        Ok(self.raw3.borrow())
    }
    pub fn value_as_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_value_as_int.get() {
            return Ok(self.value_as_int.borrow());
        }
        self.f_value_as_int.set(true);
        *self.value_as_int.borrow_mut() = (if *self.len_bytes()? == 1 { *self.raw0()? } else { if *self.len_bytes()? == 2 { (((*self.raw0()?).wrapping_shl(6_u32)) | (*self.raw1()?)) } else { if *self.len_bytes()? == 3 { (((((*self.raw0()?).wrapping_shl(12_u32)) | ((*self.raw1()?).wrapping_shl(6_u32)))) | (*self.raw2()?)) } else { if *self.len_bytes()? == 4 { (((((((*self.raw0()?).wrapping_shl(18_u32)) | ((*self.raw1()?).wrapping_shl(12_u32)))) | ((*self.raw2()?).wrapping_shl(6_u32)))) | (*self.raw3()?)) } else { (0_i32).saturating_sub(1) } } } }).try_into()?;
        Ok(self.value_as_int.borrow())
    }
}
impl Utf8String_Utf8Codepoint {
    pub fn bytes(&self) -> Ref<'_, Vec<u8>> {
        self.bytes.borrow()
    }
}
impl Utf8String_Utf8Codepoint {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
