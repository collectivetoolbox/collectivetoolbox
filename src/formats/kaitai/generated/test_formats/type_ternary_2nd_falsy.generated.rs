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
pub struct TypeTernary2ndFalsy {
    pub(crate) _root: SharedType<TypeTernary2ndFalsy>,
    pub(crate) _parent: SharedType<TypeTernary2ndFalsy>,
    pub(crate) _self_shared: SharedType<Self>,
    int_truthy: RefCell<u8>,
    ut: RefCell<OptRc<TypeTernary2ndFalsy_Foo>>,
    int_array: RefCell<Vec<u8>>,
    int_array_empty: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    f_null_ut: Cell<bool>,
    null_ut: RefCell<OptRc<TypeTernary2ndFalsy_Foo>>,
    f_t: Cell<bool>,
    t: RefCell<bool>,
    f_v_false: Cell<bool>,
    v_false: RefCell<bool>,
    f_v_float_neg_zero: Cell<bool>,
    v_float_neg_zero: RefCell<f64>,
    f_v_float_zero: Cell<bool>,
    v_float_zero: RefCell<f64>,
    f_v_int_array_empty: Cell<bool>,
    v_int_array_empty: RefCell<Vec<u8>>,
    f_v_int_neg_zero: Cell<bool>,
    v_int_neg_zero: RefCell<i32>,
    f_v_int_zero: Cell<bool>,
    v_int_zero: RefCell<i32>,
    f_v_null_ut: Cell<bool>,
    v_null_ut: RefCell<OptRc<TypeTernary2ndFalsy_Foo>>,
    f_v_str_empty: Cell<bool>,
    v_str_empty: RefCell<String>,
    f_v_str_w_zero: Cell<bool>,
    v_str_w_zero: RefCell<String>,
}
impl KStruct for TypeTernary2ndFalsy {
    type Root = TypeTernary2ndFalsy;
    type Parent = TypeTernary2ndFalsy;

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
        *self_rc.int_truthy.borrow_mut() = _io.read_u1()?;
        let t = Self::read_into::<_, TypeTernary2ndFalsy_Foo>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.ut.borrow_mut() = t;
        *self_rc.int_array.borrow_mut() = Vec::new();
        let l_int_array = 2_usize;
        for _i in 0_usize..l_int_array {
            self_rc.int_array.borrow_mut().push(_io.read_u1()?);
        }
        *self_rc.int_array_empty.borrow_mut() = Vec::new();
        let l_int_array_empty = 0_usize;
        for _i in 0_usize..l_int_array_empty {
            self_rc.int_array_empty.borrow_mut().push(_io.read_u1()?);
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TypeTernary2ndFalsy {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn null_ut(
        &self
    ) -> KResult<Ref<'_, OptRc<TypeTernary2ndFalsy_Foo>>> {
        let _io = self._io.borrow();
        if self.f_null_ut.get() {
            return Ok(self.null_ut.borrow());
        }
        if false {
            *self.null_ut.borrow_mut() = self.ut().clone();
        }
        Ok(self.null_ut.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn t(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_t.get() {
            return Ok(self.t.borrow());
        }
        self.f_t.set(true);
        *self.t.borrow_mut() = (true).try_into()?;
        Ok(self.t.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_false(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_v_false.get() {
            return Ok(self.v_false.borrow());
        }
        self.f_v_false.set(true);
        *self.v_false.borrow_mut() = (if *self.t()? { false } else { true }).try_into()?;
        Ok(self.v_false.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_float_neg_zero(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_v_float_neg_zero.get() {
            return Ok(self.v_float_neg_zero.borrow());
        }
        self.f_v_float_neg_zero.set(true);
        *self.v_float_neg_zero.borrow_mut() = (if *self.t()? { -(0.0) } else { -(2.72) }).try_into()?;
        Ok(self.v_float_neg_zero.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_float_zero(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_v_float_zero.get() {
            return Ok(self.v_float_zero.borrow());
        }
        self.f_v_float_zero.set(true);
        *self.v_float_zero.borrow_mut() = (if *self.t()? { 0.0 } else { 3.14 }).try_into()?;
        Ok(self.v_float_zero.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_int_array_empty(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_v_int_array_empty.get() {
            return Ok(self.v_int_array_empty.borrow());
        }
        self.f_v_int_array_empty.set(true);
        *self.v_int_array_empty.borrow_mut() = if *self.t()? { self.int_array_empty().clone() } else { self.int_array().clone() }.to_vec();
        Ok(self.v_int_array_empty.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_int_neg_zero(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_v_int_neg_zero.get() {
            return Ok(self.v_int_neg_zero.borrow());
        }
        self.f_v_int_neg_zero.set(true);
        *self.v_int_neg_zero.borrow_mut() = (if *self.t()? { (0_i32).saturating_sub(to_i32(0)) } else { (0_i32).saturating_sub(to_i32(20)) }).try_into()?;
        Ok(self.v_int_neg_zero.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_int_zero(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_v_int_zero.get() {
            return Ok(self.v_int_zero.borrow());
        }
        self.f_v_int_zero.set(true);
        *self.v_int_zero.borrow_mut() = (if *self.t()? { 0_i32 } else { 10_i32 }).try_into()?;
        Ok(self.v_int_zero.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_null_ut(
        &self
    ) -> KResult<Ref<'_, OptRc<TypeTernary2ndFalsy_Foo>>> {
        let _io = self._io.borrow();
        if self.f_v_null_ut.get() {
            return Ok(self.v_null_ut.borrow());
        }
        *self.v_null_ut.borrow_mut() = if *self.t()? { self.null_ut()?.clone() } else { self.ut().clone() }.clone();
        Ok(self.v_null_ut.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_str_empty(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v_str_empty.get() {
            return Ok(self.v_str_empty.borrow());
        }
        self.f_v_str_empty.set(true);
        *self.v_str_empty.borrow_mut() = if *self.t()? { "".to_string() } else { "kaitai".to_string() }.to_string();
        Ok(self.v_str_empty.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_str_w_zero(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_v_str_w_zero.get() {
            return Ok(self.v_str_w_zero.borrow());
        }
        self.f_v_str_w_zero.set(true);
        *self.v_str_w_zero.borrow_mut() = if *self.t()? { "0".to_string() } else { "30".to_string() }.to_string();
        Ok(self.v_str_w_zero.borrow())
    }
}
impl TypeTernary2ndFalsy {
    pub fn int_truthy(&self) -> Ref<'_, u8> {
        self.int_truthy.borrow()
    }
}
impl TypeTernary2ndFalsy {
    pub fn ut(&self) -> Ref<'_, OptRc<TypeTernary2ndFalsy_Foo>> {
        self.ut.borrow()
    }
}
impl TypeTernary2ndFalsy {
    pub fn int_array(&self) -> Ref<'_, Vec<u8>> {
        self.int_array.borrow()
    }
}
impl TypeTernary2ndFalsy {
    pub fn int_array_empty(&self) -> Ref<'_, Vec<u8>> {
        self.int_array_empty.borrow()
    }
}
impl TypeTernary2ndFalsy {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct TypeTernary2ndFalsy_Foo {
    pub(crate) _root: SharedType<TypeTernary2ndFalsy>,
    pub(crate) _parent: SharedType<TypeTernary2ndFalsy>,
    pub(crate) _self_shared: SharedType<Self>,
    m: RefCell<u8>,
    _io: RefCell<BytesReader>,
}
impl KStruct for TypeTernary2ndFalsy_Foo {
    type Root = TypeTernary2ndFalsy;
    type Parent = TypeTernary2ndFalsy;

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
        *self_rc.m.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl TypeTernary2ndFalsy_Foo {
}
impl TypeTernary2ndFalsy_Foo {
    pub fn m(&self) -> Ref<'_, u8> {
        self.m.borrow()
    }
}
impl TypeTernary2ndFalsy_Foo {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
