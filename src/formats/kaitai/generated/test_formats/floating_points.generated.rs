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
pub struct FloatingPoints {
    pub(crate) _root: SharedType<FloatingPoints>,
    pub(crate) _parent: SharedType<FloatingPoints>,
    pub(crate) _self_shared: SharedType<Self>,
    single_value: RefCell<f32>,
    double_value: RefCell<f64>,
    single_value_be: RefCell<f32>,
    double_value_be: RefCell<f64>,
    approximate_value: RefCell<f32>,
    _io: RefCell<BytesReader>,
    f_double_value_plus_float: Cell<bool>,
    double_value_plus_float: RefCell<f64>,
    f_single_value_plus_float: Cell<bool>,
    single_value_plus_float: RefCell<f32>,
    f_single_value_plus_int: Cell<bool>,
    single_value_plus_int: RefCell<f32>,
}
impl KStruct for FloatingPoints {
    type Root = FloatingPoints;
    type Parent = FloatingPoints;

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
        *self_rc.single_value.borrow_mut() = _io.read_f4le()?;
        *self_rc.double_value.borrow_mut() = _io.read_f8le()?;
        *self_rc.single_value_be.borrow_mut() = _io.read_f4be()?;
        *self_rc.double_value_be.borrow_mut() = _io.read_f8be()?;
        *self_rc.approximate_value.borrow_mut() = _io.read_f4le()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl FloatingPoints {
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn double_value_plus_float(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_double_value_plus_float.get() {
            return Ok(self.double_value_plus_float.borrow());
        }
        self.f_double_value_plus_float.set(true);
        *self.double_value_plus_float.borrow_mut() = (((*self.double_value()) + (0.05))).try_into()?;
        Ok(self.double_value_plus_float.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn single_value_plus_float(
        &self
    ) -> KResult<Ref<'_, f32>> {
        let _io = self._io.borrow();
        if self.f_single_value_plus_float.get() {
            return Ok(self.single_value_plus_float.borrow());
        }
        self.f_single_value_plus_float.set(true);
        *self.single_value_plus_float.borrow_mut() = (((*self.single_value()) + (to_f32(0.5)))).try_into()?;
        Ok(self.single_value_plus_float.borrow())
    }
    #[allow(clippy::approx_constant, reason = "Kaitai format specification float literal")]
    pub fn single_value_plus_int(
        &self
    ) -> KResult<Ref<'_, f32>> {
        let _io = self._io.borrow();
        if self.f_single_value_plus_int.get() {
            return Ok(self.single_value_plus_int.borrow());
        }
        self.f_single_value_plus_int.set(true);
        *self.single_value_plus_int.borrow_mut() = (((*self.single_value()) + (to_f32(1)))).try_into()?;
        Ok(self.single_value_plus_int.borrow())
    }
}
impl FloatingPoints {
    pub fn single_value(&self) -> Ref<'_, f32> {
        self.single_value.borrow()
    }
}
impl FloatingPoints {
    pub fn double_value(&self) -> Ref<'_, f64> {
        self.double_value.borrow()
    }
}
impl FloatingPoints {
    pub fn single_value_be(&self) -> Ref<'_, f32> {
        self.single_value_be.borrow()
    }
}
impl FloatingPoints {
    pub fn double_value_be(&self) -> Ref<'_, f64> {
        self.double_value_be.borrow()
    }
}
impl FloatingPoints {
    pub fn approximate_value(&self) -> Ref<'_, f32> {
        self.approximate_value.borrow()
    }
}
impl FloatingPoints {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
