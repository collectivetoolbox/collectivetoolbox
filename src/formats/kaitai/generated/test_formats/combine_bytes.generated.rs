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
pub struct CombineBytes {
    pub(crate) _root: SharedType<CombineBytes>,
    pub(crate) _parent: SharedType<CombineBytes>,
    pub(crate) _self_shared: SharedType<Self>,
    bytes_term: RefCell<Vec<u8>>,
    bytes_limit: RefCell<Vec<u8>>,
    bytes_eos: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    bytes_limit_raw: RefCell<Vec<u8>>,
    f_bytes_calc: Cell<bool>,
    bytes_calc: RefCell<Vec<i32>>,
    f_eos_or_calc: Cell<bool>,
    eos_or_calc: RefCell<Vec<i32>>,
    f_limit_or_calc: Cell<bool>,
    limit_or_calc: RefCell<Vec<i32>>,
    f_limit_or_eos: Cell<bool>,
    limit_or_eos: RefCell<Vec<u8>>,
    f_term_or_calc: Cell<bool>,
    term_or_calc: RefCell<Vec<i32>>,
    f_term_or_eos: Cell<bool>,
    term_or_eos: RefCell<Vec<u8>>,
    f_term_or_limit: Cell<bool>,
    term_or_limit: RefCell<Vec<u8>>,
}
impl KStruct for CombineBytes {
    type Root = CombineBytes;
    type Parent = CombineBytes;

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
        *self_rc.bytes_term.borrow_mut() = _io.read_bytes_term(124, false, true, true)?;
        *self_rc.bytes_limit.borrow_mut() = _io.read_bytes(4_usize)?;
        *self_rc.bytes_eos.borrow_mut() = _io.read_bytes_full()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl CombineBytes {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn bytes_calc(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_bytes_calc.get() {
            return Ok(self.bytes_calc.borrow());
        }
        self.f_bytes_calc.set(true);
        *self.bytes_calc.borrow_mut() = vec![82_i32, 110_i32, 68_i32];
        Ok(self.bytes_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn eos_or_calc(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_eos_or_calc.get() {
            return Ok(self.eos_or_calc.borrow());
        }
        self.f_eos_or_calc.set(true);
        *self.eos_or_calc.borrow_mut() = if true { self.bytes_eos().to_vec() } else { self.bytes_calc()?.to_vec() }.to_vec();
        Ok(self.eos_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn limit_or_calc(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_limit_or_calc.get() {
            return Ok(self.limit_or_calc.borrow());
        }
        self.f_limit_or_calc.set(true);
        *self.limit_or_calc.borrow_mut() = if false { self.bytes_limit().to_vec() } else { self.bytes_calc()?.to_vec() }.to_vec();
        Ok(self.limit_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn limit_or_eos(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_limit_or_eos.get() {
            return Ok(self.limit_or_eos.borrow());
        }
        self.f_limit_or_eos.set(true);
        *self.limit_or_eos.borrow_mut() = if true { self.bytes_limit().to_vec() } else { self.bytes_eos().to_vec() }.to_vec();
        Ok(self.limit_or_eos.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_calc(
        &self
    ) -> KResult<Ref<'_, Vec<i32>>> {
        let _io = self._io.borrow();
        if self.f_term_or_calc.get() {
            return Ok(self.term_or_calc.borrow());
        }
        self.f_term_or_calc.set(true);
        *self.term_or_calc.borrow_mut() = if true { self.bytes_term().to_vec() } else { self.bytes_calc()?.to_vec() }.to_vec();
        Ok(self.term_or_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_eos(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_term_or_eos.get() {
            return Ok(self.term_or_eos.borrow());
        }
        self.f_term_or_eos.set(true);
        *self.term_or_eos.borrow_mut() = if false { self.bytes_term().to_vec() } else { self.bytes_eos().to_vec() }.to_vec();
        Ok(self.term_or_eos.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn term_or_limit(
        &self
    ) -> KResult<Ref<'_, Vec<u8>>> {
        let _io = self._io.borrow();
        if self.f_term_or_limit.get() {
            return Ok(self.term_or_limit.borrow());
        }
        self.f_term_or_limit.set(true);
        *self.term_or_limit.borrow_mut() = if true { self.bytes_term().to_vec() } else { self.bytes_limit().to_vec() }.to_vec();
        Ok(self.term_or_limit.borrow())
    }
}
impl CombineBytes {
    pub fn bytes_term(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_term.borrow()
    }
}
impl CombineBytes {
    pub fn bytes_limit(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_limit.borrow()
    }
}
impl CombineBytes {
    pub fn bytes_eos(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_eos.borrow()
    }
}
impl CombineBytes {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl CombineBytes {
    pub fn bytes_limit_raw(&self) -> Ref<'_, Vec<u8>> {
        self.bytes_limit_raw.borrow()
    }
}
