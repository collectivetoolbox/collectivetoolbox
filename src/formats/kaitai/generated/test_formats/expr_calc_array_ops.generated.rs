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
pub struct ExprCalcArrayOps {
    pub(crate) _root: SharedType<ExprCalcArrayOps>,
    pub(crate) _parent: SharedType<ExprCalcArrayOps>,
    pub(crate) _self_shared: SharedType<Self>,
    _io: RefCell<BytesReader>,
    f_double_array: Cell<bool>,
    double_array: RefCell<Vec<f64>>,
    f_double_array_first: Cell<bool>,
    double_array_first: RefCell<f64>,
    f_double_array_last: Cell<bool>,
    double_array_last: RefCell<f64>,
    f_double_array_max: Cell<bool>,
    double_array_max: RefCell<i32>,
    f_double_array_mid: Cell<bool>,
    double_array_mid: RefCell<f64>,
    f_double_array_min: Cell<bool>,
    double_array_min: RefCell<i32>,
    f_double_array_size: Cell<bool>,
    double_array_size: RefCell<i32>,
    f_int_array: Cell<bool>,
    int_array: RefCell<Vec<i32>>,
    f_int_array_first: Cell<bool>,
    int_array_first: RefCell<i32>,
    f_int_array_last: Cell<bool>,
    int_array_last: RefCell<i32>,
    f_int_array_max: Cell<bool>,
    int_array_max: RefCell<i32>,
    f_int_array_mid: Cell<bool>,
    int_array_mid: RefCell<i32>,
    f_int_array_min: Cell<bool>,
    int_array_min: RefCell<i32>,
    f_int_array_size: Cell<bool>,
    int_array_size: RefCell<i32>,
    f_str_array: Cell<bool>,
    str_array: RefCell<Vec<String>>,
    f_str_array_first: Cell<bool>,
    str_array_first: RefCell<String>,
    f_str_array_last: Cell<bool>,
    str_array_last: RefCell<String>,
    f_str_array_max: Cell<bool>,
    str_array_max: RefCell<i32>,
    f_str_array_mid: Cell<bool>,
    str_array_mid: RefCell<String>,
    f_str_array_min: Cell<bool>,
    str_array_min: RefCell<i32>,
    f_str_array_size: Cell<bool>,
    str_array_size: RefCell<i32>,
}
impl KStruct for ExprCalcArrayOps {
    type Root = ExprCalcArrayOps;
    type Parent = ExprCalcArrayOps;

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
        Ok(())
    }
}
impl ExprCalcArrayOps {
    pub fn double_array(
        &self
    ) -> KResult<Ref<'_, Vec<f64>>> {
        let _io = self._io.borrow();
        if self.f_double_array.get() {
            return Ok(self.double_array.borrow());
        }
        self.f_double_array.set(true);
        *self.double_array.borrow_mut() = vec![10_f64, 25_f64, 50_f64, 100_f64, 3.14159_f64];
        Ok(self.double_array.borrow())
    }
    pub fn double_array_first(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_double_array_first.get() {
            return Ok(self.double_array_first.borrow());
        }
        self.f_double_array_first.set(true);
        *self.double_array_first.borrow_mut() = (*self.double_array()?.first().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.double_array_first.borrow())
    }
    pub fn double_array_last(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_double_array_last.get() {
            return Ok(self.double_array_last.borrow());
        }
        self.f_double_array_last.set(true);
        *self.double_array_last.borrow_mut() = (*self.double_array()?.last().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.double_array_last.borrow())
    }
    pub fn double_array_max(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_double_array_max.get() {
            return Ok(self.double_array_max.borrow());
        }
        self.f_double_array_max.set(true);
        *self.double_array_max.borrow_mut() = (*self.double_array()?.iter().reduce(|a, b| if (a.max(*b)) == *b { b } else { a }).ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.double_array_max.borrow())
    }
    pub fn double_array_mid(
        &self
    ) -> KResult<Ref<'_, f64>> {
        let _io = self._io.borrow();
        if self.f_double_array_mid.get() {
            return Ok(self.double_array_mid.borrow());
        }
        self.f_double_array_mid.set(true);
        *self.double_array_mid.borrow_mut() = (*(self.double_array()?.get(1_usize).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.double_array_mid.borrow())
    }
    pub fn double_array_min(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_double_array_min.get() {
            return Ok(self.double_array_min.borrow());
        }
        self.f_double_array_min.set(true);
        *self.double_array_min.borrow_mut() = (*self.double_array()?.iter().reduce(|a, b| if (a.min(*b)) == *b { b } else { a }).ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.double_array_min.borrow())
    }
    pub fn double_array_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_double_array_size.get() {
            return Ok(self.double_array_size.borrow());
        }
        self.f_double_array_size.set(true);
        *self.double_array_size.borrow_mut() = (self.double_array()?.len()).try_into()?;
        Ok(self.double_array_size.borrow())
    }
    pub fn int_array(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_int_array.get() {
            return Ok(self.int_array.borrow());
        }
        self.f_int_array.set(true);
        *self.int_array.borrow_mut() = vec![10_i32, 25_i32, 50_i32, 100_i32, 200_i32, 500_i32, 1000_i32];
        Ok(self.int_array.borrow())
    }
    pub fn int_array_first(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_first.get() {
            return Ok(self.int_array_first.borrow());
        }
        self.f_int_array_first.set(true);
        *self.int_array_first.borrow_mut() = (*self.int_array()?.first().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.int_array_first.borrow())
    }
    pub fn int_array_last(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_last.get() {
            return Ok(self.int_array_last.borrow());
        }
        self.f_int_array_last.set(true);
        *self.int_array_last.borrow_mut() = (*self.int_array()?.last().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.int_array_last.borrow())
    }
    pub fn int_array_max(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_max.get() {
            return Ok(self.int_array_max.borrow());
        }
        self.f_int_array_max.set(true);
        *self.int_array_max.borrow_mut() = (*self.int_array()?.iter().max().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.int_array_max.borrow())
    }
    pub fn int_array_mid(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_mid.get() {
            return Ok(self.int_array_mid.borrow());
        }
        self.f_int_array_mid.set(true);
        *self.int_array_mid.borrow_mut() = (*(self.int_array()?.get(1_usize).ok_or(KError::CastError)?)).try_into()?;
        Ok(self.int_array_mid.borrow())
    }
    pub fn int_array_min(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_min.get() {
            return Ok(self.int_array_min.borrow());
        }
        self.f_int_array_min.set(true);
        *self.int_array_min.borrow_mut() = (*self.int_array()?.iter().min().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.int_array_min.borrow())
    }
    pub fn int_array_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_int_array_size.get() {
            return Ok(self.int_array_size.borrow());
        }
        self.f_int_array_size.set(true);
        *self.int_array_size.borrow_mut() = (self.int_array()?.len()).try_into()?;
        Ok(self.int_array_size.borrow())
    }
    pub fn str_array(
        &self
    ) -> KResult<Ref<'_, Vec<String>>> {
        let _io = self._io.borrow();
        if self.f_str_array.get() {
            return Ok(self.str_array.borrow());
        }
        self.f_str_array.set(true);
        *self.str_array.borrow_mut() = vec!["un".to_string(), "deux".to_string(), "trois".to_string(), "quatre".to_string()];
        Ok(self.str_array.borrow())
    }
    pub fn str_array_first(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_array_first.get() {
            return Ok(self.str_array_first.borrow());
        }
        self.f_str_array_first.set(true);
        *self.str_array_first.borrow_mut() = self.str_array()?.first().ok_or(KError::EmptyIterator)?.to_string();
        Ok(self.str_array_first.borrow())
    }
    pub fn str_array_last(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_array_last.get() {
            return Ok(self.str_array_last.borrow());
        }
        self.f_str_array_last.set(true);
        *self.str_array_last.borrow_mut() = self.str_array()?.last().ok_or(KError::EmptyIterator)?.to_string();
        Ok(self.str_array_last.borrow())
    }
    pub fn str_array_max(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_array_max.get() {
            return Ok(self.str_array_max.borrow());
        }
        self.f_str_array_max.set(true);
        *self.str_array_max.borrow_mut() = (*self.str_array()?.iter().max().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.str_array_max.borrow())
    }
    pub fn str_array_mid(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_array_mid.get() {
            return Ok(self.str_array_mid.borrow());
        }
        self.f_str_array_mid.set(true);
        *self.str_array_mid.borrow_mut() = self.str_array()?.get(1_usize).ok_or(KError::CastError)?.to_string();
        Ok(self.str_array_mid.borrow())
    }
    pub fn str_array_min(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_array_min.get() {
            return Ok(self.str_array_min.borrow());
        }
        self.f_str_array_min.set(true);
        *self.str_array_min.borrow_mut() = (*self.str_array()?.iter().min().ok_or(KError::EmptyIterator)?).try_into()?;
        Ok(self.str_array_min.borrow())
    }
    pub fn str_array_size(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str_array_size.get() {
            return Ok(self.str_array_size.borrow());
        }
        self.f_str_array_size.set(true);
        *self.str_array_size.borrow_mut() = (self.str_array()?.len()).try_into()?;
        Ok(self.str_array_size.borrow())
    }
}
impl ExprCalcArrayOps {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
