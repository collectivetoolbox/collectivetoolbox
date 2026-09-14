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
pub struct ExprIfIntEq {
    pub(crate) _root: SharedType<ExprIfIntEq>,
    pub(crate) _parent: SharedType<ExprIfIntEq>,
    pub(crate) _self_shared: SharedType<Self>,
    skip: RefCell<Vec<u8>>,
    seq: RefCell<i16>,
    seq_if: RefCell<i16>,
    _io: RefCell<BytesReader>,
    f_calc: Cell<bool>,
    calc: RefCell<i32>,
    f_calc_eq_calc_if: Cell<bool>,
    calc_eq_calc_if: RefCell<bool>,
    f_calc_eq_lit: Cell<bool>,
    calc_eq_lit: RefCell<bool>,
    f_calc_eq_seq_if: Cell<bool>,
    calc_eq_seq_if: RefCell<bool>,
    f_calc_if: Cell<bool>,
    calc_if: RefCell<i32>,
    f_calc_if_eq_lit: Cell<bool>,
    calc_if_eq_lit: RefCell<bool>,
    f_calc_if_eq_seq_if: Cell<bool>,
    calc_if_eq_seq_if: RefCell<bool>,
    f_seq_eq_calc: Cell<bool>,
    seq_eq_calc: RefCell<bool>,
    f_seq_eq_calc_if: Cell<bool>,
    seq_eq_calc_if: RefCell<bool>,
    f_seq_eq_lit: Cell<bool>,
    seq_eq_lit: RefCell<bool>,
    f_seq_eq_seq_if: Cell<bool>,
    seq_eq_seq_if: RefCell<bool>,
    f_seq_if_eq_lit: Cell<bool>,
    seq_if_eq_lit: RefCell<bool>,
}
impl KStruct for ExprIfIntEq {
    type Root = ExprIfIntEq;
    type Parent = ExprIfIntEq;

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
        *self_rc.skip.borrow_mut() = _io.read_bytes(2_usize)?;
        *self_rc.seq.borrow_mut() = _io.read_s2le()?;
        if true {
            *self_rc.seq_if.borrow_mut() = _io.read_s2le()?;
        }
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprIfIntEq {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_calc.get() {
            return Ok(self.calc.borrow());
        }
        self.f_calc.set(true);
        *self.calc.borrow_mut() = (16705).try_into()?;
        Ok(self.calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_eq_calc_if(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_calc_eq_calc_if.get() {
            return Ok(self.calc_eq_calc_if.borrow());
        }
        self.f_calc_eq_calc_if.set(true);
        *self.calc_eq_calc_if.borrow_mut() = (*self.calc()? == *self.calc_if()?).try_into()?;
        Ok(self.calc_eq_calc_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_eq_lit(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_calc_eq_lit.get() {
            return Ok(self.calc_eq_lit.borrow());
        }
        self.f_calc_eq_lit.set(true);
        *self.calc_eq_lit.borrow_mut() = (*self.calc()? == 16705).try_into()?;
        Ok(self.calc_eq_lit.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_eq_seq_if(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_calc_eq_seq_if.get() {
            return Ok(self.calc_eq_seq_if.borrow());
        }
        self.f_calc_eq_seq_if.set(true);
        *self.calc_eq_seq_if.borrow_mut() = (((to_i128(*self.calc()?)) == (to_i128(*self.seq_if())))).try_into()?;
        Ok(self.calc_eq_seq_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_if(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_calc_if.get() {
            return Ok(self.calc_if.borrow());
        }
        self.f_calc_if.set(true);
        if true {
            *self.calc_if.borrow_mut() = (16705).try_into()?;
        }
        Ok(self.calc_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_if_eq_lit(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_calc_if_eq_lit.get() {
            return Ok(self.calc_if_eq_lit.borrow());
        }
        self.f_calc_if_eq_lit.set(true);
        *self.calc_if_eq_lit.borrow_mut() = (*self.calc_if()? == 16705).try_into()?;
        Ok(self.calc_if_eq_lit.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_if_eq_seq_if(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_calc_if_eq_seq_if.get() {
            return Ok(self.calc_if_eq_seq_if.borrow());
        }
        self.f_calc_if_eq_seq_if.set(true);
        *self.calc_if_eq_seq_if.borrow_mut() = (((to_i128(*self.calc_if()?)) == (to_i128(*self.seq_if())))).try_into()?;
        Ok(self.calc_if_eq_seq_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seq_eq_calc(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_seq_eq_calc.get() {
            return Ok(self.seq_eq_calc.borrow());
        }
        self.f_seq_eq_calc.set(true);
        *self.seq_eq_calc.borrow_mut() = (((to_i128(*self.seq())) == (to_i128(*self.calc()?)))).try_into()?;
        Ok(self.seq_eq_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seq_eq_calc_if(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_seq_eq_calc_if.get() {
            return Ok(self.seq_eq_calc_if.borrow());
        }
        self.f_seq_eq_calc_if.set(true);
        *self.seq_eq_calc_if.borrow_mut() = (((to_i128(*self.seq())) == (to_i128(*self.calc_if()?)))).try_into()?;
        Ok(self.seq_eq_calc_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seq_eq_lit(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_seq_eq_lit.get() {
            return Ok(self.seq_eq_lit.borrow());
        }
        self.f_seq_eq_lit.set(true);
        *self.seq_eq_lit.borrow_mut() = (((to_i128(*self.seq())) == (to_i128(16705)))).try_into()?;
        Ok(self.seq_eq_lit.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seq_eq_seq_if(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_seq_eq_seq_if.get() {
            return Ok(self.seq_eq_seq_if.borrow());
        }
        self.f_seq_eq_seq_if.set(true);
        *self.seq_eq_seq_if.borrow_mut() = (*self.seq() == *self.seq_if()).try_into()?;
        Ok(self.seq_eq_seq_if.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn seq_if_eq_lit(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_seq_if_eq_lit.get() {
            return Ok(self.seq_if_eq_lit.borrow());
        }
        self.f_seq_if_eq_lit.set(true);
        *self.seq_if_eq_lit.borrow_mut() = (((to_i128(*self.seq_if())) == (to_i128(16705)))).try_into()?;
        Ok(self.seq_if_eq_lit.borrow())
    }
}
impl ExprIfIntEq {
    pub fn skip(&self) -> Ref<'_, Vec<u8>> {
        self.skip.borrow()
    }
}
impl ExprIfIntEq {
    pub fn seq(&self) -> Ref<'_, i16> {
        self.seq.borrow()
    }
}
impl ExprIfIntEq {
    pub fn seq_if(&self) -> Ref<'_, i16> {
        self.seq_if.borrow()
    }
}
impl ExprIfIntEq {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
