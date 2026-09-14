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
pub struct Expr3 {
    pub(crate) _root: SharedType<Expr3>,
    pub(crate) _parent: SharedType<Expr3>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<u8>,
    two: RefCell<String>,
    _io: RefCell<BytesReader>,
    f_four: Cell<bool>,
    four: RefCell<String>,
    f_is_str_eq: Cell<bool>,
    is_str_eq: RefCell<bool>,
    f_is_str_ge: Cell<bool>,
    is_str_ge: RefCell<bool>,
    f_is_str_gt: Cell<bool>,
    is_str_gt: RefCell<bool>,
    f_is_str_le: Cell<bool>,
    is_str_le: RefCell<bool>,
    f_is_str_lt: Cell<bool>,
    is_str_lt: RefCell<bool>,
    f_is_str_lt2: Cell<bool>,
    is_str_lt2: RefCell<bool>,
    f_is_str_ne: Cell<bool>,
    is_str_ne: RefCell<bool>,
    f_test_not: Cell<bool>,
    test_not: RefCell<bool>,
    f_three: Cell<bool>,
    three: RefCell<String>,
}
impl KStruct for Expr3 {
    type Root = Expr3;
    type Parent = Expr3;

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
        *self_rc.one.borrow_mut() = _io.read_u1()?;
        *self_rc.two.borrow_mut() = bytes_to_str(&_io.read_bytes(3_usize)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Expr3 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn four(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_four.get() {
            return Ok(self.four.borrow());
        }
        self.f_four.set(true);
        *self.four.borrow_mut() = format!("{}{}", format!("{}{}", "_", self.two()), "_").to_string();
        Ok(self.four.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_eq.get() {
            return Ok(self.is_str_eq.borrow());
        }
        self.f_is_str_eq.set(true);
        *self.is_str_eq.borrow_mut() = ((self.two().as_str() == "ACK")).try_into()?;
        Ok(self.is_str_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_ge(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_ge.get() {
            return Ok(self.is_str_ge.borrow());
        }
        self.f_is_str_ge.set(true);
        *self.is_str_ge.borrow_mut() = ((self.two().as_str() >= "ACK2")).try_into()?;
        Ok(self.is_str_ge.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_gt(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_gt.get() {
            return Ok(self.is_str_gt.borrow());
        }
        self.f_is_str_gt.set(true);
        *self.is_str_gt.borrow_mut() = ((self.two().as_str() > "ACK2")).try_into()?;
        Ok(self.is_str_gt.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_le(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_le.get() {
            return Ok(self.is_str_le.borrow());
        }
        self.f_is_str_le.set(true);
        *self.is_str_le.borrow_mut() = ((self.two().as_str() <= "ACK2")).try_into()?;
        Ok(self.is_str_le.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_lt(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_lt.get() {
            return Ok(self.is_str_lt.borrow());
        }
        self.f_is_str_lt.set(true);
        *self.is_str_lt.borrow_mut() = ((self.two().as_str() < "ACK2")).try_into()?;
        Ok(self.is_str_lt.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_lt2(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_lt2.get() {
            return Ok(self.is_str_lt2.borrow());
        }
        self.f_is_str_lt2.set(true);
        *self.is_str_lt2.borrow_mut() = ((self.three()?.as_str() < self.two().as_str())).try_into()?;
        Ok(self.is_str_lt2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn is_str_ne(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_is_str_ne.get() {
            return Ok(self.is_str_ne.borrow());
        }
        self.f_is_str_ne.set(true);
        *self.is_str_ne.borrow_mut() = ((self.two().as_str() != "ACK")).try_into()?;
        Ok(self.is_str_ne.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn test_not(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_test_not.get() {
            return Ok(self.test_not.borrow());
        }
        self.f_test_not.set(true);
        *self.test_not.borrow_mut() = (!(false)).try_into()?;
        Ok(self.test_not.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn three(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_three.get() {
            return Ok(self.three.borrow());
        }
        self.f_three.set(true);
        *self.three.borrow_mut() = format!("{}{}", "@", self.two()).to_string();
        Ok(self.three.borrow())
    }
}
impl Expr3 {
    pub fn one(&self) -> Ref<'_, u8> {
        self.one.borrow()
    }
}
impl Expr3 {
    pub fn two(&self) -> Ref<'_, String> {
        self.two.borrow()
    }
}
impl Expr3 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
