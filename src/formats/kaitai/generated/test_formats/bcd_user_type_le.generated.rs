// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/*
== License information for parts derived from kaitai_struct_tests, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/
// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild

use kaitai::*;
use std::cell::{Cell, Ref, RefCell};

#[derive(Default, Debug, Clone)]
pub struct BcdUserTypeLe {
    pub(crate) _root: SharedType<BcdUserTypeLe>,
    pub(crate) _parent: SharedType<BcdUserTypeLe>,
    pub(crate) _self_shared: SharedType<Self>,
    ltr: RefCell<OptRc<BcdUserTypeLe_LtrObj>>,
    rtl: RefCell<OptRc<BcdUserTypeLe_RtlObj>>,
    leading_zero_ltr: RefCell<OptRc<BcdUserTypeLe_LeadingZeroLtrObj>>,
    _io: RefCell<BytesReader>,
    ltr_raw: RefCell<Vec<u8>>,
    rtl_raw: RefCell<Vec<u8>>,
    leading_zero_ltr_raw: RefCell<Vec<u8>>,
}
impl KStruct for BcdUserTypeLe {
    type Root = BcdUserTypeLe;
    type Parent = BcdUserTypeLe;

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
        let _raw_ltr = _io.read_bytes(4_usize)?;
        *self_rc.ltr_raw.borrow_mut() = _raw_ltr.clone();
        let _io_ltr = BytesReader::from(_raw_ltr);
        let t = Self::read_into::<BytesReader, BcdUserTypeLe_LtrObj>(&_io_ltr, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.ltr.borrow_mut() = t;
        let _raw_rtl = _io.read_bytes(4_usize)?;
        *self_rc.rtl_raw.borrow_mut() = _raw_rtl.clone();
        let _io_rtl = BytesReader::from(_raw_rtl);
        let t = Self::read_into::<BytesReader, BcdUserTypeLe_RtlObj>(&_io_rtl, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.rtl.borrow_mut() = t;
        let _raw_leading_zero_ltr = _io.read_bytes(4_usize)?;
        *self_rc.leading_zero_ltr_raw.borrow_mut() = _raw_leading_zero_ltr.clone();
        let _io_leading_zero_ltr = BytesReader::from(_raw_leading_zero_ltr);
        let t = Self::read_into::<BytesReader, BcdUserTypeLe_LeadingZeroLtrObj>(&_io_leading_zero_ltr, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.leading_zero_ltr.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BcdUserTypeLe {
}
impl BcdUserTypeLe {
    pub fn ltr(&self) -> Ref<'_, OptRc<BcdUserTypeLe_LtrObj>> {
        self.ltr.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn rtl(&self) -> Ref<'_, OptRc<BcdUserTypeLe_RtlObj>> {
        self.rtl.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn leading_zero_ltr(&self) -> Ref<'_, OptRc<BcdUserTypeLe_LeadingZeroLtrObj>> {
        self.leading_zero_ltr.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn ltr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.ltr_raw.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn rtl_raw(&self) -> Ref<'_, Vec<u8>> {
        self.rtl_raw.borrow()
    }
}
impl BcdUserTypeLe {
    pub fn leading_zero_ltr_raw(&self) -> Ref<'_, Vec<u8>> {
        self.leading_zero_ltr_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BcdUserTypeLe_LeadingZeroLtrObj {
    pub(crate) _root: SharedType<BcdUserTypeLe>,
    pub(crate) _parent: SharedType<BcdUserTypeLe>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    b2: RefCell<u8>,
    b3: RefCell<u8>,
    b4: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_as_int: Cell<bool>,
    as_int: RefCell<i32>,
    f_as_str: Cell<bool>,
    as_str: RefCell<String>,
    f_digit1: Cell<bool>,
    digit1: RefCell<i32>,
    f_digit2: Cell<bool>,
    digit2: RefCell<i32>,
    f_digit3: Cell<bool>,
    digit3: RefCell<i32>,
    f_digit4: Cell<bool>,
    digit4: RefCell<i32>,
    f_digit5: Cell<bool>,
    digit5: RefCell<i32>,
    f_digit6: Cell<bool>,
    digit6: RefCell<i32>,
    f_digit7: Cell<bool>,
    digit7: RefCell<i32>,
    f_digit8: Cell<bool>,
    digit8: RefCell<i32>,
}
impl KStruct for BcdUserTypeLe_LeadingZeroLtrObj {
    type Root = BcdUserTypeLe;
    type Parent = BcdUserTypeLe;

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
        *self_rc.b1.borrow_mut() = _io.read_u1()?;
        *self_rc.b2.borrow_mut() = _io.read_u1()?;
        *self_rc.b3.borrow_mut() = _io.read_u1()?;
        *self_rc.b4.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int.get() {
            return Ok(self.as_int.borrow());
        }
        self.f_as_int.set(true);
        *self.as_int.borrow_mut() = (((((((((*self.digit8()?).saturating_mul(1_i32)).saturating_add((*self.digit7()?).saturating_mul(10_i32))).saturating_add((*self.digit6()?).saturating_mul(100_i32))).saturating_add((*self.digit5()?).saturating_mul(1000_i32))).saturating_add((*self.digit4()?).saturating_mul(10000_i32))).saturating_add((*self.digit3()?).saturating_mul(100000_i32))).saturating_add((*self.digit2()?).saturating_mul(1000000_i32))).saturating_add((*self.digit1()?).saturating_mul(10000000_i32))).try_into()?;
        Ok(self.as_int.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_as_str.get() {
            return Ok(self.as_str.borrow());
        }
        self.f_as_str.set(true);
        *self.as_str.borrow_mut() = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", self.digit1()?.to_string(), self.digit2()?.to_string()), self.digit3()?.to_string()), self.digit4()?.to_string()), self.digit5()?.to_string()), self.digit6()?.to_string()), self.digit7()?.to_string()), self.digit8()?.to_string()).to_string();
        Ok(self.as_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit1.get() {
            return Ok(self.digit1.borrow());
        }
        self.f_digit1.set(true);
        *self.digit1.borrow_mut() = ((((i32::from(*self.b4())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit1.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit2.get() {
            return Ok(self.digit2.borrow());
        }
        self.f_digit2.set(true);
        *self.digit2.borrow_mut() = (((i32::from(*self.b4())) & (15_i32))).try_into()?;
        Ok(self.digit2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit3(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit3.get() {
            return Ok(self.digit3.borrow());
        }
        self.f_digit3.set(true);
        *self.digit3.borrow_mut() = ((((i32::from(*self.b3())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit4(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit4.get() {
            return Ok(self.digit4.borrow());
        }
        self.f_digit4.set(true);
        *self.digit4.borrow_mut() = (((i32::from(*self.b3())) & (15_i32))).try_into()?;
        Ok(self.digit4.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit5(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit5.get() {
            return Ok(self.digit5.borrow());
        }
        self.f_digit5.set(true);
        *self.digit5.borrow_mut() = ((((i32::from(*self.b2())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit6(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit6.get() {
            return Ok(self.digit6.borrow());
        }
        self.f_digit6.set(true);
        *self.digit6.borrow_mut() = (((i32::from(*self.b2())) & (15_i32))).try_into()?;
        Ok(self.digit6.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit7(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit7.get() {
            return Ok(self.digit7.borrow());
        }
        self.f_digit7.set(true);
        *self.digit7.borrow_mut() = ((((i32::from(*self.b1())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit7.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit8(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit8.get() {
            return Ok(self.digit8.borrow());
        }
        self.f_digit8.set(true);
        *self.digit8.borrow_mut() = (((i32::from(*self.b1())) & (15_i32))).try_into()?;
        Ok(self.digit8.borrow())
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    pub fn b3(&self) -> Ref<'_, u8> {
        self.b3.borrow()
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    pub fn b4(&self) -> Ref<'_, u8> {
        self.b4.borrow()
    }
}
impl BcdUserTypeLe_LeadingZeroLtrObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BcdUserTypeLe_LtrObj {
    pub(crate) _root: SharedType<BcdUserTypeLe>,
    pub(crate) _parent: SharedType<BcdUserTypeLe>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    b2: RefCell<u8>,
    b3: RefCell<u8>,
    b4: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_as_int: Cell<bool>,
    as_int: RefCell<i32>,
    f_as_str: Cell<bool>,
    as_str: RefCell<String>,
    f_digit1: Cell<bool>,
    digit1: RefCell<i32>,
    f_digit2: Cell<bool>,
    digit2: RefCell<i32>,
    f_digit3: Cell<bool>,
    digit3: RefCell<i32>,
    f_digit4: Cell<bool>,
    digit4: RefCell<i32>,
    f_digit5: Cell<bool>,
    digit5: RefCell<i32>,
    f_digit6: Cell<bool>,
    digit6: RefCell<i32>,
    f_digit7: Cell<bool>,
    digit7: RefCell<i32>,
    f_digit8: Cell<bool>,
    digit8: RefCell<i32>,
}
impl KStruct for BcdUserTypeLe_LtrObj {
    type Root = BcdUserTypeLe;
    type Parent = BcdUserTypeLe;

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
        *self_rc.b1.borrow_mut() = _io.read_u1()?;
        *self_rc.b2.borrow_mut() = _io.read_u1()?;
        *self_rc.b3.borrow_mut() = _io.read_u1()?;
        *self_rc.b4.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BcdUserTypeLe_LtrObj {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int.get() {
            return Ok(self.as_int.borrow());
        }
        self.f_as_int.set(true);
        *self.as_int.borrow_mut() = (((((((((*self.digit8()?).saturating_mul(1_i32)).saturating_add((*self.digit7()?).saturating_mul(10_i32))).saturating_add((*self.digit6()?).saturating_mul(100_i32))).saturating_add((*self.digit5()?).saturating_mul(1000_i32))).saturating_add((*self.digit4()?).saturating_mul(10000_i32))).saturating_add((*self.digit3()?).saturating_mul(100000_i32))).saturating_add((*self.digit2()?).saturating_mul(1000000_i32))).saturating_add((*self.digit1()?).saturating_mul(10000000_i32))).try_into()?;
        Ok(self.as_int.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_as_str.get() {
            return Ok(self.as_str.borrow());
        }
        self.f_as_str.set(true);
        *self.as_str.borrow_mut() = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", self.digit1()?.to_string(), self.digit2()?.to_string()), self.digit3()?.to_string()), self.digit4()?.to_string()), self.digit5()?.to_string()), self.digit6()?.to_string()), self.digit7()?.to_string()), self.digit8()?.to_string()).to_string();
        Ok(self.as_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit1.get() {
            return Ok(self.digit1.borrow());
        }
        self.f_digit1.set(true);
        *self.digit1.borrow_mut() = ((((i32::from(*self.b4())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit1.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit2.get() {
            return Ok(self.digit2.borrow());
        }
        self.f_digit2.set(true);
        *self.digit2.borrow_mut() = (((i32::from(*self.b4())) & (15_i32))).try_into()?;
        Ok(self.digit2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit3(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit3.get() {
            return Ok(self.digit3.borrow());
        }
        self.f_digit3.set(true);
        *self.digit3.borrow_mut() = ((((i32::from(*self.b3())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit4(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit4.get() {
            return Ok(self.digit4.borrow());
        }
        self.f_digit4.set(true);
        *self.digit4.borrow_mut() = (((i32::from(*self.b3())) & (15_i32))).try_into()?;
        Ok(self.digit4.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit5(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit5.get() {
            return Ok(self.digit5.borrow());
        }
        self.f_digit5.set(true);
        *self.digit5.borrow_mut() = ((((i32::from(*self.b2())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit6(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit6.get() {
            return Ok(self.digit6.borrow());
        }
        self.f_digit6.set(true);
        *self.digit6.borrow_mut() = (((i32::from(*self.b2())) & (15_i32))).try_into()?;
        Ok(self.digit6.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit7(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit7.get() {
            return Ok(self.digit7.borrow());
        }
        self.f_digit7.set(true);
        *self.digit7.borrow_mut() = ((((i32::from(*self.b1())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit7.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit8(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit8.get() {
            return Ok(self.digit8.borrow());
        }
        self.f_digit8.set(true);
        *self.digit8.borrow_mut() = (((i32::from(*self.b1())) & (15_i32))).try_into()?;
        Ok(self.digit8.borrow())
    }
}
impl BcdUserTypeLe_LtrObj {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl BcdUserTypeLe_LtrObj {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl BcdUserTypeLe_LtrObj {
    pub fn b3(&self) -> Ref<'_, u8> {
        self.b3.borrow()
    }
}
impl BcdUserTypeLe_LtrObj {
    pub fn b4(&self) -> Ref<'_, u8> {
        self.b4.borrow()
    }
}
impl BcdUserTypeLe_LtrObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BcdUserTypeLe_RtlObj {
    pub(crate) _root: SharedType<BcdUserTypeLe>,
    pub(crate) _parent: SharedType<BcdUserTypeLe>,
    pub(crate) _self_shared: SharedType<Self>,
    b1: RefCell<u8>,
    b2: RefCell<u8>,
    b3: RefCell<u8>,
    b4: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_as_int: Cell<bool>,
    as_int: RefCell<i32>,
    f_as_str: Cell<bool>,
    as_str: RefCell<String>,
    f_digit1: Cell<bool>,
    digit1: RefCell<i32>,
    f_digit2: Cell<bool>,
    digit2: RefCell<i32>,
    f_digit3: Cell<bool>,
    digit3: RefCell<i32>,
    f_digit4: Cell<bool>,
    digit4: RefCell<i32>,
    f_digit5: Cell<bool>,
    digit5: RefCell<i32>,
    f_digit6: Cell<bool>,
    digit6: RefCell<i32>,
    f_digit7: Cell<bool>,
    digit7: RefCell<i32>,
    f_digit8: Cell<bool>,
    digit8: RefCell<i32>,
}
impl KStruct for BcdUserTypeLe_RtlObj {
    type Root = BcdUserTypeLe;
    type Parent = BcdUserTypeLe;

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
        *self_rc.b1.borrow_mut() = _io.read_u1()?;
        *self_rc.b2.borrow_mut() = _io.read_u1()?;
        *self_rc.b3.borrow_mut() = _io.read_u1()?;
        *self_rc.b4.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl BcdUserTypeLe_RtlObj {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_as_int.get() {
            return Ok(self.as_int.borrow());
        }
        self.f_as_int.set(true);
        *self.as_int.borrow_mut() = (((((((((*self.digit1()?).saturating_mul(1_i32)).saturating_add((*self.digit2()?).saturating_mul(10_i32))).saturating_add((*self.digit3()?).saturating_mul(100_i32))).saturating_add((*self.digit4()?).saturating_mul(1000_i32))).saturating_add((*self.digit5()?).saturating_mul(10000_i32))).saturating_add((*self.digit6()?).saturating_mul(100000_i32))).saturating_add((*self.digit7()?).saturating_mul(1000000_i32))).saturating_add((*self.digit8()?).saturating_mul(10000000_i32))).try_into()?;
        Ok(self.as_int.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn as_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_as_str.get() {
            return Ok(self.as_str.borrow());
        }
        self.f_as_str.set(true);
        *self.as_str.borrow_mut() = format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", format!("{}{}", self.digit8()?.to_string(), self.digit7()?.to_string()), self.digit6()?.to_string()), self.digit5()?.to_string()), self.digit4()?.to_string()), self.digit3()?.to_string()), self.digit2()?.to_string()), self.digit1()?.to_string()).to_string();
        Ok(self.as_str.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit1(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit1.get() {
            return Ok(self.digit1.borrow());
        }
        self.f_digit1.set(true);
        *self.digit1.borrow_mut() = ((((i32::from(*self.b4())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit1.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit2.get() {
            return Ok(self.digit2.borrow());
        }
        self.f_digit2.set(true);
        *self.digit2.borrow_mut() = (((i32::from(*self.b4())) & (15_i32))).try_into()?;
        Ok(self.digit2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit3(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit3.get() {
            return Ok(self.digit3.borrow());
        }
        self.f_digit3.set(true);
        *self.digit3.borrow_mut() = ((((i32::from(*self.b3())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit4(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit4.get() {
            return Ok(self.digit4.borrow());
        }
        self.f_digit4.set(true);
        *self.digit4.borrow_mut() = (((i32::from(*self.b3())) & (15_i32))).try_into()?;
        Ok(self.digit4.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit5(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit5.get() {
            return Ok(self.digit5.borrow());
        }
        self.f_digit5.set(true);
        *self.digit5.borrow_mut() = ((((i32::from(*self.b2())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit6(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit6.get() {
            return Ok(self.digit6.borrow());
        }
        self.f_digit6.set(true);
        *self.digit6.borrow_mut() = (((i32::from(*self.b2())) & (15_i32))).try_into()?;
        Ok(self.digit6.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit7(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit7.get() {
            return Ok(self.digit7.borrow());
        }
        self.f_digit7.set(true);
        *self.digit7.borrow_mut() = ((((i32::from(*self.b1())) & (240_i32))).wrapping_shr(4_u32)).try_into()?;
        Ok(self.digit7.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn digit8(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_digit8.get() {
            return Ok(self.digit8.borrow());
        }
        self.f_digit8.set(true);
        *self.digit8.borrow_mut() = (((i32::from(*self.b1())) & (15_i32))).try_into()?;
        Ok(self.digit8.borrow())
    }
}
impl BcdUserTypeLe_RtlObj {
    pub fn b1(&self) -> Ref<'_, u8> {
        self.b1.borrow()
    }
}
impl BcdUserTypeLe_RtlObj {
    pub fn b2(&self) -> Ref<'_, u8> {
        self.b2.borrow()
    }
}
impl BcdUserTypeLe_RtlObj {
    pub fn b3(&self) -> Ref<'_, u8> {
        self.b3.borrow()
    }
}
impl BcdUserTypeLe_RtlObj {
    pub fn b4(&self) -> Ref<'_, u8> {
        self.b4.borrow()
    }
}
impl BcdUserTypeLe_RtlObj {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
