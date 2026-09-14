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
pub struct ExprStrEncodings {
    pub(crate) _root: SharedType<ExprStrEncodings>,
    pub(crate) _parent: SharedType<ExprStrEncodings>,
    pub(crate) _self_shared: SharedType<Self>,
    len_of_1: RefCell<u16>,
    str1: RefCell<String>,
    len_of_2: RefCell<u16>,
    str2: RefCell<String>,
    len_of_3: RefCell<u16>,
    str3: RefCell<String>,
    len_of_4: RefCell<u16>,
    str4: RefCell<String>,
    _io: RefCell<BytesReader>,
    str1_raw: RefCell<Vec<u8>>,
    str2_raw: RefCell<Vec<u8>>,
    str3_raw: RefCell<Vec<u8>>,
    str4_raw: RefCell<Vec<u8>>,
    f_str1_eq: Cell<bool>,
    str1_eq: RefCell<bool>,
    f_str2_eq: Cell<bool>,
    str2_eq: RefCell<bool>,
    f_str3_eq: Cell<bool>,
    str3_eq: RefCell<bool>,
    f_str3_eq_str2: Cell<bool>,
    str3_eq_str2: RefCell<bool>,
    f_str4_eq: Cell<bool>,
    str4_eq: RefCell<bool>,
    f_str4_gt_str_calc: Cell<bool>,
    str4_gt_str_calc: RefCell<bool>,
    f_str4_gt_str_from_bytes: Cell<bool>,
    str4_gt_str_from_bytes: RefCell<bool>,
}
impl KStruct for ExprStrEncodings {
    type Root = ExprStrEncodings;
    type Parent = ExprStrEncodings;

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
        *self_rc.len_of_1.borrow_mut() = _io.read_u2le()?;
        *self_rc.str1.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_1()))?, "ASCII")?;
        *self_rc.len_of_2.borrow_mut() = _io.read_u2le()?;
        *self_rc.str2.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_2()))?, "UTF-8")?;
        *self_rc.len_of_3.borrow_mut() = _io.read_u2le()?;
        *self_rc.str3.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_3()))?, "SJIS")?;
        *self_rc.len_of_4.borrow_mut() = _io.read_u2le()?;
        *self_rc.str4.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::from(*self_rc.len_of_4()))?, "IBM437")?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ExprStrEncodings {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str1_eq.get() {
            return Ok(self.str1_eq.borrow());
        }
        self.f_str1_eq.set(true);
        *self.str1_eq.borrow_mut() = ((self.str1().as_str() == "Some ASCII")).try_into()?;
        Ok(self.str1_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str2_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str2_eq.get() {
            return Ok(self.str2_eq.borrow());
        }
        self.f_str2_eq.set(true);
        *self.str2_eq.borrow_mut() = ((self.str2().as_str() == "こんにちは")).try_into()?;
        Ok(self.str2_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str3_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str3_eq.get() {
            return Ok(self.str3_eq.borrow());
        }
        self.f_str3_eq.set(true);
        *self.str3_eq.borrow_mut() = ((self.str3().as_str() == "こんにちは")).try_into()?;
        Ok(self.str3_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str3_eq_str2(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str3_eq_str2.get() {
            return Ok(self.str3_eq_str2.borrow());
        }
        self.f_str3_eq_str2.set(true);
        *self.str3_eq_str2.borrow_mut() = ((self.str3().as_str() == self.str2().as_str())).try_into()?;
        Ok(self.str3_eq_str2.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str4_eq(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str4_eq.get() {
            return Ok(self.str4_eq.borrow());
        }
        self.f_str4_eq.set(true);
        *self.str4_eq.borrow_mut() = ((self.str4().as_str() == "░▒▓")).try_into()?;
        Ok(self.str4_eq.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str4_gt_str_calc(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str4_gt_str_calc.get() {
            return Ok(self.str4_gt_str_calc.borrow());
        }
        self.f_str4_gt_str_calc.set(true);
        *self.str4_gt_str_calc.borrow_mut() = ((self.str4().as_str() > "┤")).try_into()?;
        Ok(self.str4_gt_str_calc.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str4_gt_str_from_bytes(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_str4_gt_str_from_bytes.get() {
            return Ok(self.str4_gt_str_from_bytes.borrow());
        }
        self.f_str4_gt_str_from_bytes.set(true);
        *self.str4_gt_str_from_bytes.borrow_mut() = ((self.str4().as_str() > bytes_to_str(&vec![0xb4u8], "IBM437")?.as_str())).try_into()?;
        Ok(self.str4_gt_str_from_bytes.borrow())
    }
}
impl ExprStrEncodings {
    pub fn len_of_1(&self) -> Ref<'_, u16> {
        self.len_of_1.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str1(&self) -> Ref<'_, String> {
        self.str1.borrow()
    }
}
impl ExprStrEncodings {
    pub fn len_of_2(&self) -> Ref<'_, u16> {
        self.len_of_2.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str2(&self) -> Ref<'_, String> {
        self.str2.borrow()
    }
}
impl ExprStrEncodings {
    pub fn len_of_3(&self) -> Ref<'_, u16> {
        self.len_of_3.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str3(&self) -> Ref<'_, String> {
        self.str3.borrow()
    }
}
impl ExprStrEncodings {
    pub fn len_of_4(&self) -> Ref<'_, u16> {
        self.len_of_4.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str4(&self) -> Ref<'_, String> {
        self.str4.borrow()
    }
}
impl ExprStrEncodings {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str1_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str1_raw.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str2_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str2_raw.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str3_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str3_raw.borrow()
    }
}
impl ExprStrEncodings {
    pub fn str4_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str4_raw.borrow()
    }
}
