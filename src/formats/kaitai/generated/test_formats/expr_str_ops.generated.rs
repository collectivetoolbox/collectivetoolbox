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
pub struct ExprStrOps {
    pub(crate) _root: SharedType<ExprStrOps>,
    pub(crate) _parent: SharedType<ExprStrOps>,
    pub(crate) _self_shared: SharedType<Self>,
    one: RefCell<String>,
    _io: RefCell<BytesReader>,
    one_raw: RefCell<Vec<u8>>,
    f_one_len: Cell<bool>,
    one_len: RefCell<i32>,
    f_one_rev: Cell<bool>,
    one_rev: RefCell<String>,
    f_one_substr_0_to_0: Cell<bool>,
    one_substr_0_to_0: RefCell<String>,
    f_one_substr_0_to_3: Cell<bool>,
    one_substr_0_to_3: RefCell<String>,
    f_one_substr_2_to_5: Cell<bool>,
    one_substr_2_to_5: RefCell<String>,
    f_one_substr_3_to_3: Cell<bool>,
    one_substr_3_to_3: RefCell<String>,
    f_to_i_attr: Cell<bool>,
    to_i_attr: RefCell<i32>,
    f_to_i_r10: Cell<bool>,
    to_i_r10: RefCell<i32>,
    f_to_i_r16: Cell<bool>,
    to_i_r16: RefCell<i32>,
    f_to_i_r2: Cell<bool>,
    to_i_r2: RefCell<i32>,
    f_to_i_r8: Cell<bool>,
    to_i_r8: RefCell<i32>,
    f_two: Cell<bool>,
    two: RefCell<String>,
    f_two_len: Cell<bool>,
    two_len: RefCell<i32>,
    f_two_rev: Cell<bool>,
    two_rev: RefCell<String>,
    f_two_substr_0_to_10: Cell<bool>,
    two_substr_0_to_10: RefCell<String>,
    f_two_substr_0_to_7: Cell<bool>,
    two_substr_0_to_7: RefCell<String>,
    f_two_substr_4_to_10: Cell<bool>,
    two_substr_4_to_10: RefCell<String>,
}
impl KStruct for ExprStrOps {
    type Root = ExprStrOps;
    type Parent = ExprStrOps;

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
        *self_rc.one.borrow_mut() = bytes_to_str(&_io.read_bytes(5_usize)?, "ASCII")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprStrOps {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_one_len.get() {
            return Ok(self.one_len.borrow());
        }
        self.f_one_len.set(true);
        *self.one_len.borrow_mut() = (self.one().len()).try_into()?;
        Ok(self.one_len.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_rev(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_one_rev.get() {
            return Ok(self.one_rev.borrow());
        }
        self.f_one_rev.set(true);
        *self.one_rev.borrow_mut() = reverse_string(&self.one())?.to_string();
        Ok(self.one_rev.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_substr_0_to_0(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_one_substr_0_to_0.get() {
            return Ok(self.one_substr_0_to_0.borrow());
        }
        self.f_one_substr_0_to_0.set(true);
        *self.one_substr_0_to_0.borrow_mut() = substring(&self.one(), 0, 0).to_string();
        Ok(self.one_substr_0_to_0.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_substr_0_to_3(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_one_substr_0_to_3.get() {
            return Ok(self.one_substr_0_to_3.borrow());
        }
        self.f_one_substr_0_to_3.set(true);
        *self.one_substr_0_to_3.borrow_mut() = substring(&self.one(), 0, 3).to_string();
        Ok(self.one_substr_0_to_3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_substr_2_to_5(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_one_substr_2_to_5.get() {
            return Ok(self.one_substr_2_to_5.borrow());
        }
        self.f_one_substr_2_to_5.set(true);
        *self.one_substr_2_to_5.borrow_mut() = substring(&self.one(), 2, 5).to_string();
        Ok(self.one_substr_2_to_5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn one_substr_3_to_3(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_one_substr_3_to_3.get() {
            return Ok(self.one_substr_3_to_3.borrow());
        }
        self.f_one_substr_3_to_3.set(true);
        *self.one_substr_3_to_3.borrow_mut() = substring(&self.one(), 3, 3).to_string();
        Ok(self.one_substr_3_to_3.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn to_i_attr(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_attr.get() {
            return Ok(self.to_i_attr.borrow());
        }
        self.f_to_i_attr.set(true);
        *self.to_i_attr.borrow_mut() = ("9173".parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_attr.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn to_i_r10(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r10.get() {
            return Ok(self.to_i_r10.borrow());
        }
        self.f_to_i_r10.set(true);
        *self.to_i_r10.borrow_mut() = ("-072".parse::<i32>().map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r10.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn to_i_r16(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r16.get() {
            return Ok(self.to_i_r16.borrow());
        }
        self.f_to_i_r16.set(true);
        *self.to_i_r16.borrow_mut() = (i32::from_str_radix("47cf", 16).map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r16.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn to_i_r2(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r2.get() {
            return Ok(self.to_i_r2.borrow());
        }
        self.f_to_i_r2.set(true);
        *self.to_i_r2.borrow_mut() = (i32::from_str_radix("1010110", 2).map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn to_i_r8(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_to_i_r8.get() {
            return Ok(self.to_i_r8.borrow());
        }
        self.f_to_i_r8.set(true);
        *self.to_i_r8.borrow_mut() = (i32::from_str_radix("721", 8).map_err(|_| KError::CastError)?).try_into()?;
        Ok(self.to_i_r8.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_two.get() {
            return Ok(self.two.borrow());
        }
        self.f_two.set(true);
        *self.two.borrow_mut() = "0123456789".to_string();
        Ok(self.two.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_two_len.get() {
            return Ok(self.two_len.borrow());
        }
        self.f_two_len.set(true);
        *self.two_len.borrow_mut() = (self.two()?.len()).try_into()?;
        Ok(self.two_len.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two_rev(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_two_rev.get() {
            return Ok(self.two_rev.borrow());
        }
        self.f_two_rev.set(true);
        *self.two_rev.borrow_mut() = reverse_string(&self.two()?)?.to_string();
        Ok(self.two_rev.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two_substr_0_to_10(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_two_substr_0_to_10.get() {
            return Ok(self.two_substr_0_to_10.borrow());
        }
        self.f_two_substr_0_to_10.set(true);
        *self.two_substr_0_to_10.borrow_mut() = substring(&self.two()?, 0, 10).to_string();
        Ok(self.two_substr_0_to_10.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two_substr_0_to_7(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_two_substr_0_to_7.get() {
            return Ok(self.two_substr_0_to_7.borrow());
        }
        self.f_two_substr_0_to_7.set(true);
        *self.two_substr_0_to_7.borrow_mut() = substring(&self.two()?, 0, 7).to_string();
        Ok(self.two_substr_0_to_7.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn two_substr_4_to_10(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_two_substr_4_to_10.get() {
            return Ok(self.two_substr_4_to_10.borrow());
        }
        self.f_two_substr_4_to_10.set(true);
        *self.two_substr_4_to_10.borrow_mut() = substring(&self.two()?, 4, 10).to_string();
        Ok(self.two_substr_4_to_10.borrow())
    }
}
impl ExprStrOps {
    pub fn one(&self) -> Ref<'_, String> {
        self.one.borrow()
    }
}
impl ExprStrOps {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprStrOps {
    pub fn one_raw(&self) -> Ref<'_, Vec<u8>> {
        self.one_raw.borrow()
    }
}
