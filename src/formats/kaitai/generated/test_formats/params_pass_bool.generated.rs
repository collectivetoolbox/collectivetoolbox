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
pub struct ParamsPassBool {
    pub(crate) _root: SharedType<ParamsPassBool>,
    pub(crate) _parent: SharedType<ParamsPassBool>,
    pub(crate) _self_shared: SharedType<Self>,
    s_false: RefCell<bool>,
    s_true: RefCell<bool>,
    seq_b1: RefCell<OptRc<ParamsPassBool_ParamTypeB1>>,
    seq_bool: RefCell<OptRc<ParamsPassBool_ParamTypeBool>>,
    literal_b1: RefCell<OptRc<ParamsPassBool_ParamTypeB1>>,
    literal_bool: RefCell<OptRc<ParamsPassBool_ParamTypeBool>>,
    inst_b1: RefCell<OptRc<ParamsPassBool_ParamTypeB1>>,
    inst_bool: RefCell<OptRc<ParamsPassBool_ParamTypeBool>>,
    _io: RefCell<BytesReader>,
    f_v_false: Cell<bool>,
    v_false: RefCell<bool>,
    f_v_true: Cell<bool>,
    v_true: RefCell<bool>,
}
impl KStruct for ParamsPassBool {
    type Root = ParamsPassBool;
    type Parent = ParamsPassBool;

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
        *self_rc.s_false.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        *self_rc.s_true.borrow_mut() = _io.read_bits_int_be(1)? != 0;
        io.align_to_byte()?;
        let f = |t : &mut ParamsPassBool_ParamTypeB1| Ok(t.set_params(*self_rc.s_true()));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeB1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.seq_b1.borrow_mut() = t;
        let f = |t : &mut ParamsPassBool_ParamTypeBool| Ok(t.set_params(*self_rc.s_false()));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeBool>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.seq_bool.borrow_mut() = t;
        let f = |t : &mut ParamsPassBool_ParamTypeB1| Ok(t.set_params(false));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeB1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.literal_b1.borrow_mut() = t;
        let f = |t : &mut ParamsPassBool_ParamTypeBool| Ok(t.set_params(true));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeBool>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.literal_bool.borrow_mut() = t;
        let f = |t : &mut ParamsPassBool_ParamTypeB1| Ok(t.set_params(*self_rc.v_true()?));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeB1>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.inst_b1.borrow_mut() = t;
        let f = |t : &mut ParamsPassBool_ParamTypeBool| Ok(t.set_params(*self_rc.v_false()?));
        let t = Self::read_into_with_init::<_, ParamsPassBool_ParamTypeBool>(&*_io, Some(self_rc._root.clone()), Some(self_rc._self_shared.clone()), &f)?.into();
        *self_rc.inst_bool.borrow_mut() = t;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassBool {
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_false(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_v_false.get() {
            return Ok(self.v_false.borrow());
        }
        self.f_v_false.set(true);
        *self.v_false.borrow_mut() = (false).try_into()?;
        Ok(self.v_false.borrow())
    }
    #[allow(clippy::approx_constant, clippy::unnecessary_fallible_conversions, reason = "Generic instance calculation conversion")]
    pub fn v_true(
        &self
    ) -> KResult<Ref<'_, bool>> {
        let _io = self._io.borrow();
        if self.f_v_true.get() {
            return Ok(self.v_true.borrow());
        }
        self.f_v_true.set(true);
        *self.v_true.borrow_mut() = (true).try_into()?;
        Ok(self.v_true.borrow())
    }
}
impl ParamsPassBool {
    pub fn s_false(&self) -> Ref<'_, bool> {
        self.s_false.borrow()
    }
}
impl ParamsPassBool {
    pub fn s_true(&self) -> Ref<'_, bool> {
        self.s_true.borrow()
    }
}
impl ParamsPassBool {
    pub fn seq_b1(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeB1>> {
        self.seq_b1.borrow()
    }
}
impl ParamsPassBool {
    pub fn seq_bool(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeBool>> {
        self.seq_bool.borrow()
    }
}
impl ParamsPassBool {
    pub fn literal_b1(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeB1>> {
        self.literal_b1.borrow()
    }
}
impl ParamsPassBool {
    pub fn literal_bool(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeBool>> {
        self.literal_bool.borrow()
    }
}
impl ParamsPassBool {
    pub fn inst_b1(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeB1>> {
        self.inst_b1.borrow()
    }
}
impl ParamsPassBool {
    pub fn inst_bool(&self) -> Ref<'_, OptRc<ParamsPassBool_ParamTypeBool>> {
        self.inst_bool.borrow()
    }
}
impl ParamsPassBool {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassBool_ParamTypeB1 {
    pub(crate) _root: SharedType<ParamsPassBool>,
    pub(crate) _parent: SharedType<ParamsPassBool>,
    pub(crate) _self_shared: SharedType<Self>,
    arg: RefCell<bool>,
    foo: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    foo_raw: RefCell<Vec<u8>>,
}
impl KStruct for ParamsPassBool_ParamTypeB1 {
    type Root = ParamsPassBool;
    type Parent = ParamsPassBool;

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
        *self_rc.foo.borrow_mut() = _io.read_bytes(usize::try_from(if *self_rc.arg() { 1_i32 } else { 2_i32 })?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassBool_ParamTypeB1 {
    pub fn arg(&self) -> Ref<'_, bool> {
        self.arg.borrow()
    }
}
impl ParamsPassBool_ParamTypeB1 {
    pub fn set_params(&mut self, arg: bool) {
        *self.arg.borrow_mut() = arg;
    }
}
impl ParamsPassBool_ParamTypeB1 {
}
impl ParamsPassBool_ParamTypeB1 {
    pub fn foo(&self) -> Ref<'_, Vec<u8>> {
        self.foo.borrow()
    }
}
impl ParamsPassBool_ParamTypeB1 {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ParamsPassBool_ParamTypeB1 {
    pub fn foo_raw(&self) -> Ref<'_, Vec<u8>> {
        self.foo_raw.borrow()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ParamsPassBool_ParamTypeBool {
    pub(crate) _root: SharedType<ParamsPassBool>,
    pub(crate) _parent: SharedType<ParamsPassBool>,
    pub(crate) _self_shared: SharedType<Self>,
    arg: RefCell<bool>,
    foo: RefCell<Vec<u8>>,
    _io: RefCell<BytesReader>,
    foo_raw: RefCell<Vec<u8>>,
}
impl KStruct for ParamsPassBool_ParamTypeBool {
    type Root = ParamsPassBool;
    type Parent = ParamsPassBool;

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
        *self_rc.foo.borrow_mut() = _io.read_bytes(usize::try_from(if *self_rc.arg() { 1_i32 } else { 2_i32 })?)?;
        *self_rc._io.borrow_mut() = io.clone();
        Ok(())
    }
}
impl ParamsPassBool_ParamTypeBool {
    pub fn arg(&self) -> Ref<'_, bool> {
        self.arg.borrow()
    }
}
impl ParamsPassBool_ParamTypeBool {
    pub fn set_params(&mut self, arg: bool) {
        *self.arg.borrow_mut() = arg;
    }
}
impl ParamsPassBool_ParamTypeBool {
}
impl ParamsPassBool_ParamTypeBool {
    pub fn foo(&self) -> Ref<'_, Vec<u8>> {
        self.foo.borrow()
    }
}
impl ParamsPassBool_ParamTypeBool {
    pub fn _io(&self) -> Ref<'_, BytesReader> {
        self._io.borrow()
    }
}
impl ParamsPassBool_ParamTypeBool {
    pub fn foo_raw(&self) -> Ref<'_, Vec<u8>> {
        self.foo_raw.borrow()
    }
}
