// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * A simple wrapper which allows to read a UTF-16 encoded string that starts
 * with a byte order mark (BOM). The BOM indicates the endianness of the UTF-16
 * encoding, which can be either big-endian (BE) or little-endian (LE).
 *
 * Use:
 *
 * * `value` to get the string value with BOM stripped, regardless of endianness.
 * * `is_be` and `is_le` to check the endianness indicated by the BOM.
 * * `bom` to check the raw byte order mark.
 * \sa - https://en.wikipedia.org/wiki/Byte_order_mark
 */

#[derive(Default, Debug, Clone)]
pub struct Utf16WithBom {
    pub(crate) _root: SharedType<Utf16WithBom>,
    pub(crate) _parent: SharedType<Utf16WithBom>,
    pub(crate) _self_shared: SharedType<Self>,
    bom: RefCell<Vec<u8>>,
    str_be: RefCell<String>,
    str_le: RefCell<String>,
    _io: RefCell<BytesReader>,
    bom_raw: RefCell<Vec<u8>>,
    f_is_be: Cell<bool>,
    is_be: RefCell<bool>,
    f_is_le: Cell<bool>,
    is_le: RefCell<bool>,
    f_value: Cell<bool>,
    value: RefCell<String>,
}
impl KStruct for Utf16WithBom {
    type Root = Utf16WithBom;
    type Parent = Utf16WithBom;

    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic validation value conversion")]
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
        *self_rc.bom.borrow_mut() = _io.read_bytes(2_usize)?;
        let _item = &*self_rc.bom();
        if !(_item == vec![0xfeu8, 0xffu8] || _item == vec![0xffu8, 0xfeu8]) {
            return Err(KError::ValidationFailed(ValidationFailedError { kind: ValidationKind::NotAnyOf, src_path: "/seq/0".to_string() }));
        }
        if *self_rc.is_be()? {
            *self_rc.str_be.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-16BE")?;
        }
        if *self_rc.is_le()? {
            *self_rc.str_le.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "UTF-16LE")?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Utf16WithBom {

    /**
     * True if the byte order mark indicates big-endian UTF-16 encoding.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_be(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_be.get() {
            return Ok(self.is_be.borrow());
        }
        self.f_is_be.set(true);
        *self.is_be.borrow_mut() = (*self.bom() == vec![0xfeu8, 0xffu8]).try_into()?;
        Ok(self.is_be.borrow())
    }

    /**
     * True if the byte order mark indicates little-endian UTF-16 encoding.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_le(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_le.get() {
            return Ok(self.is_le.borrow());
        }
        self.f_is_le.set(true);
        *self.is_le.borrow_mut() = (*self.bom() == vec![0xffu8, 0xfeu8]).try_into()?;
        Ok(self.is_le.borrow())
    }

    /**
     * The string value with BOM stripped, regardless of endianness.
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn value(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_value.get() {
            return Ok(self.value.borrow());
        }
        self.f_value.set(true);
        *self.value.borrow_mut() = if *self.is_be()? { self.str_be().to_string() } else { self.str_le().to_string() }.to_string();
        Ok(self.value.borrow())
    }
}

/**
 * The byte order mark (BOM) is a special marker at the beginning of the
 * string that indicates the endianness of the UTF-16 encoding. The
 * character U+FEFF is used as the BOM, and its byte representation differs
 * based on endianness:
 *
 * * For big-endian (BE) UTF-16, it's `[0xFE, 0xFF]`
 * * For little-endian (LE) UTF-16, it's `[0xFF, 0xFE]`
 *
 * This implementation checks for the presence of a valid BOM and strips it
 * from the resulting string value.
 */
impl Utf16WithBom {
    pub fn bom(&self) -> Ref<'_, Vec<u8>> {
        self.bom.borrow()
    }
}
impl Utf16WithBom {
    pub fn str_be(&self) -> Ref<'_, String> {
        self.str_be.borrow()
    }
}
impl Utf16WithBom {
    pub fn str_le(&self) -> Ref<'_, String> {
        self.str_le.borrow()
    }
}
impl Utf16WithBom {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Utf16WithBom {
    pub fn bom_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bom_raw.borrow()
    }
}
