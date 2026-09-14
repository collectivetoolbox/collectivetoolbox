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
pub struct Expr2 {
    pub(crate) _root: SharedType<Expr2>,
    pub(crate) _parent: SharedType<Expr2>,
    pub(crate) _self_shared: SharedType<Self>,
    str1: RefCell<OptRc<Expr2_ModStr>>,
    str2: RefCell<OptRc<Expr2_ModStr>>,
    _io: RefCell<BytesReader>,
    f_str1_avg: Cell<bool>,
    str1_avg: RefCell<i32>,
    f_str1_byte1: Cell<bool>,
    str1_byte1: RefCell<u8>,
    f_str1_char5: Cell<bool>,
    str1_char5: RefCell<String>,
    f_str1_len: Cell<bool>,
    str1_len: RefCell<i32>,
    f_str1_len_mod: Cell<bool>,
    str1_len_mod: RefCell<i32>,
    f_str1_tuple5: Cell<bool>,
    str1_tuple5: RefCell<OptRc<Expr2_Tuple>>,
    f_str2_tuple5: Cell<bool>,
    str2_tuple5: RefCell<OptRc<Expr2_Tuple>>,
}
impl KStruct for Expr2 {
    type Root = Expr2;
    type Parent = Expr2;

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
        let t = Self::read_into::<_, Expr2_ModStr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str1.borrow_mut() = t;
        let t = Self::read_into::<_, Expr2_ModStr>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.str2.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Expr2 {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_avg(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str1_avg.get() {
            return Ok(self.str1_avg.borrow());
        }
        self.f_str1_avg.set(true);
        *self.str1_avg.borrow_mut() = (*self.str1().rest().avg()?).try_into()?;
        Ok(self.str1_avg.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_byte1(
        &self
    ) -> KResult<Ref<'_, u8>> {
        let _io = self._io.borrow();
        if self.f_str1_byte1.get() {
            return Ok(self.str1_byte1.borrow());
        }
        self.f_str1_byte1.set(true);
        *self.str1_byte1.borrow_mut() = (*self.str1().rest().byte1()).try_into()?;
        Ok(self.str1_byte1.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_char5(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_str1_char5.get() {
            return Ok(self.str1_char5.borrow());
        }
        self.f_str1_char5.set(true);
        *self.str1_char5.borrow_mut() = self.str1().char5()?.to_string();
        Ok(self.str1_char5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_len(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str1_len.get() {
            return Ok(self.str1_len.borrow());
        }
        self.f_str1_len.set(true);
        *self.str1_len.borrow_mut() = (self.str1().str().len()).try_into()?;
        Ok(self.str1_len.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_len_mod(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_str1_len_mod.get() {
            return Ok(self.str1_len_mod.borrow());
        }
        self.f_str1_len_mod.set(true);
        *self.str1_len_mod.borrow_mut() = (*self.str1().len_mod()?).try_into()?;
        Ok(self.str1_len_mod.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str1_tuple5(
        &self
    ) -> KResult<Ref<'_, OptRc<Expr2_Tuple>>> {
        let _io = self._io.borrow();
        if self.f_str1_tuple5.get() {
            return Ok(self.str1_tuple5.borrow());
        }
        *self.str1_tuple5.borrow_mut() = self.str1().tuple5()?.clone();
        Ok(self.str1_tuple5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn str2_tuple5(
        &self
    ) -> KResult<Ref<'_, OptRc<Expr2_Tuple>>> {
        let _io = self._io.borrow();
        if self.f_str2_tuple5.get() {
            return Ok(self.str2_tuple5.borrow());
        }
        *self.str2_tuple5.borrow_mut() = self.str2().tuple5()?.clone();
        Ok(self.str2_tuple5.borrow())
    }
}
impl Expr2 {
    pub fn str1(&self) -> Ref<'_, OptRc<Expr2_ModStr>> {
        self.str1.borrow()
    }
}
impl Expr2 {
    pub fn str2(&self) -> Ref<'_, OptRc<Expr2_ModStr>> {
        self.str2.borrow()
    }
}
impl Expr2 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Expr2_ModStr {
    pub(crate) _root: SharedType<Expr2>,
    pub(crate) _parent: SharedType<Expr2>,
    pub(crate) _self_shared: SharedType<Self>,
    len_orig: RefCell<u16>,
    str: RefCell<String>,
    rest: RefCell<OptRc<Expr2_Tuple>>,
    _io: RefCell<BytesReader>,
    str_raw: RefCell<Vec<u8>>,
    rest_raw: RefCell<Vec<u8>>,
    f_char5: Cell<bool>,
    char5: RefCell<String>,
    f_len_mod: Cell<bool>,
    len_mod: RefCell<i32>,
    f_tuple5: Cell<bool>,
    tuple5: RefCell<OptRc<Expr2_Tuple>>,
}
impl KStruct for Expr2_ModStr {
    type Root = Expr2;
    type Parent = Expr2;

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
        *self_rc.len_orig.borrow_mut() = _io.read_u2le()?;
        *self_rc.str.borrow_mut() = bytes_to_str(&_io.read_bytes(usize::try_from(*self_rc.len_mod()?)?)?, "UTF-8")?;
        let _raw_rest = _io.read_bytes(3_usize)?;
        *self_rc.rest_raw.borrow_mut() = _raw_rest.clone();
        let _io_rest = BytesReader::from(_raw_rest);
        let t = Self::read_into::<BytesReader, Expr2_Tuple>(&_io_rest, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()))?.into();
        *self_rc.rest.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Expr2_ModStr {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn char5(
        &self
    ) -> KResult<Ref<'_, String>> {
        let _io = self._io.borrow();
        if self.f_char5.get() {
            return Ok(self.char5.borrow());
        }
        self.f_char5.set(true);
        let _pos = _io.pos();
        _io.seek(5_usize)?;
        *self.char5.borrow_mut() = bytes_to_str(&_io.read_bytes(1_usize)?, "ASCII")?;
        _io.seek(_pos)?;
        Ok(self.char5.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn len_mod(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_len_mod.get() {
            return Ok(self.len_mod.borrow());
        }
        self.f_len_mod.set(true);
        *self.len_mod.borrow_mut() = ((i32::from(*self.len_orig())).saturating_sub(3_i32)).try_into()?;
        Ok(self.len_mod.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn tuple5(
        &self
    ) -> KResult<Ref<'_, OptRc<Expr2_Tuple>>> {
        let _io = self._io.borrow();
        if self.f_tuple5.get() {
            return Ok(self.tuple5.borrow());
        }
        let _pos = _io.pos();
        _io.seek(5_usize)?;
        let t = Self::read_into::<_, Expr2_Tuple>(&*_io, Some(self._root.clone()), Some(self._self_shared.clone()))?.into();
        *self.tuple5.borrow_mut() = t;
        _io.seek(_pos)?;
        Ok(self.tuple5.borrow())
    }
}
impl Expr2_ModStr {
    pub fn len_orig(&self) -> Ref<'_, u16> {
        self.len_orig.borrow()
    }
}
impl Expr2_ModStr {
    pub fn str(&self) -> Ref<'_, String> {
        self.str.borrow()
    }
}
impl Expr2_ModStr {
    pub fn rest(&self) -> Ref<'_, OptRc<Expr2_Tuple>> {
        self.rest.borrow()
    }
}
impl Expr2_ModStr {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl Expr2_ModStr {
    pub fn str_raw(&self) -> Ref<'_, Vec<u8>> {
        self.str_raw.borrow()
    }
}
impl Expr2_ModStr {
    pub fn rest_raw(&self) -> Ref<'_, Vec<u8>> {
        self.rest_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Expr2_Tuple {
    pub(crate) _root: SharedType<Expr2>,
    pub(crate) _parent: SharedType<Expr2_ModStr>,
    pub(crate) _self_shared: SharedType<Self>,
    byte0: RefCell<u8>,
    byte1: RefCell<u8>,
    byte2: RefCell<u8>,
    _io: RefCell<BytesReader>,
    f_avg: Cell<bool>,
    avg: RefCell<i32>,
}
impl KStruct for Expr2_Tuple {
    type Root = Expr2;
    type Parent = Expr2_ModStr;

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
        *self_rc.byte0.borrow_mut() = _io.read_u1()?;
        *self_rc.byte1.borrow_mut() = _io.read_u1()?;
        *self_rc.byte2.borrow_mut() = _io.read_u1()?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl Expr2_Tuple {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn avg(
        &self
    ) -> KResult<Ref<'_, i32>> {
        let _io = self._io.borrow();
        if self.f_avg.get() {
            return Ok(self.avg.borrow());
        }
        self.f_avg.set(true);
        *self.avg.borrow_mut() = ((i32::from((*self.byte1()).saturating_add(*self.byte2()))).checked_div(2_i32).ok_or(KError::CastError)?).try_into()?;
        Ok(self.avg.borrow())
    }
}
impl Expr2_Tuple {
    pub fn byte0(&self) -> Ref<'_, u8> {
        self.byte0.borrow()
    }
}
impl Expr2_Tuple {
    pub fn byte1(&self) -> Ref<'_, u8> {
        self.byte1.borrow()
    }
}
impl Expr2_Tuple {
    pub fn byte2(&self) -> Ref<'_, u8> {
        self.byte2.borrow()
    }
}
impl Expr2_Tuple {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
