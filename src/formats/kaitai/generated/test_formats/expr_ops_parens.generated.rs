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
pub struct ExprOpsParens {
    pub(crate) _root: SharedType<ExprOpsParens>,
    pub(crate) _parent: SharedType<ExprOpsParens>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_bool_and: Cell<bool>,
    bool_and: RefCell<i32>,
    f_bool_eq: Cell<bool>,
    bool_eq: RefCell<i32>,
    f_bool_or: Cell<bool>,
    bool_or: RefCell<i32>,
    f_f_2pi: Cell<bool>,
    f_2pi: RefCell<f64>,
    f_f_e: Cell<bool>,
    f_e: RefCell<f64>,
    f_f_sum_to_int: Cell<bool>,
    f_sum_to_int: RefCell<i32>,
    f_i_42: Cell<bool>,
    i_42: RefCell<i32>,
    f_i_m13: Cell<bool>,
    i_m13: RefCell<i32>,
    f_i_sum_to_str: Cell<bool>,
    i_sum_to_str: RefCell<String>,
    f_str_0_to_4: Cell<bool>,
    str_0_to_4: RefCell<String>,
    f_str_5_to_9: Cell<bool>,
    str_5_to_9: RefCell<String>,
    f_str_concat_len: Cell<bool>,
    str_concat_len: RefCell<i32>,
    f_str_concat_rev: Cell<bool>,
    str_concat_rev: RefCell<String>,
    f_str_concat_substr_2_to_7: Cell<bool>,
    str_concat_substr_2_to_7: RefCell<String>,
    f_str_concat_to_i: Cell<bool>,
    str_concat_to_i: RefCell<i32>,
}
impl KStruct for ExprOpsParens {
    type Root = ExprOpsParens;
    type Parent = ExprOpsParens;

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
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprOpsParens {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn bool_and(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_bool_and.get() {
            return Ok(self.bool_and.borrow());
        }
        self.f_bool_and.set(true);
        *self.bool_and.borrow_mut() = ((if ({ let _ = true; false }) { 1_i32 } else { 0_i32 })).try_into()?;
        Ok(self.bool_and.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn bool_eq(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_bool_eq.get() {
            return Ok(self.bool_eq.borrow());
        }
        self.f_bool_eq.set(true);
        *self.bool_eq.borrow_mut() = ((if false == true { 1_i32 } else { 0_i32 })).try_into()?;
        Ok(self.bool_eq.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn bool_or(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_bool_or.get() {
            return Ok(self.bool_or.borrow());
        }
        self.f_bool_or.set(true);
        *self.bool_or.borrow_mut() = ((if  ((!(false)) || (false))  { 1_i32 } else { 0_i32 })).try_into()?;
        Ok(self.bool_or.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn f_2pi(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_f_2pi.get() {
            return Ok(self.f_2pi.borrow());
        }
        self.f_f_2pi.set(true);
        *self.f_2pi.borrow_mut() = (6.28).try_into()?;
        Ok(self.f_2pi.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn f_e(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_f_e.get() {
            return Ok(self.f_e.borrow());
        }
        self.f_f_e.set(true);
        *self.f_e.borrow_mut() = (2.72).try_into()?;
        Ok(self.f_e.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn f_sum_to_int(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_f_sum_to_int.get() {
            return Ok(self.f_sum_to_int.borrow());
        }
        self.f_f_sum_to_int.set(true);
        *self.f_sum_to_int.borrow_mut() = (float_to_int(*((*self.f_2pi()?) + (*self.f_e()?)))?).try_into()?;
        Ok(self.f_sum_to_int.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn i_42(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_i_42.get() {
            return Ok(self.i_42.borrow());
        }
        self.f_i_42.set(true);
        *self.i_42.borrow_mut() = (42).try_into()?;
        Ok(self.i_42.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn i_m13(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_i_m13.get() {
            return Ok(self.i_m13.borrow());
        }
        self.f_i_m13.set(true);
        *self.i_m13.borrow_mut() = (-13).try_into()?;
        Ok(self.i_m13.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn i_sum_to_str(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_i_sum_to_str.get() {
            return Ok(self.i_sum_to_str.borrow());
        }
        self.f_i_sum_to_str.set(true);
        *self.i_sum_to_str.borrow_mut() = (*self.i_42()?).saturating_add(*self.i_m13()?).to_string().to_string();
        Ok(self.i_sum_to_str.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_0_to_4(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_0_to_4.get() {
            return Ok(self.str_0_to_4.borrow());
        }
        self.f_str_0_to_4.set(true);
        *self.str_0_to_4.borrow_mut() = "01234".to_string();
        Ok(self.str_0_to_4.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_5_to_9(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_5_to_9.get() {
            return Ok(self.str_5_to_9.borrow());
        }
        self.f_str_5_to_9.set(true);
        *self.str_5_to_9.borrow_mut() = "56789".to_string();
        Ok(self.str_5_to_9.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_concat_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_concat_len.get() {
            return Ok(self.str_concat_len.borrow());
        }
        self.f_str_concat_len.set(true);
        *self.str_concat_len.borrow_mut() = (format!("{}{}", *self.str_0_to_4()?, *self.str_5_to_9()?).len()).try_into()?;
        Ok(self.str_concat_len.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_concat_rev(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_concat_rev.get() {
            return Ok(self.str_concat_rev.borrow());
        }
        self.f_str_concat_rev.set(true);
        *self.str_concat_rev.borrow_mut() = reverse_string(&format!("{}{}", *self.str_0_to_4()?, *self.str_5_to_9()?))?.to_string();
        Ok(self.str_concat_rev.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_concat_substr_2_to_7(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_concat_substr_2_to_7.get() {
            return Ok(self.str_concat_substr_2_to_7.borrow());
        }
        self.f_str_concat_substr_2_to_7.set(true);
        *self.str_concat_substr_2_to_7.borrow_mut() = substring(&format!("{}{}", *self.str_0_to_4()?, *self.str_5_to_9()?), 2, 7).to_string();
        Ok(self.str_concat_substr_2_to_7.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn str_concat_to_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_concat_to_i.get() {
            return Ok(self.str_concat_to_i.borrow());
        }
        self.f_str_concat_to_i.set(true);
        *self.str_concat_to_i.borrow_mut() = (format!("{}{}", *self.str_0_to_4()?, *self.str_5_to_9()?).parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.str_concat_to_i.borrow())
    }
}
impl ExprOpsParens {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
