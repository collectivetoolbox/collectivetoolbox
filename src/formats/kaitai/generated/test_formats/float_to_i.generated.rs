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
pub struct FloatToI {
    pub(crate) _root: SharedType<FloatToI>,
    pub(crate) _parent: SharedType<FloatToI>,
    pub(crate) _self_shared: SharedType<Self>,
    single_value: RefCell<f32>,
    double_value: RefCell<f64>,
    single_value_if: RefCell<f32>,
    double_value_if: RefCell<f64>,
    _io: RefCell<BytesReader>,
    f_calc_float1: Cell<bool>,
    calc_float1: RefCell<f64>,
    f_calc_float2: Cell<bool>,
    calc_float2: RefCell<f64>,
    f_calc_float3: Cell<bool>,
    calc_float3: RefCell<f64>,
    f_calc_float4: Cell<bool>,
    calc_float4: RefCell<f64>,
    f_calc_if: Cell<bool>,
    calc_if: RefCell<f64>,
    f_calc_if_i: Cell<bool>,
    calc_if_i: RefCell<i32>,
    f_double_i: Cell<bool>,
    double_i: RefCell<i32>,
    f_double_if_i: Cell<bool>,
    double_if_i: RefCell<i32>,
    f_float1_i: Cell<bool>,
    float1_i: RefCell<i32>,
    f_float2_i: Cell<bool>,
    float2_i: RefCell<i32>,
    f_float3_i: Cell<bool>,
    float3_i: RefCell<i32>,
    f_float4_i: Cell<bool>,
    float4_i: RefCell<i32>,
    f_single_i: Cell<bool>,
    single_i: RefCell<i32>,
    f_single_if_i: Cell<bool>,
    single_if_i: RefCell<i32>,
}
impl KStruct for FloatToI {
    type Root = FloatToI;
    type Parent = FloatToI;

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
        *self_rc.single_value.borrow_mut() = _io.read_f4le()?;
        *self_rc.double_value.borrow_mut() = _io.read_f8le()?;
        if true {
            *self_rc.single_value_if.borrow_mut() = _io.read_f4be()?;
        }
        if true {
            *self_rc.double_value_if.borrow_mut() = _io.read_f8be()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl FloatToI {
    pub fn calc_float1(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_calc_float1.get() {
            return Ok(self.calc_float1.borrow());
        }
        self.f_calc_float1.set(true);
        *self.calc_float1.borrow_mut() = (1.234).try_into()?;
        Ok(self.calc_float1.borrow())
    }
    pub fn calc_float2(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_calc_float2.get() {
            return Ok(self.calc_float2.borrow());
        }
        self.f_calc_float2.set(true);
        *self.calc_float2.borrow_mut() = (1.5).try_into()?;
        Ok(self.calc_float2.borrow())
    }
    pub fn calc_float3(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_calc_float3.get() {
            return Ok(self.calc_float3.borrow());
        }
        self.f_calc_float3.set(true);
        *self.calc_float3.borrow_mut() = (1.9).try_into()?;
        Ok(self.calc_float3.borrow())
    }
    pub fn calc_float4(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_calc_float4.get() {
            return Ok(self.calc_float4.borrow());
        }
        self.f_calc_float4.set(true);
        *self.calc_float4.borrow_mut() = (-2.7).try_into()?;
        Ok(self.calc_float4.borrow())
    }
    pub fn calc_if(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_calc_if.get() {
            return Ok(self.calc_if.borrow());
        }
        self.f_calc_if.set(true);
        *self.calc_if.borrow_mut() = (13.9).try_into()?;
        Ok(self.calc_if.borrow())
    }
    pub fn calc_if_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_calc_if_i.get() {
            return Ok(self.calc_if_i.borrow());
        }
        self.f_calc_if_i.set(true);
        *self.calc_if_i.borrow_mut() = (float_to_int(*self.calc_if()?)?).try_into()?;
        Ok(self.calc_if_i.borrow())
    }
    pub fn double_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_double_i.get() {
            return Ok(self.double_i.borrow());
        }
        self.f_double_i.set(true);
        *self.double_i.borrow_mut() = (float_to_int(*self.double_value())?).try_into()?;
        Ok(self.double_i.borrow())
    }
    pub fn double_if_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_double_if_i.get() {
            return Ok(self.double_if_i.borrow());
        }
        self.f_double_if_i.set(true);
        *self.double_if_i.borrow_mut() = (float_to_int(*self.double_value_if())?).try_into()?;
        Ok(self.double_if_i.borrow())
    }
    pub fn float1_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_float1_i.get() {
            return Ok(self.float1_i.borrow());
        }
        self.f_float1_i.set(true);
        *self.float1_i.borrow_mut() = (float_to_int(*self.calc_float1()?)?).try_into()?;
        Ok(self.float1_i.borrow())
    }
    pub fn float2_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_float2_i.get() {
            return Ok(self.float2_i.borrow());
        }
        self.f_float2_i.set(true);
        *self.float2_i.borrow_mut() = (float_to_int(*self.calc_float2()?)?).try_into()?;
        Ok(self.float2_i.borrow())
    }
    pub fn float3_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_float3_i.get() {
            return Ok(self.float3_i.borrow());
        }
        self.f_float3_i.set(true);
        *self.float3_i.borrow_mut() = (float_to_int(*self.calc_float3()?)?).try_into()?;
        Ok(self.float3_i.borrow())
    }
    pub fn float4_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_float4_i.get() {
            return Ok(self.float4_i.borrow());
        }
        self.f_float4_i.set(true);
        *self.float4_i.borrow_mut() = (float_to_int(*self.calc_float4()?)?).try_into()?;
        Ok(self.float4_i.borrow())
    }
    pub fn single_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_single_i.get() {
            return Ok(self.single_i.borrow());
        }
        self.f_single_i.set(true);
        *self.single_i.borrow_mut() = (float_to_int(*self.single_value())?).try_into()?;
        Ok(self.single_i.borrow())
    }
    pub fn single_if_i(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_single_if_i.get() {
            return Ok(self.single_if_i.borrow());
        }
        self.f_single_if_i.set(true);
        *self.single_if_i.borrow_mut() = (float_to_int(*self.single_value_if())?).try_into()?;
        Ok(self.single_if_i.borrow())
    }
}
impl FloatToI {
    pub fn single_value(&self) -> Ref<'_, f32> {
        self.single_value.borrow()
    }
}
impl FloatToI {
    pub fn double_value(&self) -> Ref<'_, f64> {
        self.double_value.borrow()
    }
}
impl FloatToI {
    pub fn single_value_if(&self) -> Ref<'_, f32> {
        self.single_value_if.borrow()
    }
}
impl FloatToI {
    pub fn double_value_if(&self) -> Ref<'_, f64> {
        self.double_value_if.borrow()
    }
}
impl FloatToI {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
