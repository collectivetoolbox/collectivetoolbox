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
pub struct CombineStr {
    pub(crate) _root: SharedType<CombineStr>,
    pub(crate) _parent: SharedType<CombineStr>,
    pub(crate) _self_shared: SharedType<Self>,
    str_term: RefCell<String>,
    str_limit: RefCell<String>,
    str_eos: RefCell<String>,
    _io: RefCell<BytesReader>,
    str_limit_raw: RefCell<Vec<u8>>,
    f_calc_bytes: Cell<bool>,
    calc_bytes: RefCell<Vec<i32>>,
    f_calc_or_calc_bytes: Cell<bool>,
    calc_or_calc_bytes: RefCell<String>,
    f_eos_or_calc: Cell<bool>,
    eos_or_calc: RefCell<String>,
    f_eos_or_calc_bytes: Cell<bool>,
    eos_or_calc_bytes: RefCell<String>,
    f_limit_or_calc: Cell<bool>,
    limit_or_calc: RefCell<String>,
    f_limit_or_calc_bytes: Cell<bool>,
    limit_or_calc_bytes: RefCell<String>,
    f_limit_or_eos: Cell<bool>,
    limit_or_eos: RefCell<String>,
    f_str_calc: Cell<bool>,
    str_calc: RefCell<String>,
    f_str_calc_bytes: Cell<bool>,
    str_calc_bytes: RefCell<String>,
    f_term_or_calc: Cell<bool>,
    term_or_calc: RefCell<String>,
    f_term_or_calc_bytes: Cell<bool>,
    term_or_calc_bytes: RefCell<String>,
    f_term_or_eos: Cell<bool>,
    term_or_eos: RefCell<String>,
    f_term_or_limit: Cell<bool>,
    term_or_limit: RefCell<String>,
}
impl KStruct for CombineStr {
    type Root = CombineStr;
    type Parent = CombineStr;

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
        *self_rc.str_term.borrow_mut() = bytes_to_str(&_io.read_bytes_term(124, false, true, true)?, "ASCII")?;
        *self_rc.str_limit.borrow_mut() = bytes_to_str(&_io.read_bytes(4_usize)?, "ASCII")?;
        *self_rc.str_eos.borrow_mut() = bytes_to_str(&_io.read_bytes_full()?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CombineStr {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_bytes(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_calc_bytes.get() {
            return Ok(self.calc_bytes.borrow());
        }
        self.f_calc_bytes.set(true);
        *self.calc_bytes.borrow_mut() = vec![98_i32, 97_i32, 122_i32];
        Ok(self.calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn calc_or_calc_bytes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_calc_or_calc_bytes.get() {
            return Ok(self.calc_or_calc_bytes.borrow());
        }
        self.f_calc_or_calc_bytes.set(true);
        *self.calc_or_calc_bytes.borrow_mut() = if false { self.str_calc()?.to_string() } else { self.str_calc_bytes()?.to_string() }.to_string();
        Ok(self.calc_or_calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn eos_or_calc(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_eos_or_calc.get() {
            return Ok(self.eos_or_calc.borrow());
        }
        self.f_eos_or_calc.set(true);
        *self.eos_or_calc.borrow_mut() = if false { self.str_eos().to_string() } else { self.str_calc()?.to_string() }.to_string();
        Ok(self.eos_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn eos_or_calc_bytes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_eos_or_calc_bytes.get() {
            return Ok(self.eos_or_calc_bytes.borrow());
        }
        self.f_eos_or_calc_bytes.set(true);
        *self.eos_or_calc_bytes.borrow_mut() = if true { self.str_eos().to_string() } else { self.str_calc_bytes()?.to_string() }.to_string();
        Ok(self.eos_or_calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn limit_or_calc(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_limit_or_calc.get() {
            return Ok(self.limit_or_calc.borrow());
        }
        self.f_limit_or_calc.set(true);
        *self.limit_or_calc.borrow_mut() = if false { self.str_limit().to_string() } else { self.str_calc()?.to_string() }.to_string();
        Ok(self.limit_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn limit_or_calc_bytes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_limit_or_calc_bytes.get() {
            return Ok(self.limit_or_calc_bytes.borrow());
        }
        self.f_limit_or_calc_bytes.set(true);
        *self.limit_or_calc_bytes.borrow_mut() = if true { self.str_limit().to_string() } else { self.str_calc_bytes()?.to_string() }.to_string();
        Ok(self.limit_or_calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn limit_or_eos(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_limit_or_eos.get() {
            return Ok(self.limit_or_eos.borrow());
        }
        self.f_limit_or_eos.set(true);
        *self.limit_or_eos.borrow_mut() = if true { self.str_limit().to_string() } else { self.str_eos().to_string() }.to_string();
        Ok(self.limit_or_eos.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str_calc(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_calc.get() {
            return Ok(self.str_calc.borrow());
        }
        self.f_str_calc.set(true);
        *self.str_calc.borrow_mut() = "bar".to_string();
        Ok(self.str_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str_calc_bytes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str_calc_bytes.get() {
            return Ok(self.str_calc_bytes.borrow());
        }
        self.f_str_calc_bytes.set(true);
        *self.str_calc_bytes.borrow_mut() = bytes_to_str(&*self.calc_bytes()?, "ASCII")?.to_string();
        Ok(self.str_calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_calc(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_term_or_calc.get() {
            return Ok(self.term_or_calc.borrow());
        }
        self.f_term_or_calc.set(true);
        *self.term_or_calc.borrow_mut() = if true { self.str_term().to_string() } else { self.str_calc()?.to_string() }.to_string();
        Ok(self.term_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_calc_bytes(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_term_or_calc_bytes.get() {
            return Ok(self.term_or_calc_bytes.borrow());
        }
        self.f_term_or_calc_bytes.set(true);
        *self.term_or_calc_bytes.borrow_mut() = if false { self.str_term().to_string() } else { self.str_calc_bytes()?.to_string() }.to_string();
        Ok(self.term_or_calc_bytes.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_eos(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_term_or_eos.get() {
            return Ok(self.term_or_eos.borrow());
        }
        self.f_term_or_eos.set(true);
        *self.term_or_eos.borrow_mut() = if false { self.str_term().to_string() } else { self.str_eos().to_string() }.to_string();
        Ok(self.term_or_eos.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_limit(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_term_or_limit.get() {
            return Ok(self.term_or_limit.borrow());
        }
        self.f_term_or_limit.set(true);
        *self.term_or_limit.borrow_mut() = if true { self.str_term().to_string() } else { self.str_limit().to_string() }.to_string();
        Ok(self.term_or_limit.borrow())
    }
}
impl CombineStr {
    pub fn str_term(&self) -> Ref<'_, String> {
        self.str_term.borrow()
    }
}
impl CombineStr {
    pub fn str_limit(&self) -> Ref<'_, String> {
        self.str_limit.borrow()
    }
}
impl CombineStr {
    pub fn str_eos(&self) -> Ref<'_, String> {
        self.str_eos.borrow()
    }
}
impl CombineStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl CombineStr {
    pub fn str_limit_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_limit_raw.borrow()
    }
}
