// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * BCD (Binary Coded Decimals) is a common way to encode integer
 * numbers in a way that makes human-readable output somewhat
 * simpler. In this encoding scheme, every decimal digit is encoded as
 * either a single byte (8 bits), or a nibble (half of a byte, 4
 * bits). This obviously wastes a lot of bits, but it makes translation
 * into human-readable string much easier than traditional
 * binary-to-decimal conversion process, which includes lots of
 * divisions by 10.
 *
 * For example, encoding integer 31337 in 8-digit, 8 bits per digit,
 * big endian order of digits BCD format yields
 *
 * ```
 * 00 00 00 03 01 03 03 07
 * ```
 *
 * Encoding the same integer as 8-digit, 4 bits per digit, little
 * endian order BCD format would yield:
 *
 * ```
 * 73 31 30 00
 * ```
 *
 * Using this type of encoding in Kaitai Struct is pretty
 * straightforward: one calls for this type, specifying desired
 * encoding parameters, and gets result using either `as_int` or
 * `as_str` attributes.
 */

#[derive(Default, Debug, Clone)]
pub struct Bcd {
    pub(crate) _root: SharedType<Bcd>,
    pub(crate) _parent: SharedType<Bcd>,
    pub(crate) _self_shared: SharedType<Self>,
    num_digits: RefCell<u8>,
    bits_per_digit: RefCell<u8>,
    is_le: RefCell<bool>,
    digits: RefCell<Vec<Bcd_Digits>>,
    _io: RefCell<BytesReader>,
    f_as_int: Cell<bool>,
    as_int: RefCell<i32>,
    f_as_int_be: Cell<bool>,
    as_int_be: RefCell<i32>,
    f_as_int_le: Cell<bool>,
    as_int_le: RefCell<i32>,
    f_last_idx: Cell<bool>,
    last_idx: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum Bcd_Digits {
    Variant(u64),
    U1(u8),
}
impl From<u64> for Bcd_Digits {
    fn from(v: u64) -> Self {
        Self::Variant(v)
    }
}
impl From<&Bcd_Digits> for u64 {
    fn from(e: &Bcd_Digits) -> Self {
        if let Bcd_Digits::Variant(v) = e {
            return *v
        }
        panic!("trying to convert from enum Bcd_Digits::Variant to u64, enum value {:?}", e)
    }
}
impl From<u8> for Bcd_Digits {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<&Bcd_Digits> for u8 {
    fn from(e: &Bcd_Digits) -> Self {
        if let Bcd_Digits::U1(v) = e {
            return *v
        }
        panic!("trying to convert from enum Bcd_Digits::U1 to u8, enum value {:?}", e)
    }
}
impl From<&Bcd_Digits> for usize {
    fn from(e: &Bcd_Digits) -> Self {
        match e {
            // Reason for fallback: invalid enum conversion to usize defaults to 0
            Bcd_Digits::Variant(v) => usize::try_from(*v).unwrap_or(0),
            // Reason for fallback: invalid enum conversion to usize defaults to 0
            Bcd_Digits::U1(v) => usize::try_from(*v).unwrap_or(0),
        }
    }
}

impl KStruct for Bcd {
    type Root = Bcd;
    type Parent = Bcd;

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
        *self_rc.digits.borrow_mut() = Vec::new();
        let l_digits = *self_rc.num_digits();
        for _i in 0..l_digits {
            match *self_rc.bits_per_digit() {
                4 => {
                    self_rc.digits.borrow_mut().push(_io.read_bits_int_be(4)?.into());
                }
                8 => {
                    self_rc.digits.borrow_mut().push(_io.read_u1()?.into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}
impl Bcd {
    pub fn num_digits(&self) -> Ref<'_, u8> {
        self.num_digits.borrow()
    }
}
impl Bcd {
    pub fn bits_per_digit(&self) -> Ref<'_, u8> {
        self.bits_per_digit.borrow()
    }
}
impl Bcd {
    pub fn is_le(&self) -> Ref<'_, bool> {
        self.is_le.borrow()
    }
}
impl Bcd {
    pub fn set_params(&mut self, num_digits: u8, bits_per_digit: u8, is_le: bool) {
        *self.num_digits.borrow_mut() = num_digits;
        *self.bits_per_digit.borrow_mut() = bits_per_digit;
        *self.is_le.borrow_mut() = is_le;
    }
}
impl Bcd {

    /**
     * Value of this BCD number as integer. Endianness would be selected based on `is_le` parameter given.
     */
    pub fn as_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int.get() {
            return Ok(self.as_int.borrow());
        }
        self.f_as_int.set(true);
        *self.as_int.borrow_mut() = (if *self.is_le() { (*self.as_int_le()?) as i32 } else { (*self.as_int_be()?) as i32 }).try_into()?;
        Ok(self.as_int.borrow())
    }

    /**
     * Value of this BCD number as integer (treating digit order as big-endian).
     */
    pub fn as_int_be(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int_be.get() {
            return Ok(self.as_int_be.borrow());
        }
        self.f_as_int_be.set(true);
        *self.as_int_be.borrow_mut() = ((((usize::from(&self.digits()[*self.last_idx()? as usize])) as u64) + ((if (((*self.num_digits()) as i32) < ((2) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((1) as i32))) as usize])) as u64) * ((10) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((3) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((2) as i32))) as usize])) as u64) * ((100) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((4) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((3) as i32))) as usize])) as u64) * ((1000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((5) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((4) as i32))) as usize])) as u64) * ((10000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((6) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((5) as i32))) as usize])) as u64) * ((100000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((7) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((6) as i32))) as usize])) as u64) * ((1000000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((8) as i32)) { (0) as u64 } else { ((((usize::from(&self.digits()[((((*self.last_idx()?) as i32) - ((7) as i32))) as usize])) as u64) * ((10000000) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))).try_into()?;
        Ok(self.as_int_be.borrow())
    }

    /**
     * Value of this BCD number as integer (treating digit order as little-endian).
     */
    pub fn as_int_le(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int_le.get() {
            return Ok(self.as_int_le.borrow());
        }
        self.f_as_int_le.set(true);
        *self.as_int_le.borrow_mut() = ((((usize::from(&self.digits()[0 as usize])) as u64) + ((if (((*self.num_digits()) as i32) < ((2) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[1 as usize])) as u64) * ((10) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((3) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[2 as usize])) as u64) * ((100) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((4) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[3 as usize])) as u64) * ((1000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((5) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[4 as usize])) as u64) * ((10000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((6) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[5 as usize])) as u64) * ((100000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((7) as i32)) { (0) as u64 } else { (((((((usize::from(&self.digits()[6 as usize])) as u64) * ((1000000) as u64))) as u64) + ((if (((*self.num_digits()) as i32) < ((8) as i32)) { (0) as u64 } else { ((((usize::from(&self.digits()[7 as usize])) as u64) * ((10000000) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))) as u64 }) as u64))).try_into()?;
        Ok(self.as_int_le.borrow())
    }

    /**
     * Index of last digit (0-based).
     */
    pub fn last_idx(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_last_idx.get() {
            return Ok(self.last_idx.borrow());
        }
        self.f_last_idx.set(true);
        *self.last_idx.borrow_mut() = ((((*self.num_digits()) as i32) - ((1) as i32))).try_into()?;
        Ok(self.last_idx.borrow())
    }
}
impl Bcd {
    pub fn digits(&self) -> Ref<'_, Vec<Bcd_Digits>> {
        self.digits.borrow()
    }
}
impl Bcd {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
