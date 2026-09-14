// SPDX-License-Identifier: CC0-1.0
// license-linter:allow-non-AGPL
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

/**
 * MessagePack (msgpack) is a system to serialize arbitrary structured
 * data into a compact binary stream.
 * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md> Source
 */

#[derive(Default, Debug, Clone)]
pub struct Msgpack {
    pub(crate) _root: SharedType<Msgpack>,
    pub(crate) _parent: SharedType<Msgpack>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    int_extra: RefCell<Option<Msgpack_IntExtra>>,
    float_32_value: RefCell<f32>,
    float_64_value: RefCell<f64>,
    str_len_8: RefCell<u8>,
    str_len_16: RefCell<u16>,
    str_len_32: RefCell<u32>,
    str_value: RefCell<String>,
    num_array_elements_16: RefCell<u16>,
    num_array_elements_32: RefCell<u32>,
    array_elements: RefCell<Vec<OptRc<Msgpack>>>,
    num_map_elements_16: RefCell<u16>,
    num_map_elements_32: RefCell<u32>,
    map_elements: RefCell<Vec<OptRc<Msgpack_MapTuple>>>,
    _io: RefCell<BytesReader>,
    str_value_raw: RefCell<Vec<u8>>,
    f_bool_value: Cell<bool>,
    bool_value: RefCell<bool>,
    f_float_value: Cell<bool>,
    float_value: RefCell<f64>,
    f_int_value: Cell<bool>,
    int_value: RefCell<i32>,
    f_is_array: Cell<bool>,
    is_array: RefCell<bool>,
    f_is_array_16: Cell<bool>,
    is_array_16: RefCell<bool>,
    f_is_array_32: Cell<bool>,
    is_array_32: RefCell<bool>,
    f_is_bool: Cell<bool>,
    is_bool: RefCell<bool>,
    f_is_fix_array: Cell<bool>,
    is_fix_array: RefCell<bool>,
    f_is_fix_map: Cell<bool>,
    is_fix_map: RefCell<bool>,
    f_is_fix_str: Cell<bool>,
    is_fix_str: RefCell<bool>,
    f_is_float: Cell<bool>,
    is_float: RefCell<bool>,
    f_is_float_32: Cell<bool>,
    is_float_32: RefCell<bool>,
    f_is_float_64: Cell<bool>,
    is_float_64: RefCell<bool>,
    f_is_int: Cell<bool>,
    is_int: RefCell<bool>,
    f_is_map: Cell<bool>,
    is_map: RefCell<bool>,
    f_is_map_16: Cell<bool>,
    is_map_16: RefCell<bool>,
    f_is_map_32: Cell<bool>,
    is_map_32: RefCell<bool>,
    f_is_neg_int5: Cell<bool>,
    is_neg_int5: RefCell<bool>,
    f_is_nil: Cell<bool>,
    is_nil: RefCell<bool>,
    f_is_pos_int7: Cell<bool>,
    is_pos_int7: RefCell<bool>,
    f_is_str: Cell<bool>,
    is_str: RefCell<bool>,
    f_is_str_16: Cell<bool>,
    is_str_16: RefCell<bool>,
    f_is_str_32: Cell<bool>,
    is_str_32: RefCell<bool>,
    f_is_str_8: Cell<bool>,
    is_str_8: RefCell<bool>,
    f_neg_int5_value: Cell<bool>,
    neg_int5_value: RefCell<i32>,
    f_num_array_elements: Cell<bool>,
    num_array_elements: RefCell<i32>,
    f_num_map_elements: Cell<bool>,
    num_map_elements: RefCell<i32>,
    f_pos_int7_value: Cell<bool>,
    pos_int7_value: RefCell<u8>,
    f_str_len: Cell<bool>,
    str_len: RefCell<i32>,
}
#[derive(Debug, Clone)]
pub enum Msgpack_IntExtra {
    U1(u8),
    U2(u16),
    U4(u32),
    U8(u64),
    S1(i8),
    S2(i16),
    S4(i32),
    S8(i64),
}
impl From<u8> for Msgpack_IntExtra {
    fn from(v: u8) -> Self {
        Self::U1(v)
    }
}
impl From<u16> for Msgpack_IntExtra {
    fn from(v: u16) -> Self {
        Self::U2(v)
    }
}
impl From<u32> for Msgpack_IntExtra {
    fn from(v: u32) -> Self {
        Self::U4(v)
    }
}
impl From<u64> for Msgpack_IntExtra {
    fn from(v: u64) -> Self {
        Self::U8(v)
    }
}
impl From<i8> for Msgpack_IntExtra {
    fn from(v: i8) -> Self {
        Self::S1(v)
    }
}
impl From<i16> for Msgpack_IntExtra {
    fn from(v: i16) -> Self {
        Self::S2(v)
    }
}
impl From<i32> for Msgpack_IntExtra {
    fn from(v: i32) -> Self {
        Self::S4(v)
    }
}
impl From<i64> for Msgpack_IntExtra {
    fn from(v: i64) -> Self {
        Self::S8(v)
    }
}
impl TryFrom<&Msgpack_IntExtra> for i16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(i16::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(i16::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for i32 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(i32::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(i32::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for i64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(i64::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(i64::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for i8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(i8::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(i8::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for u16 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(u16::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(u16::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for u32 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(u32::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(u32::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for u64 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(u64::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(u64::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for u8 {
    type Error = KError;
    #[allow(clippy::unnecessary_fallible_conversions, reason = "Generic TryFrom implementation over varied enum variant types")]
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::U2(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::U4(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(u8::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(u8::try_from(*v)?),
        }
    }
}
impl TryFrom<&Msgpack_IntExtra> for usize {
    type Error = KError;
    fn try_from(e: &Msgpack_IntExtra) -> Result<Self, Self::Error> {
        match e {
            Msgpack_IntExtra::U1(v) => Ok(usize::from(*v)),
            Msgpack_IntExtra::U2(v) => Ok(usize::from(*v)),
            Msgpack_IntExtra::U4(v) => Ok(usize::try_from(*v)?),
            Msgpack_IntExtra::U8(v) => Ok(usize::try_from(*v)?),
            Msgpack_IntExtra::S1(v) => Ok(usize::try_from(*v)?),
            Msgpack_IntExtra::S2(v) => Ok(usize::try_from(*v)?),
            Msgpack_IntExtra::S4(v) => Ok(usize::try_from(*v)?),
            Msgpack_IntExtra::S8(v) => Ok(usize::try_from(*v)?),
        }
    }
}

impl KStruct for Msgpack {
    type Root = Msgpack;
    type Parent = Msgpack;

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
        *self_rc.b1.borrow_mut() = _io.read_u1()?;
        match *self_rc.b1() {
            204 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_u1()?.into());
            }
            205 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_u2be()?.into());
            }
            206 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_u4be()?.into());
            }
            207 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_u8be()?.into());
            }
            208 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_s1()?.into());
            }
            209 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_s2be()?.into());
            }
            210 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_s4be()?.into());
            }
            211 => {
                *self_rc.int_extra.borrow_mut() = Some(_io.read_s8be()?.into());
            }
            _ => {}
        }
        if *self_rc.is_float_32()? {
            *self_rc.float_32_value.borrow_mut() = _io.read_f4be()?;
        }
        if *self_rc.is_float_64()? {
            *self_rc.float_64_value.borrow_mut() = _io.read_f8be()?;
        }
        if *self_rc.is_str_8()? {
            *self_rc.str_len_8.borrow_mut() = _io.read_u1()?;
        }
        if *self_rc.is_str_16()? {
            *self_rc.str_len_16.borrow_mut() = _io.read_u2be()?;
        }
        if *self_rc.is_str_32()? {
            *self_rc.str_len_32.borrow_mut() = _io.read_u4be()?;
        }
        if *self_rc.is_str()? {
            *self_rc.str_value.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.str_len()?)?)?, "UTF-8")?;
        }
        if *self_rc.is_array_16()? {
            *self_rc.num_array_elements_16.borrow_mut() = _io.read_u2be()?;
        }
        if *self_rc.is_array_32()? {
            *self_rc.num_array_elements_32.borrow_mut() = _io.read_u4be()?;
        }
        if *self_rc.is_array()? {
            *self_rc.array_elements.borrow_mut() = Vec::new();
            let l_array_elements = usize::try_from(*self_rc.num_array_elements()?)?;
            for _i in 0_usize..l_array_elements {
                let t = Self::read_into::<_, Msgpack>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.array_elements.borrow_mut().push(t);
            }
        }
        if *self_rc.is_map_16()? {
            *self_rc.num_map_elements_16.borrow_mut() = _io.read_u2be()?;
        }
        if *self_rc.is_map_32()? {
            *self_rc.num_map_elements_32.borrow_mut() = _io.read_u4be()?;
        }
        if *self_rc.is_map()? {
            *self_rc.map_elements.borrow_mut() = Vec::new();
            let l_map_elements = usize::try_from(*self_rc.num_map_elements()?)?;
            for _i in 0_usize..l_map_elements {
                let t = Self::read_into::<_, Msgpack_MapTuple>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
                self_rc.map_elements.borrow_mut().push(t);
            }
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Msgpack {

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-bool> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn bool_value(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_bool_value.get() {
            return Ok(self.bool_value.borrow());
        }
        self.f_bool_value.set(true);
        if *self.is_bool()? {
            *self.bool_value.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(195)))).try_into()?;
        }
        Ok(self.bool_value.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn float_value(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_float_value.get() {
            return Ok(self.float_value.borrow());
        }
        self.f_float_value.set(true);
        if *self.is_float()? {
            *self.float_value.borrow_mut() = (if *self.is_float_32()? { to_f64(*self.float_32_value()) } else { *self.float_64_value() }).try_into()?;
        }
        Ok(self.float_value.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn int_value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_value.get() {
            return Ok(self.int_value.borrow());
        }
        self.f_int_value.set(true);
        if *self.is_int()? {
            *self.int_value.borrow_mut() = (if *self.is_pos_int7()? { i32::from(*self.pos_int7_value()?) } else { if *self.is_neg_int5()? { *self.neg_int5_value()? } else { 4919_i32 } }).try_into()?;
        }
        Ok(self.int_value.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-array> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_array(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_array.get() {
            return Ok(self.is_array.borrow());
        }
        self.f_is_array.set(true);
        *self.is_array.borrow_mut() = ( ((*self.is_fix_array()?) || (*self.is_array_16()?) || (*self.is_array_32()?)) ).try_into()?;
        Ok(self.is_array.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-array> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_array_16(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_array_16.get() {
            return Ok(self.is_array_16.borrow());
        }
        self.f_is_array_16.set(true);
        *self.is_array_16.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(220)))).try_into()?;
        Ok(self.is_array_16.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-array> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_array_32(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_array_32.get() {
            return Ok(self.is_array_32.borrow());
        }
        self.f_is_array_32.set(true);
        *self.is_array_32.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(221)))).try_into()?;
        Ok(self.is_array_32.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_bool(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_bool.get() {
            return Ok(self.is_bool.borrow());
        }
        self.f_is_bool.set(true);
        *self.is_bool.borrow_mut() = ( ((((to_i128(*self.b1())) == (to_i128(194)))) || (((to_i128(*self.b1())) == (to_i128(195))))) ).try_into()?;
        Ok(self.is_bool.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-array> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_fix_array(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_fix_array.get() {
            return Ok(self.is_fix_array.borrow());
        }
        self.f_is_fix_array.set(true);
        *self.is_fix_array.borrow_mut() = (((i32::from(*self.b1())) & (240_i32)) == 144).try_into()?;
        Ok(self.is_fix_array.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-map> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_fix_map(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_fix_map.get() {
            return Ok(self.is_fix_map.borrow());
        }
        self.f_is_fix_map.set(true);
        *self.is_fix_map.borrow_mut() = (((i32::from(*self.b1())) & (240_i32)) == 128).try_into()?;
        Ok(self.is_fix_map.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-str> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_fix_str(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_fix_str.get() {
            return Ok(self.is_fix_str.borrow());
        }
        self.f_is_fix_str.set(true);
        *self.is_fix_str.borrow_mut() = (((i32::from(*self.b1())) & (224_i32)) == 160).try_into()?;
        Ok(self.is_fix_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_float(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_float.get() {
            return Ok(self.is_float.borrow());
        }
        self.f_is_float.set(true);
        *self.is_float.borrow_mut() = ( ((*self.is_float_32()?) || (*self.is_float_64()?)) ).try_into()?;
        Ok(self.is_float.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-float> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_float_32(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_float_32.get() {
            return Ok(self.is_float_32.borrow());
        }
        self.f_is_float_32.set(true);
        *self.is_float_32.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(202)))).try_into()?;
        Ok(self.is_float_32.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-float> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_float_64(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_float_64.get() {
            return Ok(self.is_float_64.borrow());
        }
        self.f_is_float_64.set(true);
        *self.is_float_64.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(203)))).try_into()?;
        Ok(self.is_float_64.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_int(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_int.get() {
            return Ok(self.is_int.borrow());
        }
        self.f_is_int.set(true);
        *self.is_int.borrow_mut() = ( ((*self.is_pos_int7()?) || (*self.is_neg_int5()?)) ).try_into()?;
        Ok(self.is_int.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-map> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_map(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_map.get() {
            return Ok(self.is_map.borrow());
        }
        self.f_is_map.set(true);
        *self.is_map.borrow_mut() = ( ((*self.is_fix_map()?) || (*self.is_map_16()?) || (*self.is_map_32()?)) ).try_into()?;
        Ok(self.is_map.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-map> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_map_16(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_map_16.get() {
            return Ok(self.is_map_16.borrow());
        }
        self.f_is_map_16.set(true);
        *self.is_map_16.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(222)))).try_into()?;
        Ok(self.is_map_16.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-map> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_map_32(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_map_32.get() {
            return Ok(self.is_map_32.borrow());
        }
        self.f_is_map_32.set(true);
        *self.is_map_32.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(223)))).try_into()?;
        Ok(self.is_map_32.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_neg_int5(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_neg_int5.get() {
            return Ok(self.is_neg_int5.borrow());
        }
        self.f_is_neg_int5.set(true);
        *self.is_neg_int5.borrow_mut() = (((i32::from(*self.b1())) & (224_i32)) == 224).try_into()?;
        Ok(self.is_neg_int5.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-nil> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_nil(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_nil.get() {
            return Ok(self.is_nil.borrow());
        }
        self.f_is_nil.set(true);
        *self.is_nil.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(192)))).try_into()?;
        Ok(self.is_nil.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_pos_int7(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_pos_int7.get() {
            return Ok(self.is_pos_int7.borrow());
        }
        self.f_is_pos_int7.set(true);
        *self.is_pos_int7.borrow_mut() = (((i32::from(*self.b1())) & (128_i32)) == 0).try_into()?;
        Ok(self.is_pos_int7.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str.get() {
            return Ok(self.is_str.borrow());
        }
        self.f_is_str.set(true);
        *self.is_str.borrow_mut() = ( ((*self.is_fix_str()?) || (*self.is_str_8()?) || (*self.is_str_16()?) || (*self.is_str_32()?)) ).try_into()?;
        Ok(self.is_str.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-str> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_16(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_16.get() {
            return Ok(self.is_str_16.borrow());
        }
        self.f_is_str_16.set(true);
        *self.is_str_16.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(218)))).try_into()?;
        Ok(self.is_str_16.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-str> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_32(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_32.get() {
            return Ok(self.is_str_32.borrow());
        }
        self.f_is_str_32.set(true);
        *self.is_str_32.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(219)))).try_into()?;
        Ok(self.is_str_32.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-str> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_8(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_8.get() {
            return Ok(self.is_str_8.borrow());
        }
        self.f_is_str_8.set(true);
        *self.is_str_8.borrow_mut() = (((to_i128(*self.b1())) == (to_i128(217)))).try_into()?;
        Ok(self.is_str_8.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn neg_int5_value(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_neg_int5_value.get() {
            return Ok(self.neg_int5_value.borrow());
        }
        self.f_neg_int5_value.set(true);
        if *self.is_neg_int5()? {
            *self.neg_int5_value.borrow_mut() = ((0_i32).saturating_sub(to_i32(((i32::from(*self.b1())) & (31_i32))))).try_into()?;
        }
        Ok(self.neg_int5_value.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-array> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn num_array_elements(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_num_array_elements.get() {
            return Ok(self.num_array_elements.borrow());
        }
        self.f_num_array_elements.set(true);
        if *self.is_array()? {
            *self.num_array_elements.borrow_mut() = (if *self.is_fix_array()? { u32::try_from(((i32::from(*self.b1())) & (15_i32)))? } else { if *self.is_array_16()? { u32::from(*self.num_array_elements_16()) } else { *self.num_array_elements_32() } }).try_into()?;
        }
        Ok(self.num_array_elements.borrow())
    }

    /**
     * \sa <https://github.com/msgpack/msgpack/blob/master/spec.md#formats-map> Source
     */
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn num_map_elements(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_num_map_elements.get() {
            return Ok(self.num_map_elements.borrow());
        }
        self.f_num_map_elements.set(true);
        if *self.is_map()? {
            *self.num_map_elements.borrow_mut() = (if *self.is_fix_map()? { u32::try_from(((i32::from(*self.b1())) & (15_i32)))? } else { if *self.is_map_16()? { u32::from(*self.num_map_elements_16()) } else { *self.num_map_elements_32() } }).try_into()?;
        }
        Ok(self.num_map_elements.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn pos_int7_value(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_pos_int7_value.get() {
            return Ok(self.pos_int7_value.borrow());
        }
        self.f_pos_int7_value.set(true);
        if *self.is_pos_int7()? {
            *self.pos_int7_value.borrow_mut() = (*self.b1()).try_into()?;
        }
        Ok(self.pos_int7_value.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_len.get() {
            return Ok(self.str_len.borrow());
        }
        self.f_str_len.set(true);
        if *self.is_str()? {
            *self.str_len.borrow_mut() = (if *self.is_fix_str()? { u32::try_from(((i32::from(*self.b1())) & (31_i32)))? } else { if *self.is_str_8()? { u32::from(*self.str_len_8()) } else { if *self.is_str_16()? { u32::from(*self.str_len_16()) } else { *self.str_len_32() } } }).try_into()?;
        }
        Ok(self.str_len.borrow())
    }
}

/**
 * First byte is msgpack message is either a piece of data by
 * itself or determines types of further, more complex data
 * structures.
 */
impl Msgpack {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl Msgpack {
    pub fn int_extra(&self) -> u64 {
        // Reason for fallback: unwrap on parsed numeric switch option falls back to 0
        self.int_extra.borrow().as_ref().and_then(|v| u64::try_from(v).ok()).unwrap_or(0)
    }
    pub fn int_extra_enum(&self) -> Ref<'_, Option<Msgpack_IntExtra>> {
        self.int_extra.borrow()
    }
}
impl Msgpack {
    pub fn float_32_value(&self) -> Ref<'_, f32> {
        self.float_32_value.borrow()
    }
}
impl Msgpack {
    pub fn float_64_value(&self) -> Ref<'_, f64> {
        self.float_64_value.borrow()
    }
}
impl Msgpack {
    pub fn str_len_8(&self) -> Ref<'_, u8> {
        self.str_len_8.borrow()
    }
}
impl Msgpack {
    pub fn str_len_16(&self) -> Ref<'_, u16> {
        self.str_len_16.borrow()
    }
}
impl Msgpack {
    pub fn str_len_32(&self) -> Ref<'_, u32> {
        self.str_len_32.borrow()
    }
}
impl Msgpack {
    pub fn str_value(&self) -> Ref<'_, String> {
        self.str_value.borrow()
    }
}
impl Msgpack {
    pub fn num_array_elements_16(&self) -> Ref<'_, u16> {
        self.num_array_elements_16.borrow()
    }
}
impl Msgpack {
    pub fn num_array_elements_32(&self) -> Ref<'_, u32> {
        self.num_array_elements_32.borrow()
    }
}
impl Msgpack {
    pub fn array_elements(&self) -> Ref<'_, Vec<OptRc<Msgpack>>> {
        self.array_elements.borrow()
    }
}
impl Msgpack {
    pub fn num_map_elements_16(&self) -> Ref<'_, u16> {
        self.num_map_elements_16.borrow()
    }
}
impl Msgpack {
    pub fn num_map_elements_32(&self) -> Ref<'_, u32> {
        self.num_map_elements_32.borrow()
    }
}
impl Msgpack {
    pub fn map_elements(&self) -> Ref<'_, Vec<OptRc<Msgpack_MapTuple>>> {
        self.map_elements.borrow()
    }
}
impl Msgpack {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Msgpack {
    pub fn str_value_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_value_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Msgpack_MapTuple {
    pub(crate) _root: SharedType<Msgpack>,
    pub(crate) _parent: SharedType<Msgpack>,
    pub(crate) _self_shared: SharedType<Self>,
    key: RefCell<OptRc<Msgpack>>,
    value: RefCell<OptRc<Msgpack>>,
    _io: RefCell<BytesReader>,
}
impl KStruct for Msgpack_MapTuple {
    type Root = Msgpack;
    type Parent = Msgpack;

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
        let t = Self::read_into::<_, Msgpack>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.key.borrow_mut() = t;
        let t = Self::read_into::<_, Msgpack>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.value.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Msgpack_MapTuple {
}
impl Msgpack_MapTuple {
    pub fn key(&self) -> Ref<'_, OptRc<Msgpack>> {
        self.key.borrow()
    }
}
impl Msgpack_MapTuple {
    pub fn value(&self) -> Ref<'_, OptRc<Msgpack>> {
        self.value.borrow()
    }
}
impl Msgpack_MapTuple {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
